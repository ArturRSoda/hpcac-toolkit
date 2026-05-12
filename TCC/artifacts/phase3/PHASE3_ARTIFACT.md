# Phase 3 Artifact - Interruption-Aware Runtime Recovery

Date: 2026-05-12
Project: Integrating MANA Fault Tolerance into HPC@Cloud for AWS Spot Clusters
Repository: hpcac-toolkit

## 1. Purpose
This artifact records the completed Phase 3 implementation: runtime interruption handling
for active workloads executed by `cluster run-task`, with policy-based recovery for
`REPLACE_RESUME` and `DEGRADED_RESUME`.

Phase 3 objective: validate end-to-end interruption handling during active MANA wrapper
execution, including checkpoint, Slurm node state management, restart orchestration,
recovery-aware command progression, and event-based reporting.

---

## 2. Changes Implemented

### 2.1 Runtime watcher integrated into `run-task`
- File: `src/commands/cluster/run_task.rs`
- `fault_tolerance` block parsing and validation integrated into task execution.
- Watcher is spawned only when `strategy != NONE`.
- Worker mapping uses deterministic private-IP-to-index mapping.
- Failed run command under FT waits for watcher terminal recovery events.
- Recovery output is parsed from `restart_completed.details` and included in report.

### 2.2 Watcher module and policy handlers
- Files: `src/commands/cluster/watcher.rs`, `src/commands/cluster/mod.rs`
- Added watcher loop with queued/serialized interruption handling.
- Implemented common interruption pipeline:
  1. `interruption_detected`
  2. checkpoint directory clear
  3. MANA bcheckpoint
  4. Slurm drain
- Implemented policy handlers:
  - `REPLACE_RESUME`: wait terminate -> respawn worker -> Slurm resume -> restart monitor.
  - `DEGRADED_RESUME`: mark failed node down -> restart with oversubscribe path.
- Added background restart monitor and terminal event emission.

### 2.3 Interruption event persistence model
- Files: `migrations/9__add_interruption_events_table.sql`, `src/database/models/interruption_event.rs`, `src/database/models/mod.rs`
- Added `interruption_events` table and model APIs (`new`, `insert`, cluster fetch).
- Event stream used for run-task/watcher synchronization and final report timeline.

### 2.4 Provider abstraction and AWS runtime primitives
- Files: `src/integrations/cloud_interface.rs`, `src/integrations/providers/aws/resource_manager.rs`, `src/integrations/providers/aws/resources/elastic_compute.rs`
- Extended cloud trait with FT support methods (`respawn_worker_node`, warning-aware failure simulation).
- Added AWS spot status helper for interruption detection.
- Implemented AWS worker respawn flow used by replace policy.

### 2.5 SSM execution hardening for recovery diagnostics
- File: `src/integrations/providers/aws/resources/services_system_manager_command.rs`
- Shell wrapper now uses strict mode (`set -euo pipefail`).
- Failure paths return detailed stdout/stderr payloads for report/debug visibility.

### 2.6 FT simulation and operations support commands
- Files: `src/main.rs`, `src/commands/cluster/test_failure.rs`
- Added `--warning-time` for simulated interruption warning windows.
- `cluster restore` command implemented and validated as post-failure topology repair path
  outside active MANA wrapper runtime.

---

## 3. Validation Summary (Passed)

### 3.1 Build and config validation gates
- Result: PASS
- Evidence:
  - `cargo build` successful.
  - Invalid FT strategy rejected (`tasks-bad.yaml`).
  - Invalid replace mode rejected (`tasks-bad-replace-mode.yaml`).

### 3.2 No-FT regression gates
- Result: PASS
- Evidence:
  - `tasks-no-ft.yaml` completed without watcher startup.
  - `tasks-ft-none.yaml` (`strategy: NONE`) completed without watcher startup.

### 3.3 REPLACE_RESUME recovery gate
- Result: PASS
- Evidence:
  - watcher startup and deterministic worker mapping logged (`10.0.0.11 -> 1`, `10.0.0.12 -> 2`).
  - interruption warning and termination-requested events observed.
  - recovery pipeline observed through detect/checkpoint/drain/respawn/restart.
  - `run-task` treated failed command as recovered and completed all tasks.

### 3.4 DEGRADED_RESUME recovery gate
- Result: PASS
- Evidence:
  - terminal event `degraded_resume` observed.
  - restart output persisted and printed in final report.
  - report details include oversubscribe=true path and successful completion.

