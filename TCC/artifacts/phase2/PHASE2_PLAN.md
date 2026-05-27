# Phase 2 Plan — Slurm and MANA Installation Flow

**Goal:** Bootstrap a complete, working Slurm + MANA execution environment automatically
on a spawned 1-head + N-worker cluster, using role-specific `init_commands` dispatched
via SSM.

**Depends on:**
- Phase 0 AMI `ami-08b9f0fb120be798a` (us-east-1 build; replicated to us-west-2 as `ami-053c434f0309cfbe0`)
- Phase 1 role-aware node model and head-first provisioning order

**This version is post-fix:**
- Uses `MpiDefault=pmi2` (not `pmix`) for AL2023 plugin compatibility.
- Uses hardened Slurm runtime fields and stale-state cleanup on head startup.
- Assumes EFS mount is done by mount-target IP in spawn step 18 (DNS-independent) with bounded retries.
- Validation commands include explicit `--mpi=pmi2` where appropriate for reproducibility.

---

## Background: How init_commands Work

From reading the codebase:

1. Each node's `init_commands` list (from `cluster.yaml`) is stored as `ShellCommand` rows
   in the DB at `cluster create` time.
2. At `cluster spawn` time (`resource_manager.rs` step 19), per-node commands are fetched,
   joined with ` && `, and dispatched as a single SSM `AWS-RunShellScript` command.
3. The SSM command is wrapped as `sudo -u ec2-user -i bash << 'EOF' ... EOF` — runs as
   ec2-user with a login shell (profile.d scripts loaded).
4. **All nodes' SSM commands are dispatched concurrently**, then polled for completion.
   There is NO sequential head-before-worker guarantee at the SSM dispatch level.
5. Private IPs are deterministic (set by ENI before EC2 boot):
   - Head (node_index 0 after Phase 1 sort): `10.0.0.10`, hostname `ip-10-0-0-10`
   - Worker 1 (node_index 1): `10.0.0.11`, hostname `ip-10-0-0-11`
   - Worker 2 (node_index 2): `10.0.0.12`, hostname `ip-10-0-0-12`
   - (Pattern: `10.0.0.{node_index + 10}`)
6. When EFS is enabled, it is mounted on all nodes at `/shared` **before** init_commands
   run (step 18 completes before step 19).

**Coordination strategy:** Since all init_commands run concurrently, the worker scripts
must poll EFS for a sentinel file written by the head. The head writes
`/shared/slurm/head_ready` as its last init action. Workers wait up to 300 seconds for
this file before proceeding.

---

## Overview of Changes

| Layer | File | What changes |
|---|---|---|
| Rust — spawn | `src/integrations/providers/aws/resource_manager.rs` | Inject `HPCAC_*` env-vars into each node's SSM script |
| YAML — cluster | `cluster.example.yaml` | Add role-specific `init_commands` for head and workers |
| YAML — tasks | `tasks.example.yaml` | Update to use Slurm (`srun`) for multi-node NPB run |

---

## Runtime Fixes Already Incorporated

1. **EFS attach reliability:** spawn step 18 mounts EFS using mount-target IP and `nfsvers=4.1`, with bounded retries.
2. **Terminate recoverability:** if terminate fails mid-flight, cluster state is set to `failed` (prevents permanent `terminating`).
3. **Slurm plugin compatibility:** cluster template defaults to `MpiDefault=pmi2` because `pmix` is unavailable in this AMI.
4. **Head Slurm hardening:** `SlurmctldHost` + `SlurmctldAddr`, explicit pid/spool/state dirs, stale cluster state cleanup, and service health checks.

---

## Step 1 — Inject Cluster Context Env-Vars into SSM Init Scripts

**File:** `src/integrations/providers/aws/resource_manager.rs`

**Why:** init_commands in the YAML must not hardcode IPs or node counts, because
`cluster.example.yaml` is meant to work for any cluster size. The Rust spawner knows the
full node list at dispatch time and can inject it cleanly.

**Where:** In step 19, after fetching `node_init_commands` for a node and before joining
them with `&&`, prepend an export block as the first element of the command list.

**Variables to inject:**

