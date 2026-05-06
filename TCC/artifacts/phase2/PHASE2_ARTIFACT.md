# Phase 2 Artifact — Slurm and MANA Installation Flow

Date: 2026-05-06
Project: Integrating MANA Fault Tolerance into HPC@Cloud for AWS Spot Clusters
Repository: hpcac-toolkit

## 1. Purpose
This artifact records the completed Phase 2 implementation: automatic Slurm and MANA
bootstrap on a 1-head + N-worker cluster spawned by HPC@Cloud, and validated end-to-end
MANA checkpoint/restart on multi-node EFS-backed storage.

Phase 2 objective: spawn a real 1-head + 2-worker cluster and achieve:
1. Slurm operational on all nodes with the controller on head and slurmd on workers.
2. MPI workloads running under `srun` across nodes.
3. MANA checkpoint writing to shared EFS storage.
4. MANA restart resuming from those checkpoint artifacts.

---

## 2. Changes Implemented

### 2.1 SSM Init Script Env-Var Injection
- File: `src/integrations/providers/aws/resource_manager.rs`
- At spawn step 19, immediately after fetching per-node `init_commands`, a leading export
  block is prepended so every node's SSM script receives cluster-wide context without
  hardcoding IPs or counts in the YAML.
- Variables injected:

| Variable | Value |
|---|---|
| `HPCAC_NODE_ROLE` | `head` or `worker` |
| `HPCAC_NODE_INDEX` | integer index in provisioning order |
| `HPCAC_HEAD_PRIVATE_IP` | `10.0.0.10` (constant for this subnet) |
| `HPCAC_NODE_COUNT` | total nodes in cluster |
| `HPCAC_WORKER_COUNT` | `node_count - 1` |
| `HPCAC_USE_EFS` | boolean from cluster config |
| `HPCAC_EFS_MOUNT` | `/shared` |

### 2.2 Head Init Script (`cluster.example.yaml`)
The head node's `init_commands` block was written to execute via SSM in this order:
1. Create and set ownership on `/shared/slurm` and `/shared/checkpoints` on EFS.
2. Generate fresh munge key and start munge.
3. Generate `slurm.conf` dynamically using injected env-vars (cluster name, head host/IP,
   CPU count from `nproc`, worker count and IPs from deterministic `10.0.0.N` scheme).
4. Stop any stale `slurmctld` from the AMI, remove stale controller state files, then
   start `slurmctld` and `slurmd` on head.
5. Copy munge key and `slurm.conf` to EFS (`/shared/slurm/`).
6. Write readiness sentinel `/shared/slurm/head_ready` to unblock workers.

### 2.3 Worker Init Script (`cluster.example.yaml`)
Each worker's `init_commands` block:
1. Stops and disables `slurmctld` (present in AMI from Phase 0 single-node setup).
2. Removes stale `clustername` and `.old` state files that would cause CLUSTER NAME MISMATCH.
3. Polls `/shared/slurm/head_ready` for up to 300 seconds before continuing.
4. Copies munge key from EFS and starts munge.
5. Copies `slurm.conf` from EFS.
6. Starts `slurmd` and validates it is active.

### 2.4 EFS Directory Permissions
- `/shared/slurm` and `/shared/checkpoints` are created by the head with
  `chown ec2-user:ec2-user` and `chmod 775` so the runtime user can write checkpoint
  artifacts and MANA job rc files without sudo.

### 2.5 MANA Multi-Node Coordinator Workflow (documented and validated)
The working MANA launch/checkpoint/restart sequence for non-shared HOME clusters:
1. Start coordinator with a fixed port inside the Slurm allocation.
2. Publish the per-job rc file (`$HOME/.mana-slurm-$SLURM_JOB_ID.rc`) to EFS.
3. Distribute the rc file to each node's local `$HOME` using `srun --overlap env RC=... DEST=... /bin/sh -c 'cp ...'`.
4. Export `DMTCP_COORD_HOST` and `DMTCP_COORD_PORT` to all Slurm steps.
5. Launch with `srun --overlap --mpi=pmi2 -N2 -n4 /opt/mana/bin/mana_launch ...`.
6. Trigger checkpoint with `mana_status --bcheckpoint` (blocking variant).
7. Restart with a fresh coordinator and `srun ... mana_restart --restartdir /shared/checkpoints`.

