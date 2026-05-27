# TCC Project Proposal
## Integrating MANA Fault Tolerance into HPC@Cloud for AWS Spot Clusters

Date: 2026-04-24
Last Updated: 2026-05-27 (Phase 4 completed — all planned phases done)

## 1. Project Idea (Problem and Motivation)
This TCC proposes integrating MANA (MPI-Agnostic Network-Agnostic checkpoint/restart) into HPC@Cloud to improve fault tolerance of AWS spot-based HPC clusters.

### Motivation
- Spot instances reduce cost but can be interrupted.
- HPC MPI workloads are sensitive to node loss.
- Transparent checkpoint/restart can reduce lost work and make spot usage practical.

### Research Objective
Design, implement, and evaluate a resilient cluster execution strategy that combines:
- Spot workers (cost efficiency)
- On-demand control plane (stability)
- Shared checkpoint storage (recovery)
- Automated interruption handling and restart orchestration

## 2. Target Cluster Architecture
### Recommended Topology
- Head/coordinator node (on-demand):
  - Slurm controller components
  - MANA coordinator
  - Orchestration scripts/services
- Compute worker nodes (spot, optionally mixed with on-demand fallback):
  - Slurm compute roles
  - MPI runtime + MANA runtime dependencies
- Shared storage:
  - EFS mounted on all nodes (already supported by HPC@Cloud)
  - Stores checkpoints, restart artifacts, logs, and hostfiles

### Why This Topology Is Consistent
- Head node remains stable, avoiding full control-plane loss.
- Workers remain cost-optimized.
- Checkpoints survive node replacement events.

## 3. What Already Exists in HPC@Cloud (Usable Foundation)
### Current Strengths to Leverage
- AWS infrastructure provisioning and teardown flows are already implemented.
- Per-node deterministic networking (private/public IP tracking).
- Shared EFS integration and mount support.
- Per-node post-boot command execution through SSM.
- Cluster/node metadata persisted in SQLite.
- Existing failure handling during spawn (migration and on-demand fallback policies).

### Implication
- The platform already provides almost all infrastructure primitives required for robust MANA integration.
- The main missing pieces are runtime orchestration and role-aware node management.

## 4. Key Gaps to Implement
### A) Role-Aware Node Model
- Add explicit node role (head vs worker) in configuration and DB.
- Enforce invariants:
  - Exactly one head node.
  - Head node allocation mode fixed to on-demand.

### B) Slurm + MANA Bootstrap Automation
- Extend init commands/templates to install and configure:
  - Slurm controller (head)
  - Slurm workers
  - MANA binaries and dependencies
  - Shared checkpoint directories on EFS

### C) Spot Interruption Detection and Reaction Loop
- Add runtime monitoring service for worker nodes.
- Detect interruption/rebalance signals.
- Trigger coordinated checkpoint before node loss when possible.

### D) Checkpoint/Restart Orchestration
- Add workflow to:
  - checkpoint running MPI workload
  - remove/drain lost nodes from scheduler
  - optionally respawn replacement spot nodes
  - restart from latest checkpoint with selected policy

### E) Experiment and Metrics Pipeline
- Automate benchmark runs.
- Collect timing, interruption, restart, and cost metrics.

## 5. Recommended Implementation Path (Phased)
### Phase 0 - Scope Lock and Reproducible Baseline ✅ COMPLETED 2026-05-02
#### Goal
- Freeze a baseline setup where a representative MPI workload runs under Slurm on HPC@Cloud without interruption handling.

#### Interpretation
- This is the environment-definition phase.
- Decide and lock the exact stack to avoid moving targets during implementation and evaluation.

#### Baseline Decisions for This TCC (Current Proposal)

> **Note on deviations from original plan (updated 2026-05-27):**
> The original plan targeted m5.8xlarge workers, FT benchmark, and 20–40 min runtimes.
> Actual execution used m5.xlarge workers (quota availability), EP instead of FT (EP provides
> a cleaner low-communication contrast than FT), and the runtimes were shorter than targeted
> (CG ~35 s, LU ~147 s, EP ~330 s natively). See Phase 4 artifact for the locked experiment matrix.