### 3.5 Multi-cycle resilience and failure-path liveness gate
- Result: PASS
- Evidence:
  - dual-failure REPLACE run triggered two interruption cycles (one per worker).
  - first cycle restart monitor observed failure (`recovery_failed` behavior path).
  - watcher remained alive and processed second cycle to successful completion.
  - run still completed all task commands.

### 3.6 Restore command operational coverage (out-of-wrapper)
- Result: PASS
- Command: `cargo run cluster restore --cluster-id ClusterMANA_OREGON`
- Evidence:
  - cluster checked against 3 slots.
  - 1 worker respawned.
  - summary reported `2 healthy, 1 restored`.

---

## 4. Known Scope Boundaries and Constraints

| Topic | Status |
|---|---|
| Watcher lifecycle | Bound to `cluster run-task` process (no daemon). |
| MANA recovery applicability | Recovery is valid during active MANA wrapper context. |
| Preflight missing-worker reconciliation as FT acceptance gate | Not required for TCC Phase 3 acceptance. |
| Out-of-wrapper topology repair | Covered by `cluster restore`. |
| Multi-provider parity | AWS implemented; Vultr remains stubbed for these FT methods. |

---

## 5. Files Changed in Phase 3

| File | Type of change |
|---|---|
| `src/commands/cluster/run_task.rs` | FT parsing/validation, watcher spawn, recovery-aware progression, report integration |
| `src/commands/cluster/watcher.rs` | New watcher runtime, queueing, policy handlers, restart monitor |
| `src/commands/cluster/mod.rs` | Export watcher module |
| `migrations/9__add_interruption_events_table.sql` | New migration |
| `src/database/models/interruption_event.rs` | New event model |
| `src/database/models/mod.rs` | Model export update |
| `src/integrations/cloud_interface.rs` | FT trait surface extensions |
| `src/integrations/providers/aws/resource_manager.rs` | Respawn/recovery/simulation implementation |
| `src/integrations/providers/aws/resources/elastic_compute.rs` | Spot interruption status helper |
| `src/integrations/providers/aws/resources/services_system_manager_command.rs` | Strict shell and richer failure propagation |
| `src/commands/cluster/test_failure.rs` | Warning-time simulation wiring |
| `src/main.rs` | CLI args and command dispatch updates |
| `src/commands/cluster/restore.rs` | New restore command |
| `TCC/artifacts/phase3/PHASE3_PLAN.md` | Implementation-aligned plan and validation matrix |

---

## 6. Acceptance Criteria (Checklist)

- [x] `cargo build` passes.
- [x] FT schema validation rejects invalid strategy and replacement mode.
- [x] `strategy=NONE` and absent FT block run without watcher.
- [x] Watcher starts for `REPLACE_RESUME` and `DEGRADED_RESUME`.
- [x] Worker index mapping is deterministic from private IP.
- [x] Warning-time simulation emits warning before termination-requested event.
- [x] Common interruption path records detect/checkpoint/drain events.
- [x] `REPLACE_RESUME` executes respawn and restart monitoring path.
- [x] `DEGRADED_RESUME` executes degraded restart path and emits terminal policy event.
- [x] Restart output is persisted and surfaced in result report.
- [x] `recovery_failed` path is tolerated and later recovery can still complete.
- [x] Watcher remains alive after a failed recovery cycle and processes subsequent cycles.
- [x] FT report includes event counts, cycle IDs, and timeline.
- [N/A] Gate J preflight missing-worker reconciliation is out of Phase 3 FT acceptance scope for TCC; post-failure slot repair outside active MANA wrapper is covered by `cluster restore`.

---

## 7. Immediate Next Actions (Post-Phase 3)

1. Build a repeatable benchmark protocol for cost/performance comparison:
   on-demand baseline vs spot + Phase 3 FT policies.
2. Add optional structured cycle summary export (JSON/CSV) for analysis automation.
3. Expand failure-injection scenarios (timing and node combinations) to stress test policy behavior.
4. Decide whether a future phase should add daemonized/offline recovery orchestration.

---

## 8. File References
- Phase 3 plan: `TCC/artifacts/phase3/PHASE3_PLAN.md`
- Phase 2 artifact: `TCC/artifacts/phase2/PHASE2_ARTIFACT.md`
- Phase 1 artifact: `TCC/artifacts/phase1/PHASE1_ARTIFACT.md`
- Phase 0 artifact: `TCC/artifacts/phase0/PHASE0_BASELINE_ARTIFACT.md`
- Project context: `TCC/TCC_MANA_INTEGRATION_PLAN.md`