| Variable | Value | Example |
|---|---|---|
| `HPCAC_NODE_ROLE` | `node.role` | `head` or `worker` |
| `HPCAC_NODE_INDEX` | `node_index` | `0`, `1`, `2` |
| `HPCAC_HEAD_PRIVATE_IP` | always `10.0.0.10` | `10.0.0.10` |
| `HPCAC_NODE_COUNT` | `nodes.len()` | `3` |
| `HPCAC_WORKER_COUNT` | `nodes.len() - 1` | `2` |
| `HPCAC_USE_EFS` | `cluster.use_elastic_file_system` | `true` or `false` |
| `HPCAC_EFS_MOUNT` | `/shared` (constant) | `/shared` |

**Implementation:**

In `resource_manager.rs` step 19, after:
```rust
let mut node_init_commands = node.get_init_commands(pool).await?;
```

Add:
```rust
let worker_count = nodes.len() - 1;
let env_block = format!(
    "export HPCAC_NODE_ROLE={role} \
     HPCAC_NODE_INDEX={idx} \
     HPCAC_HEAD_PRIVATE_IP=10.0.0.10 \
     HPCAC_NODE_COUNT={count} \
     HPCAC_WORKER_COUNT={wcount} \
     HPCAC_USE_EFS={efs} \
     HPCAC_EFS_MOUNT=/shared",
    role = node.role,
    idx = node_index,
    count = nodes.len(),
    wcount = worker_count,
    efs = cluster.use_elastic_file_system,
);
node_init_commands.insert(0, env_block);
```

This single line is prepended before the SSH key setup script is inserted at index 0
(adjust insertion order so env_block is first, then ssh_key_setup_script second).

---

## Step 2 — Head Init Script

