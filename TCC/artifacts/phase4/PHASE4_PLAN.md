# Phase 4 Plan - Evaluation and Metrics Consolidation

Date: 2026-05-13  
Project: Integrating MANA Fault Tolerance into HPC@Cloud for AWS Spot Clusters

Goal: define and execute a reproducible evaluation campaign for runtime fault-tolerance policies already implemented in Phase 3, producing TCC-quality metrics, comparisons, and conclusions.

Depends on:
- Phase 0: frozen software stack and AMI reproducibility baseline
- Phase 1: role-aware topology (1 head + N workers)
- Phase 2: Slurm + MANA bootstrap and execution flow
- Phase 3: interruption-aware runtime recovery (NONE, REPLACE_RESUME, DEGRADED_RESUME)

---

## 1. Phase 4 Scope (Reframed)

Phase 4 is not focused on adding new recovery policies (already covered by Phase 3 implementation in this repository).  
Phase 4 is focused on:
- Experimental design finalization
- Execution standardization and reproducibility
- Metrics capture and consolidation
- Comparative analysis across policies, workloads, and cluster sizes

Primary deliverable: a clean dataset and analysis-ready summary proving overhead, recovery behavior, and cost trade-offs.

---

## 2. Experimental Design (Locked for Start)

### 2.1 Cluster profile
- Head node: on-demand t3.2xlarge (burstable)
- Worker nodes: spot m5.4xlarge
- Topology: exactly 1 head + {2, 4, 8} workers
- 16-worker scenarios are out of initial scope unless later needed

### 2.2 Benchmarks and classes
- NPB MPI benchmarks:
  - EP (low communication)
  - CG (memory bound, higher communication)
  - LU (CPU/communication mix)
- Initial class target: C
- Runtime target: around 10 minutes per workload
- Rule: if runtime is too short/long, adjust NPB class before full matrix execution

### 2.3 Recovery strategies
- NONE
- REPLACE_RESUME
- DEGRADED_RESUME

### 2.4 Failure scenarios
- S0: no failure (mechanism overhead baseline)
- S1: one injected worker interruption
- S2: two sequential interruptions on different workers

### 2.5 Repetitions
- 3 repetitions per scenario point

### 2.6 Matrix size
- Benchmarks: 3
- Cluster sizes: 3
- Strategies: 3
- Failure scenarios: 3
- Repetitions: 3

Total planned runs: 3 x 3 x 3 x 3 x 3 = 243

---

## 3. Metrics Contract

### 3.1 Core timing metrics
- Total application completion time
- Run start -> warning/detection interval
- Warning/detection -> checkpoint completed interval
- Checkpoint completed -> restart dispatch interval
- Restart dispatch -> restart completed interval
- MANA/FT overhead vs NONE in no-failure scenario
- Additional time caused by recovery in S1 and S2
- Reprovision duration (REPLACE_RESUME only)

### 3.2 Event-derived timing windows
From interruption event timeline:
- Warning (or detection) -> checkpoint completed
- Checkpoint completed -> restart initiated
- Restart initiated -> restart completed

### 3.3 Cost metric
- Approximate run cost computed from elapsed resource usage time
- Approximate cost per recovery interval (same interval partition used by timing metrics)
- Cost is estimated from node type, node count, and run duration (no deep billing integration required)

### 3.4 Out of scope metrics
- Checkpoint file size tracking (optional, not required for acceptance)
- Dedicated interruption-warning-latency instrumentation beyond existing event timestamps

---

## 4. Data and Output Standardization

### 4.1 Canonical run identifier
Each run must have a unique run_id encoding:
- benchmark
- class
- worker_count
- strategy
- failure_scenario
- repetition_index
- timestamp

### 4.2 Mandatory artifacts per run
- Raw command/report output from run-task
- Interruption event timeline entries used in that run
- Aggregated metric row for analysis table
- Execution status (success, recovered, invalid, aborted)

### 4.3 Consolidated dataset format
Produce one flat table (CSV is sufficient) with at least:
- run_id
- cluster_id
- region
- task_file
- benchmark
- class
- workers
- strategy
- scenario
- repetition
- run_started_at
- run_finished_at
- success_flag
- total_time_sec
- t_start_to_warning_sec
- t_warning_to_checkpoint_sec
- t_checkpoint_to_restart_dispatch_sec
- t_restart_dispatch_to_restart_completed_sec
- reprovision_time_sec
- observed_failure_count
- recovery_cycle_count
- approx_cost_start_to_warning
- approx_cost_warning_to_checkpoint
- approx_cost_checkpoint_to_restart_dispatch
- approx_cost_restart_dispatch_to_restart_completed
- approx_cost_reprovision
- approx_cost
- notes