- Head node: t3.large (on-demand) — *original plan: t3.2xlarge; t3.large used in actual experiments*
- Worker nodes: m5.xlarge (spot) — *original plan: m5.8xlarge; changed due to quota availability*
- Workload suite: NAS Parallel Benchmarks (NPB), MPI version (NPB 3.4.x)
- Selected benchmarks:
  - CG: irregular memory access and communication
  - EP: embarrassingly parallel, low communication — *substituted for FT for cleaner low-communication baseline*
  - LU: pseudo-application close to real CFD behavior
- Final problem classes: CG Class C, EP Class D, LU Class C
  - EP Class D used instead of C because Class C completed too quickly on m5.xlarge
- Worker counts for experiments: 2, 4 — *8-worker runs pending AWS EIP quota increase*
- Target runtime:
  - Baseline (no fault): 20 to 40 minutes (not met — actual runtimes much shorter; see note above)
  - Fault-injection run: varies by benchmark

#### Pilot Procedure to Finalize Class Choice
1. Spawn a 4-worker cluster (m5.8xlarge spot + t3.2xlarge head).
2. Run CG.C, FT.C, LU.C once each.
3. For each benchmark:
   - If runtime < 15 min, promote to D.
   - If runtime > 60 min, downgrade to B.
   - Otherwise keep C.
4. Lock classes and do not change them during the project.

#### Locked Experiment Matrix (Actual — finalized during Phase 4)
| Benchmark | Class | Worker Counts | Strategies | Repetitions (actual) |
|---|---|---|---|---|
| CG | C | 2 / 4 | noFT, MANA_noFT, REPLACE, DEGRADED | 1 per point |
| EP | D | 2 / 4 | noFT, MANA_noFT, REPLACE, DEGRADED | 1 per point |
| LU | C | 2 / 4 | noFT, MANA_noFT, REPLACE, DEGRADED | 1 per point |

Total runs executed: 24 (3 benchmarks × 2 cluster sizes × 4 strategies).
8-worker runs and statistical repetitions (≥3 per point) are deferred to future work.
FT benchmark from original plan replaced by EP.

#### Deliverables
- [x] AMI built and validated: ami-06af33e2399c52709 (us-east-1a, t3.medium)
- [x] Stack locked: MPICH 3.3.2 + MANA (f967d3a1) + Slurm 24.05.4 + NPB 3.4.4
- [x] First-boot smoke test defined and passing (section 10 of AL2023_SETUP_COMMANDS.md)
- [x] MANA checkpoint/restart validated on single node (mpi_hello_world + srun path)
- [x] Reproducibility record in TCC/artifacts/phase0/PHASE0_BASELINE_ARTIFACT.md
- [ ] Baseline cluster config YAML (t3.2xlarge head + m5.8xlarge workers) — pending credits
- [ ] Baseline task workflow YAML (CG, FT, LU runs) — pending credits
- [ ] Pilot run report (runtime per benchmark/class/size) — pending credits
- [ ] Baseline validation report (clean run, multi-node, no interruptions) — pending credits

#### Temporary Low-Credit Mode (Current Sponsorship Constraint)
##### Objective
- Keep progress moving with minimal AWS spend while preserving technical validity.

##### Recommended Temporary Profile
- Head node: t3.medium or t3.large (on-demand)
- Worker nodes: m5.large (spot), starting with 2 workers
- Benchmarks for this stage:
  - EP.A for pipeline sanity
  - CG.A or CG.B for MPI behavior
- Runtime target:
  - 5 to 15 min (smoke)
  - 15 to 25 min (stability)
- Repetitions: 2 to 3 per point

##### What to Postpone Until Credits Return
- Full benchmark matrix with CG/FT/LU at larger classes
- 2/4/8 (and higher) scaling comparisons with full statistical confidence
- Final cost-performance conclusions for dissertation results