The head init_commands are a sequence of bash one-liners (to be written in
`cluster.example.yaml` under the head node's `init_commands` list).

**What the head must do:**
1. Generate munge key and start munge.
2. Generate `slurm.conf` covering all nodes (using known IP scheme + injected vars).
3. Start `slurmctld` (and `slurmd` on head).
4. Create the shared EFS checkpoint directory.
5. Copy munge key and slurm.conf to EFS so workers can read them.
6. Write sentinel `/shared/slurm/head_ready` to unblock workers.

**Key slurm.conf fields for AL2023 (from Phase 0):**
- `CgroupPlugin=disabled` (separate cgroup.conf, already baked in AMI)
- `TaskPlugin=task/none`
- `MpiDefault=pmi2`
- `SlurmctldHost=ip-10-0-0-10` and `SlurmctldAddr=10.0.0.10` (derived from `HPCAC_HEAD_PRIVATE_IP`)
- `SlurmUser=slurm`, explicit pid/state/spool dirs (`SlurmctldPidFile`, `SlurmdPidFile`, `StateSaveLocation`, `SlurmdSpoolDir`)
- One `NodeName` line per node (head + each worker), using deterministic IPs
- Head startup must remove stale controller state (`/var/spool/slurmctld/clustername`, `*.old`) before starting `slurmctld`

**Script structure (each bullet = one entry in `init_commands`):**
```yaml
init_commands:
  # 1. Munge key generation
  - sudo bash -c 'dd if=/dev/urandom bs=1 count=1024 2>/dev/null > /etc/munge/munge.key && chmod 400 /etc/munge/munge.key && chown munge:munge /etc/munge/munge.key'
  - sudo systemctl start munge && sudo systemctl enable munge

  # 2. Slurm.conf generation — head NodeName line
  - HEAD_HOST="ip-${HPCAC_HEAD_PRIVATE_IP//./-}" && sudo bash -c "printf '%s\n' 'ClusterName=hpcac' 'SlurmctldHost='${HEAD_HOST} 'SlurmctldAddr='${HPCAC_HEAD_PRIVATE_IP} 'SlurmUser=slurm' 'SlurmdUser=root' 'SlurmctldPidFile=/var/spool/slurmctld/slurmctld.pid' 'SlurmdPidFile=/var/spool/slurmd/slurmd.pid' 'StateSaveLocation=/var/spool/slurmctld' 'SlurmdSpoolDir=/var/spool/slurmd' 'MailProg=/bin/true' 'AuthType=auth/munge' 'MpiDefault=pmi2' 'ProctrackType=proctrack/linuxproc' 'TaskPlugin=task/none' 'SchedulerType=sched/backfill' 'SelectType=select/cons_tres' 'SelectTypeParameters=CR_Core' 'JobAcctGatherType=jobacct_gather/none' 'SlurmctldLogFile=/var/log/slurm/slurmctld.log' 'SlurmdLogFile=/var/log/slurm/slurmd.log' 'NodeName='${HEAD_HOST}' NodeAddr='${HPCAC_HEAD_PRIVATE_IP}' State=UNKNOWN' > /etc/slurm/slurm.conf"

  # 3. Append worker NodeName lines
  - for i in $(seq 1 "$HPCAC_WORKER_COUNT"); do WHOST="ip-10-0-0-$((10+i))"; WIP="10.0.0.$((10+i))"; sudo bash -c "echo 'NodeName='${WHOST}' NodeAddr='${WIP}' State=UNKNOWN' >> /etc/slurm/slurm.conf"; done

  # 4. Append partition
  - sudo bash -c "echo 'PartitionName=hpcac Nodes=ALL Default=YES MaxTime=INFINITE State=UP' >> /etc/slurm/slurm.conf"

  # 5. Create runtime dirs, clean stale state, and start Slurm daemons on head
  - sudo mkdir -p /var/log/slurm /var/spool/slurmctld /var/spool/slurmd && sudo chown -R slurm:slurm /var/log/slurm /var/spool/slurmctld /var/spool/slurmd
  - sudo rm -f /var/spool/slurmctld/clustername /var/spool/slurmctld/*.old
  - sudo systemctl enable --now slurmctld
  - sudo systemctl is-active --quiet slurmctld || (sudo journalctl -u slurmctld -n 80 --no-pager; exit 1)
  - sudo systemctl enable --now slurmd
  - sudo systemctl is-active --quiet slurmd || (sudo journalctl -u slurmd -n 80 --no-pager; exit 1)

  # 6. Share to EFS
  - sudo mkdir -p /shared/slurm /shared/checkpoints
  - sudo chown ec2-user:ec2-user /shared/slurm /shared/checkpoints && sudo chmod 775 /shared/slurm /shared/checkpoints
  - sudo cp /etc/munge/munge.key /shared/slurm/munge.key && sudo chmod 644 /shared/slurm/munge.key
  - sudo cp /etc/slurm/slurm.conf /shared/slurm/slurm.conf

  # 7. Write sentinel — workers are blocked until this exists
  - sudo bash -c 'echo ok > /shared/slurm/head_ready'
```

---

## Step 3 — Worker Init Script

**What each worker must do:**
1. Poll EFS for `/shared/slurm/head_ready` (max 300 s).
2. Copy munge key from EFS, start munge.
3. Copy `slurm.conf` from EFS, start `slurmd`.
4. Validate `slurmd` is active and fail fast with journald logs if startup fails.

**Script structure:**
```yaml
init_commands:
  # 1. Wait for head to be ready
  - timeout=300; elapsed=0; while [ ! -f /shared/slurm/head_ready ]; do sleep 5; elapsed=$((elapsed+5)); [ $elapsed -ge $timeout ] && echo "TIMEOUT waiting for head" && exit 1; done

  # 2. Munge key from EFS
  - sudo cp /shared/slurm/munge.key /etc/munge/munge.key && sudo chmod 400 /etc/munge/munge.key && sudo chown munge:munge /etc/munge/munge.key
  - sudo systemctl enable --now munge

  # 3. Slurm.conf from EFS
  - sudo cp /shared/slurm/slurm.conf /etc/slurm/slurm.conf

  # 4. Start slurmd
  - sudo mkdir -p /var/log/slurm /var/spool/slurmd && sudo chown slurm:slurm /var/log/slurm /var/spool/slurmd
  - sudo systemctl enable --now slurmd
  - sudo systemctl is-active --quiet slurmd || (sudo journalctl -u slurmd -n 80 --no-pager; exit 1)
```

---

## Step 4 — Update cluster.example.yaml

Add `init_commands` to the head and worker node entries using the scripts from Steps 2
and 3. The `use_elastic_file_system: true` setting must be present (required for the
coordination strategy to work).

---

## Step 5 — Update tasks.example.yaml

Replace the legacy `mpirun -hostfile` pattern with Slurm-native `srun`. The head node
is the entry point for all task commands.

**New tasks.example.yaml structure:**
```yaml
tasks:
  - task_tag: "slurm_smoke_test"
    setup_commands:
      - scontrol show nodes
    run_commands: []

  - task_tag: "npb_ep_a"
    setup_commands: []
    run_commands:
      - srun --mpi=pmi2 -N2 -n4 /home/ec2-user/downloads/NPB3.4.4/NPB3.4-MPI/bin/ep.A.x

  - task_tag: "npb_cg_a"
    setup_commands: []
    run_commands:
      - srun --mpi=pmi2 -N2 -n4 /home/ec2-user/downloads/NPB3.4.4/NPB3.4-MPI/bin/cg.A.x
```

If `MpiDefault=pmi2` is present in `/etc/slurm/slurm.conf`, `--mpi=pmi2` is optional but still recommended for explicitness in validation logs.

**Note:** `run_task` dispatches to ALL nodes in the cluster. Slurm tasks should target
only the head node (`node.role == "head"`). This is a known limitation to flag for
Phase 3 — currently run_task sends the srun command to every node, which is harmless
(only the head runs slurmctld and can submit jobs) but wasteful. A cleaner fix is
scoping `run_task` to the head node, but that is out of Phase 2 scope.

---

## Step 6 — Validation Procedure

After spawning the cluster with `cluster spawn`:

### 6.1 Verify Slurm is healthy
SSH to head (`10.0.0.10`):
```bash
scontrol show nodes
# Expect: all nodes appear, State=IDLE
sinfo -N
# Expect: hpcac partition, all nodes listed
sudo systemctl is-active slurmctld slurmd
# Expect: active active (on head)
```

### 6.2 Run NPB under Slurm
```bash
srun --mpi=pmi2 -N2 -n4 ~/downloads/NPB3.4.4/NPB3.4-MPI/bin/ep.A.x
# Expect: "Verification SUCCESSFUL"
```

### 6.3 MANA checkpoint/restart on multi-node (head only)
```bash
salloc -N2 -n4 -t 00:30:00

# Always start from clean coordinator/process state
sudo pkill -9 -f dmtcp_coordinator || true
sudo pkill -9 -f mana_coordinator || true
srun --overlap --immediate=5 -N2 -n2 --label /bin/sh -c 'pkill -9 -f lower-half || true; pkill -9 -f mana_launch || true; pkill -9 -f dmtcp_launch || true; pkill -9 -f dmtcp_worker || true'
rm -f "$HOME/.mana-slurm-$SLURM_JOB_ID.rc"

# Ensure shared checkpoint path is writable by ec2-user
sudo chown ec2-user:ec2-user /shared/checkpoints
sudo chmod 775 /shared/checkpoints
rm -rf /shared/checkpoints/*

# Start coordinator on fixed port for deterministic troubleshooting
/opt/mana/bin/mana_coordinator --port 7779 --ckptdir /shared/checkpoints --exit-on-last &
sleep 1

# Publish job-scoped rc file to all nodes (HOME is not shared across nodes)
RC="/shared/slurm/.mana-slurm-${SLURM_JOB_ID}.rc"
DEST=".mana-slurm-${SLURM_JOB_ID}.rc"
sudo cp -f "$HOME/$DEST" "$RC"
sudo chmod 644 "$RC"
srun --overlap --immediate=5 -N2 -n2 --label env RC="$RC" DEST="$DEST" /bin/sh -c 'cp -f "$RC" "$HOME/$DEST"'

# Export coordinator endpoint for all step commands
export DMTCP_COORD_HOST=ip-10-0-0-10.ec2.internal
export DMTCP_COORD_PORT=7779

# Launch MANA-wrapped MPI job
srun --overlap --mpi=pmi2 -N2 -n4 /opt/mana/bin/mana_launch --ckptdir /shared/checkpoints /opt/mana/mpi-proxy-split/test/mpi_hello_world.mana.exe &
sleep 8
mana_status --list  # Expect exactly 4 clients, all WorkerState::RUNNING

# Trigger blocking checkpoint and verify artifacts
mana_status --bcheckpoint
find /shared/checkpoints -maxdepth 3 -ls

# Quit original run
mana_status --quit

# Restart from checkpoint (same allocation)
/opt/mana/bin/mana_coordinator --port 7779 --ckptdir /shared/checkpoints --exit-on-last &
sleep 1
srun --overlap --mpi=pmi2 -N2 -n4 /opt/mana/bin/mana_restart --restartdir /shared/checkpoints &
```

### 6.4 EFS checkpoint dir visible on all workers
SSH to worker (`10.0.0.11`):
```bash
sudo systemctl is-active munge slurmd
# Expect: active active
ls /shared/checkpoints
# Expect: same checkpoint dirs created on head
```

### 6.5 Quick troubleshooting checks (if validation fails)
```bash
# On head
sudo journalctl -u slurmctld -n 120 --no-pager
sudo journalctl -u slurmd -n 120 --no-pager

# On worker
sudo journalctl -u munge -n 120 --no-pager
sudo journalctl -u slurmd -n 120 --no-pager

# Confirm plugins available (expect pmi2 and no pmix)
srun --mpi=list
```

---

## Acceptance Criteria

- [x] `cargo build` passes after Step 1 Rust change.
- [x] Head node: `slurmctld` running and both nodes show State=IDLE after spawn.
- [x] Worker node(s): `slurmd` running and registered with head.
- [x] NPB EP.A runs successfully under `srun --mpi=pmi2` across 2 nodes.
- [x] MANA checkpoint creates dirs in `/shared/checkpoints`.
- [x] MANA restart succeeds from `/shared/checkpoints` on multi-node cluster.
- [x] `/shared/checkpoints` is visible on all nodes (EFS consistency confirmed).
- [x] Phase 1 acceptance criterion confirmed: head is node_index 0 in EC2 spawn logs.

---

## Notes for Re-runs

- Always terminate previous test clusters before respawn to avoid stale shared state confusion.
- Keep cluster name stable (`hpcac`) unless you also clean stale Slurm state on head.
- If a command works without `sudo` but fails with `sudo` (`scontrol` not found), use plain user command or full binary path (for example `/opt/slurm-24.05.4/bin/scontrol`).
- **AMI bug (Phase 0 single-node setup):** `slurmctld` was enabled on all nodes in the AMI. Workers must never run `slurmctld`. The AMI also has stale `/var/spool/slurmctld/clustername` containing `local`, which causes `CLUSTER NAME MISMATCH` on every worker at boot. Fixed in `cluster.example.yaml` worker step 0 (stop+disable+clear). Fixed in `AMI_SETUP_COMMANDS.md` section 9 for the next AMI rebuild.
- **NodeName lines must include `CPUs=N`:** without an explicit `CPUs=` spec, Slurm defaults to 1 CPU per node, making `-n4` impossible to schedule on a 3-node cluster. `cluster.example.yaml` now sets `CPUs=$(nproc)` on head and `CPUs=2` on `m5.large` workers. Update `CPUs=` if you change the worker instance type.
- **`scontrol reconfigure` requires sudo + full path:** `sudo /opt/slurm-24.05.4/bin/scontrol reconfigure`. Running without sudo gives `Invalid user id`.
- **Per-allocation MANA rc file:** each `salloc` creates a new `SLURM_JOB_ID`, and MANA expects `$HOME/.mana-slurm-$SLURM_JOB_ID.rc` on every node. Recreate/distribute this file for every new allocation.
- **HOME is node-local in this cluster:** copy or symlink the job rc file from `/shared/slurm` to each node's local `$HOME` before launching `mana_launch`.
- **Use `mana_status --bcheckpoint` for validation:** this blocks until checkpoint finishes. `srun` task exits after checkpoint request can be non-fatal if checkpoint artifacts exist in `/shared/checkpoints`.
- **Use `--overlap` on validation `srun` steps:** avoids "Requested nodes are busy" during interactive `salloc` sessions.

---

## Implementation Order

1. Step 1 (Rust): inject env-vars into SSM init script in `resource_manager.rs`.
2. Step 2–3 (bash): finalize and test head/worker init scripts locally using the AMI.
3. Step 4 (YAML): write both init_commands blocks into `cluster.example.yaml`.
4. Step 5 (YAML): update `tasks.example.yaml`.
5. Build (`cargo build`) and fix any compile errors.
6. Spawn a real 1-head + 2-worker cluster and walk through validation procedure.
7. Record outcomes in Phase 2 artifact.

---

## What is NOT in Phase 2

- Automatic interruption detection (Phase 3).
- Spot termination handling (Phase 3).
- Recovery policy switch (Phase 4).
- Full benchmark matrix with class C/D (pending AWS credits — Phase 5).
- Scoping `run_task` to head-only for Slurm jobs (nice-to-have; not blocking Phase 2).
