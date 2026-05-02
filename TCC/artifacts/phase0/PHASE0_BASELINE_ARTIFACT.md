# Phase 0 Baseline Artifact

Date: 2026-05-02
Project: Integrating MANA Fault Tolerance into HPC@Cloud for AWS Spot Clusters
Repository: hpcac-toolkit

## 1. Purpose
This artifact records the validated Phase 0 baseline environment for Amazon Linux 2023 with MPICH 3.3.2, Slurm 24.05.4, MANA, and NPB.

Phase 0 objective: establish a reproducible, working baseline before implementing role-aware orchestration and interruption handling.

## 2. Baseline Stack (Locked)
- OS: Amazon Linux 2023
- MPI: MPICH 3.3.2
- Slurm: 24.05.4 (source build at /opt/slurm-24.05.4)
- MANA: installed at /opt/mana
- NPB: 3.4.4 (MPI)

## 3. Critical Configuration Decisions
- Slurm cgroup plugin disabled on AL2023:
  - CgroupPlugin=disabled in /etc/slurm/cgroup.conf
  - TaskPlugin=task/none in /etc/slurm/slurm.conf
- slurm.conf is host-specific and must be generated on first boot (not baked in AMI).
- cgroup.conf is host-agnostic and is safe to bake in AMI.
- For MPICH 3.3.2 multi-rank restart, use Slurm path (srun), not Hydra (mpirun).

## 4. Validation Summary (Passed)
### 4.1 Slurm daemon and node registration
- Result: PASS
- Evidence: scontrol show node reported node State=IDLE on fresh instance.

### 4.2 NPB MPI sanity run
- Benchmark: EP Class A (2 ranks)
- Command path: srun -N1 -n2 .../NPB3.4-MPI/bin/ep.A.x
- Result: PASS
- Evidence: "Verification SUCCESSFUL"
- Observed runtime: approximately 3.9 seconds (single-node, 2 ranks)

### 4.3 MANA checkpoint/restart smoke test
- Test target: /opt/mana/mpi-proxy-split/test/mpi_hello_world.mana.exe
- Execution mode: salloc + srun + mana_coordinator + mana_launch/mana_status/mana_restart
- Result: PASS
- Evidence:
  - mana_status --list showed WorkerState::RUNNING for 2 ranks.
  - Checkpoint directories created: ckpt_rank_0 and ckpt_rank_1.
  - Restart resumed execution successfully after checkpoint.

## 5. Known Constraints and Workarounds
- mpirun/Hydra restart with MPICH 3.3.2 multi-rank is a known failing path (signal 9 behavior).
- Use srun for launch and restart in this stack.
- Keep coordinator and launch/restart in the same Slurm allocation to preserve matching .mana-slurm-$SLURM_JOB_ID.rc.

## 6. AMI Reproducibility Record
Fill and keep frozen for this experiment batch.

- AWS Region: use-east-1
- Availability Zone: us-east-1a
- Source instance ID: i-07bca576b432a26f2
- Source instance type: t3.medium
- Final AMI ID: ami-06af33e2399c52709
- AMI Name: hpcac-al2023-mpich3.3.2-mana-slurm24.05.4-v1
- AMI Creation Timestamp (UTC): 2026/05/02 16:05 GMT-3

Software fingerprints:
- MPICH version output: MPICH 3.3.2
- Slurm version output: slurm 24.05.4
- MANA git commit (from /opt/mana): f967d3a129b667d2fbfb5aa4821df6dafe793a39
- NPB version: 3.4.4

Suggested AMI tags:
- phase=0-baseline
- mana=ok
- slurm=ok
- npb=ok
- date=2026-05-02

## 7. Acceptance Criteria for Phase 0 (Checklist)
- [x] Slurm services start and node becomes IDLE on fresh instance.
- [x] NPB EP.A runs under Slurm with Verification SUCCESSFUL.
- [x] MANA launch/checkpoint/restart works with hello-world test under Slurm.
- [x] Setup commands updated to include AL2023 cgroup constraints and first-boot slurm.conf regeneration.
- [x] Final AMI metadata fields completed in this artifact.

## 8. Immediate Next Actions (Phase 1 Entry)
1. Implement role-aware node model in HPC@Cloud (head vs worker).
2. Enforce one on-demand head with head-first provisioning.
3. Automate Slurm + MANA bootstrap per role.
4. Validate checkpoint/restart on 1 head + 2 workers cluster.
5. Add interruption detection and first recovery policy.

## 9. File References
- Setup and validation commands: AL2023_SETUP_COMMANDS.md
- Project plan: TCC_MANA_INTEGRATION_PLAN.md