##### Practical Decision for Now
- Execute a lightweight Phase 0 (engineering baseline) and prioritize Phase 1 and Phase 2 implementation.
- **Status:** Phase 0 engineering baseline completed. AMI frozen. Proceed to Phase 1.

### Phase 1 - Node Roles and Hybrid Topology ✅ COMPLETED 2026-05-02
#### Goal
- Enable one on-demand head and N spot workers.

#### Tasks
- [x] Extend cluster YAML schema with role field per node.
- [x] Update validation rules in create flow (fast-fail: unknown role, !=1 head, head with spot).
- [x] Persist role in DB node model and migrations.
- [x] Adapt spawn ordering so head is initialized first.

#### Deliverable
- [x] Cluster creation/spawn with role-aware behavior.
- [x] Artifact: TCC/phase1/PHASE1_ARTIFACT.md

### Phase 2 - Slurm and MANA Installation Flow ✅ COMPLETED 2026-05-06
#### Goal
- Bootstrap complete execution environment automatically.

#### Tasks
- [x] Add role-specific init command templates (head vs worker).
- [x] Configure Slurm controller on head and slurmd on workers via SSM.
- [x] Configure MANA coordinator placement on head node.
- [x] Verify EFS checkpoint directory consistency across nodes.
- [x] ~~Pre-baked AMI strategy~~ — **completed in Phase 0:**
  - Single common AMI for all node roles (al2023 + MPICH + MANA + Slurm binaries).
  - Role differentiation done at first-boot via init commands (slurm.conf generation).
  - AMI versioned and tagged: ami-08b9f0fb120be798a.

#### Deliverable
- [x] End-to-end job launch and manual MANA checkpoint/restart on multi-node cluster.
- [x] Artifact: `TCC/artifacts/phase2/PHASE2_ARTIFACT.md`

#### Key Findings
- `slurm.conf` is generated at spawn time from injected env-vars (`HPCAC_*`); no AMI baking needed.
- `$HOME` is node-local: MANA rc file must be copied via EFS to each node per allocation.
- EFS (`/shared`) stores both Slurm config (munge key, slurm.conf) and MANA checkpoints.
- Fixed coordinator port (7779) required for reliable multi-run operation.
- `MpiDefault=pmi2` required; `pmix` plugin unavailable on AL2023.

### Phase 2.5 - AMI Hardening and Reproducibility Gate ✅ COMPLETED IN PHASE 0
#### Goal
- Reduce bootstrap drift and startup time before interruption-aware automation.

#### Tasks
- [x] Freeze package versions and MPI toolchain in AMI build scripts (AL2023_SETUP_COMMANDS.md).
- [x] First-boot smoke tests defined and passing:
  - [x] Slurm node State=IDLE
  - [x] mana_coordinator available
  - [x] mana_launch + checkpoint + restart validated
  - [x] EFS mount check — confirmed in Phase 2 multi-node validation
- [x] AMI IDs recorded in TCC/artifacts/phase0/PHASE0_BASELINE_ARTIFACT.md.

#### Deliverable
- [x] Versioned, validated single AMI for all node roles: ami-06af33e2399c52709
- [x] EFS mount check confirmed: `/shared/checkpoints` visible on all nodes via `srun -N2 -n2 --label /bin/sh -c 'ls /shared/checkpoints'`.

### Phase 3 - Interruption-Aware Runtime Control ✅ COMPLETED 2026-05-13

#### Goal
- React to spot interruption signals with controlled checkpoint/recovery.

#### Tasks
- [x] Implement watcher process running alongside the MPI job (`src/commands/cluster/watcher.rs`).
- [x] Trigger `mana_status --bcheckpoint` on interruption / failure detection.
- [x] Integrate node draining and replacement logic in HPC@Cloud.
- [x] Implement `auto_test_failure` YAML field for reproducible synthetic failure injection.
- [x] Implement two recovery strategies switchable via task YAML:
  - `REPLACE_RESUME`: replace the lost node with a new EC2 spot instance and restart.
  - `DEGRADED_RESUME`: restart immediately with one fewer process (no reprovisioning).
