# TCC Project Proposal
## Integrating MANA Fault Tolerance into HPC@Cloud for AWS Spot Clusters

Date: 2026-04-24

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
### Phase 0 - Scope Lock and Reproducible Baseline
#### Goal
- Freeze a baseline setup where a representative MPI workload runs under Slurm on HPC@Cloud without interruption handling.

#### Interpretation
- This is the environment-definition phase.
- Decide and lock the exact stack to avoid moving targets during implementation and evaluation.

#### Baseline Decisions for This TCC (Current Proposal)
- Head node: t3.2xlarge (on-demand, burstable mode Unlimited)
- Worker nodes: m5.8xlarge (spot), primary profile
  - m5.16xlarge reserved as optional stress profile for larger classes only
  - m5.8xlarge gives good memory headroom with lower per-run cost, enabling more repetitions
- Workload suite: NAS Parallel Benchmarks (NPB), MPI version (NPB 3.4.x)
- Selected benchmarks:
  - CG: irregular memory access and communication
  - FT: all-to-all communication stress
  - LU: pseudo-application close to real CFD behavior
- Initial problem class: C for all three benchmarks
  - Promote to D if runtime < 15 min on 4-worker pilot
  - Downgrade to B if runtime > 60 min on 4-worker pilot
- Worker counts for experiments: 2, 4, 8
- Target runtime:
  - Baseline (no fault): 20 to 40 minutes
  - Fault-injection run: 30 to 60 minutes

#### Pilot Procedure to Finalize Class Choice
1. Spawn a 4-worker cluster (m5.8xlarge spot + t3.2xlarge head).
2. Run CG.C, FT.C, LU.C once each.
3. For each benchmark:
   - If runtime < 15 min, promote to D.
   - If runtime > 60 min, downgrade to B.
   - Otherwise keep C.
4. Lock classes and do not change them during the project.

#### Locked Experiment Matrix (To Confirm After Pilot)
| Benchmark | Class | Worker Counts | Repetitions |
|---|---|---|---|
| CG | C | 2 / 4 / 8 | >= 5 per point |
| FT | C | 2 / 4 / 8 | >= 5 per point |
| LU | C | 2 / 4 / 8 | >= 5 per point |

#### Deliverables
- Baseline cluster config (YAML with t3.2xlarge head + m5.8xlarge workers)
- Baseline task workflow (YAML with CG, FT, LU runs)
- Pilot run report (runtime per benchmark/class/size)
- Reproducibility checklist (versions, AMI IDs, region, AZ, instance types, NPB version)
- Baseline validation report (clean run with timing/logs, no interruptions)

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

### Phase 1 - Node Roles and Hybrid Topology
#### Goal
- Enable one on-demand head and N spot workers.

#### Tasks
- Extend cluster YAML schema with role field per node.
- Update validation rules in create flow.
- Persist role in DB node model and migrations.
- Adapt spawn ordering so head is initialized first.

#### Deliverable
- Cluster creation/spawn with role-aware behavior.

### Phase 2 - Slurm and MANA Installation Flow
#### Goal
- Bootstrap complete execution environment automatically.

#### Tasks
- Add role-specific init command templates.
- Configure Slurm controller on head and slurmd on workers.
- Configure MANA coordinator placement (head).
- Verify EFS checkpoint directory consistency across nodes.
- Add pre-baked AMI strategy:
  - Build AMIs with Slurm and MANA pre-installed per role.
  - Use separate AMIs for head and worker roles (or common AMI + role scripts).
  - Version/tag AMIs per experiment batch.

#### Deliverable
- End-to-end job launch and manual MANA checkpoint/restart on the cluster.

### Phase 2.5 - AMI Hardening and Reproducibility Gate
#### Goal
- Reduce bootstrap drift and startup time before interruption-aware automation.

#### Tasks
- Freeze package versions and MPI toolchain in AMI build scripts.
- Add first-boot smoke tests:
  - Slurm status
  - mana_coordinator
  - mana_launch dry run
  - EFS mount check
- Record AMI IDs in experiment metadata and cluster/task configs.

#### Deliverable
- Versioned, reusable AMI set validated for repeatable experiments.

### Phase 3 - Interruption-Aware Runtime Control
#### Goal
- React to spot interruption signals with controlled checkpoint/recovery.

#### Tasks
- Implement interruption detection agent/process.
- Trigger checkpoint command path.
- Integrate node draining and replacement logic.
- Persist runtime event log:
  - interruption detected
  - checkpoint started/completed
  - restart started/completed

#### Deliverable
- Automatic recovery pipeline validated in controlled tests.

### Phase 4 - Recovery Policies for Evaluation
#### Goal
- Compare at least two recovery strategies.

#### Suggested Policies
- Policy A: pause/restart only after replacement node is ready (fixed-size recovery).
- Policy B: resume with reduced nodes, then rescale and restart later with full size.

#### Deliverable
- Policy implementation switch in config.

### Phase 5 - Evaluation and Analysis
#### Goal
- Produce TCC-quality evidence.

#### Metrics
- Total execution time (makespan)
- Checkpoint overhead
- Recovery latency
- Lost work after interruption
- Job completion success rate
- Total cloud cost per completed run
- Cost/performance ratio vs on-demand baseline

#### Deliverable
- Structured experiment matrix and statistical summary.

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
1. Freeze low-credit baseline config (t3.medium or t3.large head, m5.large workers) and validate one full clean run.
2. Add node role support to cluster configuration and DB model.
3. Implement head-first bootstrap with Slurm + MANA installation scripts.
4. Validate manual checkpoint/restart on a 1 head + 2 worker cluster.
5. Implement interruption detection plus single-policy automatic recovery, then expand benchmarking when credits return.

## 10. Expected TCC Contribution Statement
This project contributes a practical strategy for making spot-based HPC clusters more resilient and economically viable by combining scheduler-aware orchestration, transparent MPI checkpoint/restart (MANA), and cloud-native dynamic node replacement within HPC@Cloud.