For multi-failure runs, all interval-based time/cost fields above are aggregated as the sum across all observed recovery cycles in the run. This keeps one row per run while preserving fair S1 vs S2 comparisons.

### 4.4 Analysis-only derived metrics (not stored as base columns)
Use these during analysis to compare multi-failure behavior without expanding the base table.

- Per-failure time normalization:

$$
t_{interval\_per\_failure} = \frac{t_{interval\_sum}}{\max(1, observed\_failure\_count)}
$$

- Per-failure cost normalization:

$$
c_{interval\_per\_failure} = \frac{c_{interval\_sum}}{\max(1, observed\_failure\_count)}
$$

- Marginal impact of second failure (S2 vs S1, same benchmark/class/workers/strategy):

$$
\Delta t_{S2-S1} = t_{S2} - t_{S1}
$$

$$
\Delta c_{S2-S1} = c_{S2} - c_{S1}
$$

Metric field definitions:

| Field | Description (what it represents) |
|---|---|
| `run_id` | Unique identifier of one experimental execution point, including workload/size/strategy/scenario/repetition/time. |
| `cluster_id` | Cluster identifier used in this run (`ClusterMANA`, `ClusterMANA_OREGON`, etc.). |
| `region` | AWS region where the run was executed (for example `us-east-1`). |
| `task_file` | Task YAML used for execution (important for reproducibility and auditing command differences). |
| `benchmark` | NPB workload executed in the run (`EP`, `CG`, `LU`). |
| `class` | NPB problem class used for this run (for example `B`, `C`, `D`). |
| `workers` | Number of worker nodes actively allocated for the run at start. |
| `strategy` | Fault-tolerance strategy configured for the run (`NONE`, `REPLACE_RESUME`, `DEGRADED_RESUME`). |
| `scenario` | Failure scenario label (`S0`, `S1`, `S2`) used to define interruption injections. |
| `repetition` | Repetition index for the same scenario point (for statistical aggregation). |
| `run_started_at` | Wall-clock timestamp when the run execution starts. |
| `run_finished_at` | Wall-clock timestamp when the run execution ends. |
| `success_flag` | Final run status (completed successfully vs invalid/aborted/failed). |
| `total_time_sec` | End-to-end application runtime from run start to final completion. |
| `t_start_to_warning_sec` | Time from run start to first interruption warning/detection event (0 for no-failure scenario by convention). |
| `t_warning_to_checkpoint_sec` | Time from warning/detection to checkpoint completion. |
| `t_checkpoint_to_restart_dispatch_sec` | Time from checkpoint completion to restart dispatch. |
| `t_restart_dispatch_to_restart_completed_sec` | Time from restart dispatch to restart completion terminal event. |
| `reprovision_time_sec` | Worker replacement duration in `REPLACE_RESUME` (instance loss to replacement readiness). |
| `observed_failure_count` | Number of interruption/failure events actually observed in the run (for example 0, 1, 2). |
| `recovery_cycle_count` | Number of completed recovery cycles recorded for the run (can be lower than observed failures if some cycles fail). |
| `approx_cost_start_to_warning` | Approximate cost accrued during `t_start_to_warning_sec`. |
| `approx_cost_warning_to_checkpoint` | Approximate cost accrued during `t_warning_to_checkpoint_sec`. |
| `approx_cost_checkpoint_to_restart_dispatch` | Approximate cost accrued during `t_checkpoint_to_restart_dispatch_sec`. |
| `approx_cost_restart_dispatch_to_restart_completed` | Approximate cost accrued during `t_restart_dispatch_to_restart_completed_sec`. |
| `approx_cost_reprovision` | Approximate cost associated with reprovisioning interval(s) in `REPLACE_RESUME`. |
| `approx_cost` | Estimated monetary cost of this run from elapsed resource time and instance pricing assumptions. |
| `notes` | Free-text context for anomalies, retries, invalidation reason, or operational observations. |

---

## 5. Lightweight Orchestration (Phase 4 Implementation Target)

Objective: keep executions manually controlled while reducing logging and metric capture errors.

### 5.1 Script role
A lightweight helper script should:
- Accept scenario parameters (benchmark, workers, strategy, failure scenario, repetition)
- Launch the corresponding cluster task command
- Record start/end timestamps automatically
- Collect output/report path references
- Parse key event timestamps from interruption_events or generated report
- Emit one metrics row per run to consolidated CSV

### 5.2 Design constraints
- Do not daemonize or replace existing cluster orchestration
- Keep operator in control (manual trigger per run or small batch)
- Be restart-safe: partial executions must not corrupt previous rows
- Mark invalid runs explicitly for rerun tracking

