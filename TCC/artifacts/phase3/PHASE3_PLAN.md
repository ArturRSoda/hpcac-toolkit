# Phase 3 Plan - Interruption-Aware Runtime and Recovery (As Implemented)

Goal: document the real Phase 3 implementation currently in this repository, not the original intended design.

This rewrite is source-of-truth aligned with:
- [src/commands/cluster/run_task.rs](src/commands/cluster/run_task.rs)
- [src/commands/cluster/watcher.rs](src/commands/cluster/watcher.rs)
- [src/integrations/cloud_interface.rs](src/integrations/cloud_interface.rs)
- [src/integrations/providers/aws/resource_manager.rs](src/integrations/providers/aws/resource_manager.rs)
- [src/integrations/providers/aws/resources/elastic_compute.rs](src/integrations/providers/aws/resources/elastic_compute.rs)
- [src/integrations/providers/aws/resources/services_system_manager_command.rs](src/integrations/providers/aws/resources/services_system_manager_command.rs)
- [src/database/models/interruption_event.rs](src/database/models/interruption_event.rs)
- [migrations/9__add_interruption_events_table.sql](migrations/9__add_interruption_events_table.sql)

Depends on:
- Phase 1: deterministic node identity and role model
- Phase 2: Slurm + MANA environment bootstrapped through spawn and init_commands

---

## 1. Architecture Actually Implemented

### 1.1 Runtime placement

Phase 3 recovery logic runs in the local CLI process during:

- `cargo run -- cluster run-task --cluster-id <id> --file <tasks.yaml>`

There is no persistent daemon started by `cluster spawn`. The watcher lifecycle is bounded to `run-task` execution.

### 1.2 High-level control flow

1. `run_task` parses `tasks.yaml`.
2. If `fault_tolerance.strategy` is not `NONE`, it spawns watcher task (`tokio::spawn`).
3. `run_task` executes task commands via SSM on head node.
4. Watcher polls AWS interruption signals and local interruption events.
5. On interruption, watcher runs policy-specific recovery (`REPLACE_RESUME` or `DEGRADED_RESUME`).
6. Watcher writes interruption/recovery events to local SQLite.
7. `run_task` uses those events to decide whether failed command should be treated as recovered and skipped.
8. On task completion, watcher is aborted.

### 1.3 Coordination channel

Primary synchronization channel is `interruption_events` DB table.

`run_task` does not call recovery logic directly. It waits for watcher terminal events and interprets them.

---

## 2. Scope and Feature Inventory

| Area | File(s) | Implemented in current code |
|---|---|---|
| Fault tolerance YAML parsing in run-task | [src/commands/cluster/run_task.rs](src/commands/cluster/run_task.rs) | `fault_tolerance` block parsing, validation, watcher spawn, recovery-aware command progression |
| Watcher module | [src/commands/cluster/watcher.rs](src/commands/cluster/watcher.rs), [src/commands/cluster/mod.rs](src/commands/cluster/mod.rs) | Full polling, queueing, event write model, REPLACE/DEGRADED handlers, background restart monitor |
| DB event persistence | [migrations/9__add_interruption_events_table.sql](migrations/9__add_interruption_events_table.sql), [src/database/models/interruption_event.rs](src/database/models/interruption_event.rs), [src/database/models/mod.rs](src/database/models/mod.rs) | Event schema + insert/fetch model |
| Provider abstraction extensions | [src/integrations/cloud_interface.rs](src/integrations/cloud_interface.rs) | `simulate_cluster_failure` warning arg, `respawn_worker_node`, provider dispatch |
| AWS interruption and recovery primitives | [src/integrations/providers/aws/resource_manager.rs](src/integrations/providers/aws/resource_manager.rs), [src/integrations/providers/aws/resources/elastic_compute.rs](src/integrations/providers/aws/resources/elastic_compute.rs) | warning-time simulation, `fetch_spot_instance_status`, worker respawn routine |
| SSM execution robustness | [src/integrations/providers/aws/resources/services_system_manager_command.rs](src/integrations/providers/aws/resources/services_system_manager_command.rs) | stricter shell wrapper (`set -euo pipefail`) and richer error propagation |
| FT test command argument | [src/main.rs](src/main.rs), [src/commands/cluster/test_failure.rs](src/commands/cluster/test_failure.rs) | `--warning-time` parameter wired through to cloud simulation |

Notes:
- The codebase also now includes `cluster restore`. That command is operationally useful but is outside the original Phase 3 watcher/run-task fault-tolerance loop.