Key constraints discovered and documented:
- `mana_launch` opens `$HOME/.mana-slurm-$SLURM_JOB_ID.rc` before respecting env-var overrides.
  In clusters where `$HOME` is node-local, this file must be copied to every node.
- The coordinator must be started on a fixed port (not the default random `--port 0` behavior
  that worked once but caused `Address already in use` on subsequent runs).
- `mana_status --checkpoint` may cause `srun` tasks to exit with code 99 even when
  checkpoint succeeds — use `find /shared/checkpoints` to confirm artifacts, not exit codes.
- Stale coordinator processes from earlier runs will bind port 7779 and block new starts.
  Always run `sudo pkill -9 -f dmtcp_coordinator` before starting a new coordinator.
- `srun` step creation inside `salloc` may return "Requested nodes are busy" for small
  in-allocation steps. Use `--overlap` to bypass this for admin/setup steps.

---

## 3. Cluster Configuration Used

### 3.1 Instance Profile
- Head: `t3.medium`, on-demand, `burstable_mode: Unlimited`
- Workers: `m5.large` × 2, on-demand (spot used in full-scale profile)
- AMI: `ami-08b9f0fb120be798a` (all nodes)
- Region: `us-east-1`, AZ: `us-east-1a`
- EFS: enabled, mounted at `/shared`

### 3.2 Slurm Configuration Generated at Boot
- `ClusterName=hpcac`
- `SlurmctldHost=ip-10-0-0-10`, `SlurmctldAddr=10.0.0.10`
- `MpiDefault=pmi2` (pmix unavailable on AL2023)
- `TaskPlugin=task/none`, `ProctrackType=proctrack/linuxproc` (cgroup plugin disabled)
- NodeName lines for head + each worker with explicit `CPUs=N` and `NodeAddr=10.0.0.N`
- `PartitionName=hpcac Nodes=ALL Default=YES MaxTime=INFINITE State=UP`

---

## 4. Validation Summary

### 4.1 Head-First Provisioning (Phase 1 carry-over confirmation)
- Result: PASS
- Evidence: head node (node_index 0) appeared first in EC2 spawn logs. Workers polled
  `/shared/slurm/head_ready` and proceeded only after the head wrote it.

### 4.2 Slurm Cluster Health
- Result: PASS
- Evidence:
  - `scontrol show nodes` reported both nodes with `State=IDLE` after spawn.
  - `sinfo -N` showed both nodes in the `hpcac` partition.
  - `sudo systemctl is-active slurmctld slurmd` returned `active active` on head.
  - `sudo systemctl is-active munge slurmd` returned `active active` on worker.

### 4.3 NPB EP.A Under Slurm (multi-node)
- Result: PASS
- Command: `srun --mpi=pmi2 -N2 -n4 ~/downloads/NPB3.4.4/NPB3.4-MPI/bin/ep.A.x`
- Evidence: `Verification SUCCESSFUL`

### 4.4 MANA Checkpoint on Multi-Node Cluster
- Result: PASS
- Test binary: `/opt/mana/mpi-proxy-split/test/mpi_hello_world.mana.exe`
- Allocation: `salloc -N2 -n4 -t 00:30:00`
- Launch command:
  ```
  srun --overlap --mpi=pmi2 -N2 -n4 /opt/mana/bin/mana_launch \
    --ckptdir /shared/checkpoints \
    /opt/mana/mpi-proxy-split/test/mpi_hello_world.mana.exe
  ```
- `mana_status --list` showed exactly 4 clients, all `WorkerState::RUNNING`, across both nodes.
- `mana_status --bcheckpoint` completed successfully.
- Checkpoint artifacts confirmed in `/shared/checkpoints`:
  - `ckpt_rank_0/`, `ckpt_rank_1/`, `ckpt_rank_2/`, `ckpt_rank_3/`
  - Each directory contains `header.mana` and a `ckpt_lower-half_*.dmtcp` file (~28 MB each).
  - `dmtcp_restart_script.sh` symlink present.