### 5.3 Suggested script outputs
- results/phase4/raw/<run_id>.txt
- results/phase4/metrics/phase4_metrics.csv
- results/phase4/metrics/phase4_invalid_runs.csv

---

## 6. Step-by-Step Plan to Conclude Phase 4

1. Finalize benchmark class per application
- Run pilot timing on EP/CG/LU with class C and 2 workers
- Adjust class only if runtime target is not met
- Freeze classes before full matrix

2. Freeze scenario catalog
- Define canonical names for S0, S1, S2
- Define exact failure injection moments and worker targeting rule

3. Implement lightweight metrics helper
- Add script and minimal config conventions
- Ensure run_id generation and CSV append are deterministic

4. Execute integration smoke matrix
- Run one end-to-end sample for each strategy with 2 workers
- Confirm all mandatory artifacts are produced

5. Execute full matrix campaign
- Run 243 planned points (or documented subset if later narrowed)
- Track invalid runs and rerun only invalid points

6. Consolidate and verify dataset quality
- Check missing values, duplicate run_id, inconsistent strategy labels
- Validate event ordering constraints (checkpoint before restart, etc.)

7. Generate analysis tables and figures
- Compare NONE vs REPLACE_RESUME vs DEGRADED_RESUME
- Compare S0/S1/S2 sensitivity by benchmark and worker count
- Compare per-failure normalized time/cost intervals for fair S1 vs S2 interpretation
- Compute S2-S1 marginal impact for time and cost intervals
- Compute approximate cost and cost/performance trade-offs

8. Produce Phase 4 artifact report
- Methods, matrix, quality gates, results summary, threats to validity

---

## 7. Integration Tests (Required Before Full Campaign)

### IT-1: Script-to-run-task integration
- Input one scenario parameter set
- Validate helper invokes run-task correctly
- Validate raw output and run metadata are persisted

### IT-2: Event timeline extraction
- For a run with interruption, validate extractor finds:
  - interruption_detected or interruption_warning
  - checkpoint_completed
  - restart_dispatched marker
  - restart_completed or terminal policy event
- Validate computed interval durations are non-negative for all four timing intervals

### IT-3: Strategy-specific fields
- REPLACE_RESUME run must emit reprovision_time_sec
- DEGRADED_RESUME run must emit interval fields and keep reprovision_time_sec as zero by convention
- NONE run must keep interruption/recovery interval fields at zero by convention

### IT-4: Invalid run handling
- Force one failed/invalid run
- Validate invalid record is written and main CSV remains consistent

### IT-5: Idempotent append behavior
- Re-run same scenario with different run_id
- Validate no row overwrite and no malformed CSV

---

## 8. Validation Gates (Phase 4 Acceptance)

- [ ] G1: Class calibration completed and frozen before main campaign
- [ ] G2: Scenario catalog and naming convention frozen
- [ ] G3: Lightweight helper script integrated and smoke-tested
- [ ] G4: Integration tests IT-1..IT-5 pass
- [ ] G5: Full matrix executed (or clearly justified reduced matrix)
- [ ] G6: Consolidated dataset has no duplicate run_id and no critical missing metrics
- [ ] G7: Analysis tables answer core questions:
  - overhead by strategy
  - recovery time by strategy and scenario
  - cost of recovery and approximate total cost impact
  - workload sensitivity (EP/CG/LU)
  - scaling impact (2/4/8 workers)
- [ ] G8: Phase 4 artifact document produced with reproducible methods and conclusions

---

## 9. Risks and Mitigations (Execution Phase)

Risk: high variability in spot replacement time  
Mitigation: preserve repetition count, report dispersion (min/max/stddev), flag outliers.

Risk: invalid runs due to transient cloud/tooling issues  
Mitigation: explicit invalid-run registry and controlled reruns only for invalid points.

Risk: inconsistent manual execution metadata  
Mitigation: force all runs through helper script for timestamping and metric row creation.

Risk: benchmark runtime drift across sizes  
Mitigation: freeze per-benchmark class after calibration and do not change mid-campaign.

---

## 10. Out of Scope for Phase 4

- New recovery policy implementation (already addressed in current codebase)
- Daemonized recovery service redesign
- Multi-provider parity expansion beyond AWS
- Advanced billing API integration for exact cloud invoice attribution

---

## 11. Expected Phase 4 Deliverables

- Phase 4 execution helper script (lightweight, metrics-focused)
- Frozen experiment matrix and scenario catalog
- Consolidated metrics dataset (CSV)
- Validation log (integration tests + acceptance gates)
- Final Phase 4 artifact report with analysis-ready results and conclusions