---

## 3. Data Model and Event Contract

### 3.1 Migration

File: [migrations/9__add_interruption_events_table.sql](migrations/9__add_interruption_events_table.sql)

```sql
CREATE TABLE interruption_events (
    id VARCHAR(32) PRIMARY KEY,
    cluster_id VARCHAR(32) NOT NULL,
    node_id VARCHAR(32) NOT NULL,
    node_private_ip TEXT NOT NULL,
    event_type TEXT NOT NULL,
    occurred_at TEXT NOT NULL,
    details TEXT,
    FOREIGN KEY (cluster_id) REFERENCES clusters(id),
    FOREIGN KEY (node_id) REFERENCES nodes(id)
);
```

`occurred_at` is `TEXT` and stored as RFC3339 (`Utc::now().to_rfc3339()`).

### 3.2 Model

File: [src/database/models/interruption_event.rs](src/database/models/interruption_event.rs)

`InterruptionEvent` provides:
- constructor `new(...)`
- insert method `insert(...)`
- debug fetch helper `fetch_all_by_cluster_id(...)`

### 3.3 Event types used in current implementation

The runtime currently uses this event taxonomy:

- `interruption_warning`
- `interruption_detected`
- `interruption_termination_requested`
- `checkpoint_dir_cleared`
- `checkpoint_completed`
- `node_drained`
- `recovery_started`
- `restart_completed`
- `recovery_completed`
- `degraded_resume`
- `no_workers_remaining`
- `recovery_failed`

`details` commonly carries `cycle_id=<id>` plus contextual metadata (restart output, counts, reason).

---

## 4. Tasks YAML Contract (As Implemented)

### 4.1 Schema in `run_task`

File: [src/commands/cluster/run_task.rs](src/commands/cluster/run_task.rs)

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
struct FaultToleranceConfig {
    strategy: String,
    #[serde(default = "default_replacement_allocation_mode")]
    replacement_allocation_mode: String,
    process_count: u32,
    checkpoint_dir: String,
    #[serde(default = "default_poll_interval")]
    poll_interval_secs: u64,
    #[serde(default)]
    terminate_cluster_on_zero_workers: bool,
}
```

Top-level tasks file:

```yaml
fault_tolerance:
  strategy: "NONE" | "REPLACE_RESUME" | "DEGRADED_RESUME"
  replacement_allocation_mode: "spot" | "on-demand"
  process_count: <u32>
  checkpoint_dir: "/shared/checkpoints"
  poll_interval_secs: <u64>
  terminate_cluster_on_zero_workers: <bool>

tasks:
  - task_tag: "..."
    setup_commands: [...]
    run_commands: [...]