- [x] Persist structured event log embedded in run-task output as `[RUN_METRICS]` blocks.
- [x] Deploy MANA binaries (6 files) to worker nodes after each AMI respawn.

#### Deliverable
- [x] Artifact: `TCC/artifacts/phase3/PHASE3_ARTIFACT.md`
- [x] Both recovery strategies (REPLACE_RESUME, DEGRADED_RESUME) validated in controlled tests.
- [x] `auto_test_failure` mechanism produces reproducible failure timing across runs.

#### Key Findings
- MANA binaries must be redeployed to worker nodes after every respawn (AMI does not pre-install the version used in experiments).
- Phase 2 of recovery (checkpoint → restart dispatch) is the dominant cost differentiator: ~185–220 s for REPLACE vs ~21–23 s for DEGRADED.
- The `auto_test_failure` YAML field (`trigger_after_secs`, `target_node_index`) provides fully reproducible fault injection.

### Phase 4 - Evaluation and Metrics Consolidation ✅ COMPLETED 2026-05-27

#### Goal
- Execute a reproducible evaluation campaign for the recovery policies implemented in Phase 3,
  producing TCC-quality metrics, comparisons, and cost/performance analysis.

#### Tasks
- [x] Finalize benchmark classes: CG Class C, EP Class D, LU Class C.
- [x] Implement `[RUN_METRICS]` structured output blocks in run-task output for machine-readable metric extraction.
- [x] Implement `analyze.py` pipeline: parse result files → flat CSV + summary.md + 6 analysis plots.
- [x] Execute 24-run pilot matrix (3 benchmarks × 2 cluster sizes × 4 strategies, 1 repetition each).
- [x] Produce consolidated dataset (`results_raw.csv`).
- [x] Generate analysis figures:
  - `fig1_wall_time.png` — absolute wall time per benchmark and strategy
  - `fig2_mana_overhead.png` — MANA interposition overhead relative to noFT
  - `fig3_recovery_phases.png` — stacked recovery phases (Phase 1 / 2 / 3) for FT runs
  - `fig4_scalability.png` — wall time vs worker count per strategy
  - `fig5_cost.png` — cost per run: noFT on-demand vs FT strategies on spot
  - `fig6_replace_vs_degraded.png` — FT overhead relative to MANA_noFT baseline
- [x] Write English analysis report: `TCC/artifacts/phase4/analysis/analysis_report.md`
- [x] Write PT-BR professor update: `TCC/artifacts/phase4/analysis/analysis_report_ptbr.md`
- [x] Write Phase 4 artifact: `TCC/artifacts/phase4/PHASE4_ARTIFACT.md`

#### Cluster Profile (Actual)
- Head node: t3.large (on-demand), region us-west-2
- Worker nodes: m5.xlarge (spot), AMI `ami-053c434f0309cfbe0`
- EFS: enabled, `/shared/checkpoints`

#### Key Findings
- MANA overhead scales with communication intensity: EP ~1.49×, LU ~1.66×, CG 4w ~1.57×.
- DEGRADED_RESUME consistently outperforms REPLACE_RESUME due to Phase 2 infrastructure cost.
- Phase 2 (checkpoint → dispatch): DEGRADED ~21–23 s; REPLACE ~185–221 s (EC2 provisioning).
- Economic argument: DEGRADED on spot is 34% cheaper than noFT on-demand for EP Class D (4 workers).
- Short jobs (CG) favor DEGRADED; for very short jobs, FT adds cost — noFT on-demand is cheaper.

#### Deliverable
- [x] Artifact: `TCC/artifacts/phase4/PHASE4_ARTIFACT.md`
- [x] Consolidated dataset: `TCC/artifacts/phase4/analysis/results_raw.csv`
- [x] Analysis figures: `TCC/artifacts/phase4/analysis/plots/`
- [x] Analysis reports: `analysis_report.md` (EN), `analysis_report_ptbr.md` (PT-BR)