### 4.5 EFS Checkpoint Visibility on All Nodes
- Result: PASS
- Evidence: `srun -N2 -n2 --label /bin/sh -c 'ls /shared/checkpoints'` listed the same
  checkpoint directories on both head and worker nodes.

### 4.6 MANA Restart from Checkpoint on Multi-Node Cluster
- Result: PASS
- Command:
  ```
  /opt/mana/bin/mana_coordinator --port 7779 --ckptdir /shared/checkpoints --exit-on-last &
  srun --overlap --mpi=pmi2 -N2 -n4 /opt/mana/bin/mana_restart \
    --restartdir /shared/checkpoints
  ```
- Evidence: all 4 ranks resumed execution from checkpointed state.

---

## 5. Known Constraints and Workarounds

| Constraint | Workaround |
|---|---|
| `$HOME` is node-local (not NFS-shared) | Copy `.mana-slurm-$SLURM_JOB_ID.rc` via EFS to each node before `mana_launch` |
| MANA rc file is per-Slurm-job | Must regenerate and redistribute for every new `salloc` |
| Port 7779 may already be bound from a previous run | Always `sudo pkill -9 -f dmtcp_coordinator` before starting a new coordinator |
| `mana_status --checkpoint` exits srun tasks with code 99 | Not a failure; check artifact presence in `/shared/checkpoints` |
| `srun` step creation inside allocation returns "nodes are busy" | Use `srun --overlap` for admin/setup steps |
| `mana_launch` ignores `DMTCP_COORD_HOST/PORT` env until after rc file is read | rc file must exist on every node before launch |
| `mana_status --list` may show UNKNOWN clients after a failed run | Kill all lower-half, mana_launch, dmtcp_* processes across all nodes and restart |

---

## 6. Files Changed

| File | Type of change |
|---|---|
| `src/integrations/providers/aws/resource_manager.rs` | Inject `HPCAC_*` env-vars into SSM init script per node |
| `cluster.example.yaml` | Add role-specific `init_commands` for head and worker nodes; fix EFS directory ownership |
| `TCC/artifacts/phase2/PHASE2_PLAN.md` | Updated step 6.3 with proven multi-node MANA workflow |
| `TCC/artifacts/phase0/AMI_SETUP_COMMANDS.md` | Added note on node-local HOME rc propagation |

---

## 7. Acceptance Criteria (Checklist)

- [x] `cargo build` passes after Rust env-var injection change.
- [x] Head node: `slurmctld` running and both nodes show `State=IDLE` after spawn.
- [x] Worker node(s): `slurmd` running and registered with head.
- [x] NPB EP.A runs successfully under `srun --mpi=pmi2` across 2 nodes.
- [x] MANA checkpoint creates dirs in `/shared/checkpoints`.
- [x] MANA restart succeeds from `/shared/checkpoints` on multi-node cluster.
- [x] `/shared/checkpoints` is visible on all nodes (EFS consistency confirmed).
- [x] Phase 1 acceptance criterion confirmed: head is node_index 0 in EC2 spawn logs.

---

## 8. Immediate Next Actions (Phase 3 Entry)

1. Implement spot interruption detection agent on each worker node.
   - Poll EC2 instance metadata endpoint for termination notice.
   - Trigger `mana_status --checkpoint` when notice is detected.
2. Automate coordinator lifecycle:
   - Start coordinator as a systemd service on head at cluster spawn time.
   - Keep it running across allocations with a stable port and shared rc via EFS.
3. Integrate node drain in Slurm on interruption:
   - `scontrol update NodeName=<host> State=DRAIN Reason=spot-interruption`
4. Implement replacement node respawn workflow in HPC@Cloud.
5. Validate full: interrupt → checkpoint → drain → respawn → restart cycle.

---

## 9. File References
- Phase 2 plan: `TCC/artifacts/phase2/PHASE2_PLAN.md`
- Phase 1 artifact: `TCC/artifacts/phase1/PHASE1_ARTIFACT.md`
- Phase 0 artifact: `TCC/artifacts/phase0/PHASE0_BASELINE_ARTIFACT.md`
- Project plan: `TCC/TCC_MANA_INTEGRATION_PLAN.md`