```

### 4.2 Validation

Current guards in `run_task`:

- `strategy` must be one of `NONE`, `REPLACE_RESUME`, `DEGRADED_RESUME`.
- For `REPLACE_RESUME`, `replacement_allocation_mode` must be `spot` or `on-demand`.

### 4.3 Operational behavior by strategy

- Fault tolerance block absent: no watcher
- `strategy=NONE`: no watcher
- `strategy=REPLACE_RESUME`: watcher active
- `strategy=DEGRADED_RESUME`: watcher active

---

## 5. `run_task` Integration Details

File: [src/commands/cluster/run_task.rs](src/commands/cluster/run_task.rs)

### 5.1 Preflight topology extraction

`run_task` derives node indexes from deterministic private IP:

- `10.0.0.10 -> index 0` (head)
- `10.0.0.11 -> index 1`
- `10.0.0.12 -> index 2`
- generic formula: last_octet - 10

Validations:

- exactly one head node
- head maps to index 0

### 5.2 Worker instance ID map

When FT is active, `run_task` resolves running EC2 instance IDs for workers via:

- private IP filter
- running-state filter
- cluster tag filter

Missing worker instance is tolerated with warning because watcher preflight can recover missing slots.

### 5.3 Watcher spawn contract

Watcher is spawned with:

- cloned pool and cluster
- full node list
- `(node_index, node)` worker vector
- worker `index -> instance_id` map
- head instance ID
- dedicated AWS interface/context clones
- FT config
- optional progress bar handle

### 5.4 Command execution semantics under FT

For each setup/run command:

1. Send SSM command to head.
2. If command succeeds: continue.
3. If command fails and FT inactive: fail fast.
4. If command fails and FT active:
   - wait for watcher recovery terminal event(s)
   - if recovery succeeds: mark failed command as recovered and continue to next command
   - if only `recovery_failed` appears and no later success before timeout: fail

Current behavior intentionally does not rerun the failed command. It advances after successful recovery terminal event.

### 5.5 Recovery output ingestion

`run_task` parses `restart_completed` details for markers:

- `--restart_output_begin--`
- `--restart_output_end--`

and writes watcher restart stdout/stderr excerpts into the task report.

### 5.6 End-of-run FT summary

If FT active, `run_task` appends a recovery summary block to report file:

- event counts by event type
- cycle IDs
- chronological recovery timeline

---

## 6. Watcher Runtime Design

File: [src/commands/cluster/watcher.rs](src/commands/cluster/watcher.rs)

### 6.1 Concurrency model

`run_watcher` is a long-lived async loop with serialized interruption processing.

State used in loop:

- `in_flight: HashSet<usize>`
- `queued: HashSet<usize>`
- `pending: VecDeque<usize>`
- `missing_running_instance_counts: HashMap<usize, u8>`
- `consumed_warning_event_ids: HashSet<String>`

### 6.2 Detection channels

Watcher enqueues recoveries from two channels:

1. Simulated warning events in DB (`interruption_warning` at/after watcher start time)
2. AWS spot status polling (`fetch_spot_instance_status`) for each worker instance

### 6.3 AWS status mapping used by watcher

Watcher treats these as interruption-triggering statuses:

- `marked-for-termination`
- `instance-terminated-by-user`
- `instance-terminated-no-capacity`
- `instance-terminated-by-price`
- `instance-terminated-capacity-oversubscribed`
- `closed`

### 6.4 Missing-instance preflight and threshold

If a worker has no running instance for 3 consecutive polls, watcher queues recovery for that worker index.

This is the practical preflight reconciliation path for missed idle-time failures.

### 6.5 Serialization guarantee

Only one recovery flow runs at a time. Queue processing pops one worker index and handles entire recovery before next queued interruption.

---

## 7. Interruption Handling Pipeline (Common Path)

Function: `handle_interruption(...)` in [src/commands/cluster/watcher.rs](src/commands/cluster/watcher.rs)

Common steps:

1. Insert `interruption_detected` event.
2. Clear checkpoint directory (new-cycle hygiene):
   - SSM command to head: `find ... -exec rm -rf`
   - insert `checkpoint_dir_cleared` event.
3. Trigger MANA checkpoint:
   - command: `dmtcp_command --bcheckpoint` with coordinator env
   - on success: insert `checkpoint_completed`
   - on failure: warning only, continue.
4. Drain interrupted worker in Slurm:
   - `scontrol update NodeName=<worker_host> State=DRAIN ...`
   - insert `node_drained`.
5. Dispatch policy-specific handler based on strategy.

---

## 8. Policy A - `REPLACE_RESUME` (As Implemented)

Function: `handle_policy_replace_resume(...)` in [src/commands/cluster/watcher.rs](src/commands/cluster/watcher.rs)

### 8.1 Steps

1. Insert `recovery_started` with `cycle_id`.
2. Wait for interrupted instance termination (`wait_for_worker_termination`).
3. Reset EFS configured flag in DB (`node.set_efs_configuration_state(false)`).
4. Call provider respawn:
   - `respawn_worker_node(...)`
   - mode override from `replacement_allocation_mode`.
5. Resume node in Slurm:
   - `scontrol update NodeName=<worker_host> State=RESUME`.
6. Wait schedulable Slurm state:
   - accepts `idle*`, `mix*`, `alloc*`
   - rejects early for `down*`, `drain*`, `drng*`, `fail*`, `unk*`, `maint*`.
7. Build and dispatch restart command to head.
8. Do not block main watcher loop on restart completion:
   - spawn background monitor (`spawn_restart_monitor`) to poll SSM completion.

### 8.2 Terminal events

On background monitor success:

- insert `restart_completed` (with restart output details block)
- insert `recovery_completed`

On monitor failure:

- insert `recovery_failed`

---

## 9. Policy B - `DEGRADED_RESUME` (As Implemented)

Function: `handle_policy_degraded_resume(...)` in [src/commands/cluster/watcher.rs](src/commands/cluster/watcher.rs)

### 9.1 Steps

1. Insert `recovery_started` with `cycle_id`.
2. Compute `remaining_workers = total_workers - 1`.
3. If zero workers remain:
   - issue quit/stop commands (`dmtcp_command --quit`, `scancel`)
   - insert `no_workers_remaining`
   - optionally call `terminate_cluster` if configured
   - return.
4. Mark failed node `DOWN` in Slurm.
5. Build degraded restart command for surviving workers only.
6. Dispatch restart command and monitor in background.

### 9.2 Terminal events

On background monitor success:

- insert `restart_completed`
- insert `degraded_resume`

On monitor failure:

- insert `recovery_failed`

---

## 10. Restart Command Builder Details

Function: `build_restart_command(...)` in [src/commands/cluster/watcher.rs](src/commands/cluster/watcher.rs)

### 10.1 Behavior

Generated command writes `/tmp/hpcac_restart.sh`, then runs through `salloc`.

Script currently does:

- kill stale coordinator/processes
- export DMTCP coordinator env
- clean old lower-half/mana/dmtcp processes via `srun`
- recreate coordinator and rc distribution via EFS
- execute `mana_restart`

### 10.2 Degraded flags

When `oversubscribe=true` (degraded mode):

- adds `--oversubscribe --overcommit` to both `srun` and `salloc` restart envelope.

### 10.3 Output capture

Wrapper pipes command output through `tee restart_output.txt`, and monitor stores command output payload in `restart_completed.details` markers.

---

## 11. AWS Provider Changes Underpinning Phase 3

### 11.1 Spot status helper

File: [src/integrations/providers/aws/resources/elastic_compute.rs](src/integrations/providers/aws/resources/elastic_compute.rs)

`fetch_spot_instance_status(...)`:

- returns AWS spot status code for instance
- returns `not-spot` when no spot request exists (head/on-demand)
- returns `unknown` fallback when status code missing

### 11.2 Failure simulation with warning window

Files:
- [src/main.rs](src/main.rs)
- [src/commands/cluster/test_failure.rs](src/commands/cluster/test_failure.rs)
- [src/integrations/providers/aws/resource_manager.rs](src/integrations/providers/aws/resource_manager.rs)

`cluster test-failure` accepts:

- `--warning-time <seconds>`

When warning time > 0, simulation inserts `interruption_warning` event, sleeps, then terminates instance and inserts `interruption_termination_requested`.

### 11.3 Worker respawn API and implementation

Files:
- [src/integrations/cloud_interface.rs](src/integrations/cloud_interface.rs)
- [src/integrations/providers/aws/resource_manager.rs](src/integrations/providers/aws/resource_manager.rs)
- [src/integrations/providers/vultr/resource_manager.rs](src/integrations/providers/vultr/resource_manager.rs)

`respawn_worker_node(...)` flow in AWS:

- re-create/find baseline resources (vpc/subnet/sg/iam/ssh/efs)
- ensure ENI/EIP for worker index
- launch replacement instance
- wait running + SSM readiness
- reapply EFS and node init commands
- optionally persist replacement allocation mode (`nodes.allocation_mode`)

Vultr keeps compile-safe stubs (`Not implemented`).

### 11.4 SSM command hardening

File: [src/integrations/providers/aws/resources/services_system_manager_command.rs](src/integrations/providers/aws/resources/services_system_manager_command.rs)

Changes in current code:

- wrapped script uses `set -euo pipefail` inside ec2-user shell
- failure paths return detailed errors with stdout/stderr content instead of direct `println!` only

---

## 12. Additional Supporting Changes

### 12.1 Clone requirements

Files:
- [src/database/models/provider_config.rs](src/database/models/provider_config.rs)
- [src/integrations/providers/aws/interface.rs](src/integrations/providers/aws/interface.rs)

Current code derives `Clone` for:

- `ConfigVar`
- `AwsClusterContext`
- `AwsInterface`

This enables watcher/background monitor ownership patterns in async tasks.

### 12.2 Watcher module export

File: [src/commands/cluster/mod.rs](src/commands/cluster/mod.rs)

`pub mod watcher;` is present and wired.

---

## 13. Validation Procedure (Updated to Current Behavior)

### Gate A - Build and static checks

```bash
cargo build
```

Pass:
- builds successfully

### Gate B - FT config validation

```bash
cargo run -- cluster run-task --cluster-id <id> --file tasks-bad.yaml -y
cargo run -- cluster run-task --cluster-id <id> --file tasks-bad-replace-mode.yaml -y
```

Pass:
- invalid strategy rejected
- invalid replacement mode rejected

### Gate C - No-FT regressions

```bash
cargo run -- cluster run-task --cluster-id <id> --file tasks-no-ft.yaml -y
cargo run -- cluster run-task --cluster-id <id> --file tasks-ft-none.yaml -y
```

Pass:
- no watcher startup in both cases

### Gate D - FT startup and worker index mapping

```bash
cargo run -- cluster run-task --cluster-id <id> --file tasks-ft-replace.yaml -y
```

Pass:
- FT block printed
- worker mapping printed (`10.0.0.11 -> 1`, `10.0.0.12 -> 2`)
- watcher startup line printed

### Gate E - Warning-time simulation

```bash
cargo run -- cluster test-failure --cluster-id <id> --node-private-ip 10.0.0.11 --warning-time 120 -y
```

Pass:
- warning event generated before termination
- watcher can consume warning path and queue recovery

### Gate F - REPLACE flow end-to-end

1. Start `run-task` with `REPLACE_RESUME`
2. trigger failure on worker
3. verify sequence in logs and DB:
   - `interruption_detected`
   - `checkpoint_dir_cleared`
   - `node_drained`
   - `recovery_started`
   - `restart_completed`
   - `recovery_completed` (or `recovery_failed`)

### Gate G - DEGRADED flow end-to-end

1. Start `run-task` with `DEGRADED_RESUME`
2. trigger failure on worker
3. verify:
   - worker set `DOWN`
   - restart dispatched with oversubscribe/overcommit
   - terminal event `degraded_resume` or `recovery_failed`

### Gate H - run_task recovery-aware command progression

Force command failure during active interruption and verify:

- `run_task` waits for recovery terminal events
- on successful recovery terminal event, failed command is treated as recovered and skipped
- restart output is written to report

### Gate I - multi-cycle resilience

During recovery window, trigger another interruption/failure.

Pass:
- watcher continues running
- failures can emit `recovery_failed` for one cycle and still handle later cycles

### Gate J - preflight/missing-worker reconciliation

Start `run-task` when one worker is already missing.

Pass:
- watcher queues recovery after missing-instance threshold and repairs topology during same run-task session

---

## 14. Acceptance Criteria (Current Code)

- [x] Build passes (`cargo build`).
- [x] FT schema validation rejects invalid strategy/mode.
- [x] `strategy=NONE` and absent FT block run without watcher.
- [x] Watcher starts only for `REPLACE_RESUME` and `DEGRADED_RESUME`.
- [x] Worker index mapping is deterministic from private IP.
- [x] `test-failure --warning-time` generates warning event prior to termination.
- [x] Common interruption path records detect/checkpoint/drain events.
- [x] `REPLACE_RESUME` calls respawn and reaches restart monitoring path.
- [x] `DEGRADED_RESUME` marks node down and restarts with oversubscribe flags.
- [x] Restart completion output is persisted and surfaced in reports.
- [x] `recovery_failed` is treated as retryable by `run_task` waiting logic.
- [x] Watcher remains alive after failed recovery cycle.
- [x] FT report section includes event counts, cycle IDs, and timeline.
- [N/A] Gate J (preflight/missing-worker reconciliation) is out of Phase 3 FT acceptance scope for TCC; topology repair outside active MANA wrapper context is covered by `cluster restore`.

---

## 15. Implementation Order (Retroactive, Matching Current Tree)

1. Add interruption events migration and model.
2. Add spot status helper and warning-time simulation pathway.
3. Extend cloud resource manager abstraction (`respawn_worker_node`, warning-time arg).
4. Implement AWS worker respawn path.
5. Add watcher module and export it in cluster command module.
6. Integrate FT parser/spawn/wait semantics in `run_task`.
7. Harden SSM wrapper and error reporting for clearer debugging.
8. Add run/report summaries and cycle-based event parsing.
9. Validate REPLACE and DEGRADED through end-to-end runtime tests.

---

## 16. Out-of-Scope for Phase 3

- Permanent background watcher daemon independent of `run-task` lifecycle.
- Full autonomous control-plane service for recovery while operator is offline.
- Multi-provider implementation parity (Vultr remains stubbed for this surface).
- Cost/performance analysis framework itself (Phase 3 provides runtime mechanism; analysis workflow is separate).

---

## 17. Practical Notes for TCC Execution

- Keep `run-task` process alive during experiments; watcher lives inside it.
- Prefer explicit FT tasks files per strategy (`tasks-ft-replace.yaml`, `tasks-ft-degraded.yaml`).
- Use `--warning-time` in `test-failure` for deterministic warning-window tests.
- Correlate timeline by `cycle_id` in report and DB records.
- If a cycle emits `recovery_failed`, do not assume run ended; later successful cycles may still complete the run.