> **Note:** Phase 5 (Evaluation and Analysis) from the original plan was folded into Phase 4.
> The original Phase 5 metrics are all covered by the Phase 4 analysis pipeline and reports.

## 6. Consistency Requirements for a Strong Project
### A) Experimental Consistency
- Fix region/AZ when possible.
- Fix software stack and AMI per experiment batch.
- Repeat each scenario multiple times.
- Separate warm-up runs from measured runs.

### B) Configuration Consistency
- Version-control all cluster and task YAML files used in experiments.
- Track exact policy flags and checkpoint intervals used by MANA.
- Keep one canonical shared checkpoint path convention.

### C) Operational Consistency
- Add structured logs for each orchestration stage.
- Add explicit timeout and retry policies.
- Ensure idempotency of restart and recovery commands.

### D) Analysis Consistency
- Compare against clear baselines:
  - full on-demand cluster
  - spot cluster without MANA
  - spot cluster with MANA
- Report mean and variability (min/max or standard deviation).

## 7. Minimal Viable Implementation (MVP) Recommendation
If timeline is tight, prioritize:
- One stable on-demand head + spot workers
- Slurm + MANA integrated and working
- Interruption event triggers checkpoint and restart after worker replacement
- One benchmark application and one synthetic interruption scenario
- Cost/time comparison against full on-demand baseline

This MVP is sufficient to demonstrate technical feasibility and economic value.

## 8. Risk Register and Mitigations
### Risk
Slurm/MANA integration complexity across node bootstrap timing.

### Mitigation
Create idempotent bootstrap scripts and staged readiness checks.

### Risk
Checkpoint duration too high for interruption notice window.

### Mitigation
Use periodic checkpoints in addition to reactive checkpointing.

### Risk
Inconsistent restart due to environment drift.

### Mitigation
Keep immutable AMI and pinned dependency versions.

### Risk
Spot replacement delays degrade throughput.

### Mitigation
Evaluate fallback to temporary on-demand worker replacement.

## 9. Suggested Immediate Next Actions

### Completed
- [x] Freeze low-credit baseline environment (t3.medium AMI, single node) and validate clean run.
- [x] Validate MANA checkpoint/restart on single node with Slurm path.
- [x] Produce Phase 0 artifact and lock AMI.
- [x] Implement role-aware node model (head vs worker) with DB migration, validation, head-first spawn.
- [x] Produce Phase 1 artifact.
- [x] Produce Phase 2 artifact.
- [x] Implement interruption-aware watcher, auto_test_failure, REPLACE_RESUME, DEGRADED_RESUME.
- [x] Produce Phase 3 artifact.
- [x] Implement [RUN_METRICS] structured output blocks and analyze.py pipeline.
- [x] Execute 24-run pilot matrix and produce analysis reports.
- [x] Produce Phase 4 artifact.

### Current Status: All Phases Completed (updated 2026-05-27)

All planned implementation phases (Phase 0 through Phase 4) are complete.
The pilot evaluation campaign (24 runs) and analysis reports have been produced.

#### Remaining Work (Future / Post-TCC)
1. Repeat each configuration ≥3 times for statistical confidence (standard deviation on all metrics).
2. Extend to 8-worker clusters (pending AWS EIP quota increase in us-west-2).
3. Test LU Class D and EP Class E for longer job durations where spot savings should be more pronounced.
4. Confirm CG Class C MANA overhead at 2 workers (suspected single-run outlier at 3.13×).
5. Investigate REPLACE_RESUME for workloads >10 min where its Phase 2 cost can be amortized.
## 10. Expected TCC Contribution Statement
This project contributes a practical strategy for making spot-based HPC clusters more resilient and economically viable by combining scheduler-aware orchestration, transparent MPI checkpoint/restart (MANA), and cloud-native dynamic node replacement within HPC@Cloud.
