# Phase 4 Artifact — Evaluation and Metrics Consolidation

Date: 2026-05-27
Project: Integrating MANA Fault Tolerance into HPC@Cloud for AWS Spot Clusters
Repository: hpcac-toolkit

## 1. Purpose

This artifact records the completed Phase 4 work: the design and execution of a
reproducible evaluation campaign for the fault-tolerance recovery policies
implemented in Phase 3, including the metrics capture pipeline, the 24-run
experiment matrix, and the full comparative analysis.

Phase 4 objective: produce TCC-quality evidence comparing four strategies (noFT,
MANA_noFT, REPLACE_RESUME, DEGRADED_RESUME) across three benchmarks and two
cluster sizes, with economic analysis using AWS spot vs on-demand pricing.

---

## 2. Scope Alignment (Planned vs Actual)

The Phase 4 plan projected a 243-run full matrix (3 benchmarks × 3 cluster sizes
× 3 strategies × 3 failure scenarios × 3 repetitions). The actual execution
was a 24-run pilot campaign due to time and AWS quota constraints:

| Dimension | Planned | Actual |
|---|---|---|
| Benchmarks | CG, EP, LU (3) | CG, EP, LU (3) — EP substituted for FT |
| Problem classes | C for all | CG: C, LU: C, EP: D |
| Cluster sizes | 2, 4, 8 workers | 2, 4 workers (8-worker pending EIP quota) |
| Strategies | NONE, REPLACE, DEGRADED | noFT, MANA_noFT, REPLACE, DEGRADED (4) |
| Failure scenarios | S0, S1, S2 | S0 (no failure) + S1 (one failure) only |
| Repetitions per point | 3 | 1 |
| Total runs | 243 | 24 |

The 24-run pilot covers all core research questions (MANA overhead, recovery
phase decomposition, strategy comparison, scalability, economic trade-offs) and
is sufficient to draw the principal conclusions of this TCC.

---

## 3. Cluster and Environment Profile

- **AWS Region:** us-west-2
- **Availability Zone:** us-west-2a
- **Head node:** t3.large, on-demand
- **Worker nodes:** m5.xlarge, spot
- **AMI (workers):** ami-053c434f0309cfbe0 (`mana-worker-fix-20260526`)
  - MANA fix: `libdmtcp` wrapperLockCount race fix + `libmana` MPI_Test_internal fix
- **EFS:** enabled, mounted at `/shared`; checkpoint directory: `/shared/checkpoints`
- **Pricing used in cost analysis:**
  - m5.xlarge spot: $0.0585/hr (~70% discount vs on-demand $0.1920/hr)
  - t3.large on-demand: $0.0832/hr

---

## 4. Metrics Infrastructure Implemented

### 4.1 `auto_test_failure` Mechanism

A YAML-level field (`auto_test_failure`) was added to the task specification,
enabling fully reproducible failure injection without manual intervention:

```yaml
auto_test_failure:
  trigger_after_secs: 3      # Kill the target node this many seconds after job starts
  target_node_index: 1       # Worker index to terminate (0 = head, 1 = first worker, ...)
```

The watcher (`src/commands/cluster/watcher.rs`) reads this field and issues an
EC2 `terminate-instances` call at the specified time. This guarantees that the
failure moment is identical across REPLACE and DEGRADED runs for the same
benchmark, making phase timings directly comparable.

### 4.2 `[RUN_METRICS]` Structured Output Blocks

The run-task command (`src/commands/cluster/run_task.rs`) embeds a
machine-readable block at the end of every result file:

```
[RUN_METRICS]
benchmark = CG
class = C
workers = 4
strategy = DEGRADED
...
phase1_s = 26.644
phase2_s = 23.099
phase3_s = 110.259
recovery_total_s = 160.002
[/RUN_METRICS]
```

Fields captured per run:
- `benchmark`, `class`, `workers`, `strategy`
- `worker_instance_type`, `head_instance_type`
- `ft_wall_time_s` — time from job start to MPI application completion
- `total_time_s` — ft_wall_time_s + setup overhead
- `app_time_s`, `mops_total`, `mops_per_proc` — from NPB output
- `verification` — NPB verification status (SUCCESSFUL / UNSUCCESSFUL)
- `recovery_cycles` — count of completed recovery events
- `phase1_s`, `phase2_s`, `phase3_s`, `recovery_total_s` — recovery phase breakdown
- `auto_failure_trigger_secs` — the configured failure injection time

### 4.3 `analyze.py` Pipeline

`TCC/analyze.py` is the analysis entry point. It:

1. Recursively scans `TCC/` for result `.txt` files containing `[RUN_METRICS]` blocks.
2. Parses each block into a row using the `parse_result_file()` function.
3. Derives the run identifier from the task tag, using the regex
   `^([a-zA-Z]+)-([A-Z])_(\d+)w_[^_]+_(.+)$` — the `[^_]+` pattern (not `\w+`)
   correctly handles instance types like `m5xl` without consuming the strategy suffix.
4. Computes per-run cost using spot pricing for workers and on-demand for the head node.
5. Writes `results_raw.csv` with one row per run.
6. Generates `summary.md` with formatted tables.
7. Produces six analysis figures (see Section 5).

---

## 5. Analysis Figures Produced

| File | Content |
|---|---|
| `plots/fig1_wall_time.png` | Absolute wall time per benchmark and strategy (grouped bar chart) |
| `plots/fig2_mana_overhead.png` | MANA interposition overhead factor relative to noFT (all 4 strategies) |
| `plots/fig3_recovery_phases.png` | Stacked Phase 1 / Phase 2 / Phase 3 breakdown for FT runs |
| `plots/fig4_scalability.png` | Wall time vs worker count per strategy |
| `plots/fig5_cost.png` | Cost per run: noFT on-demand vs FT strategies on spot workers |
| `plots/fig6_replace_vs_degraded.png` | FT overhead relative to MANA_noFT baseline (REPLACE vs DEGRADED) |

---

## 6. Experiment Execution Log

All 24 runs were executed on 2026-05-26 in region us-west-2. Each run was
triggered manually via `hpcac-toolkit cluster run-task`. Result files are stored
in `TCC/` and referenced by `results_raw.csv`.

**Run matrix:**

| Benchmark | Workers | noFT | MANA_noFT | REPLACE | DEGRADED |
|---|---|---|---|---|---|
| CG Class C | 2 | ✓ | ✓ | ✓ | ✓ |
| CG Class C | 4 | ✓ | ✓ | ✓ | ✓ |
| EP Class D | 2 | ✓ | ✓ | ✓ | ✓ |
| EP Class D | 4 | ✓ | ✓ | ✓ | ✓ |
| LU Class C | 2 | ✓ | ✓ | ✓ | ✓ |
| LU Class C | 4 | ✓ | ✓ | ✓ | ✓ |

All 24 runs completed with NPB `Verification SUCCESSFUL`. No invalid runs.

---

## 7. Results Summary

Full tables and graphs are in `analysis/analysis_report.md` (English) and
`analysis/analysis_report_ptbr.md` (Portuguese). Key results below.

### 7.1 Wall Time (seconds)

| Benchmark | Workers | noFT | MANA noFT | REPLACE | DEGRADED |
|---|---|---|---|---|---|
| CG Class C | 2 | 34.3 | 107.2 | 283.4 | 170.4 |
| CG Class C | 4 | 20.9 | 32.8 | 256.8 | 93.7 |
| EP Class D | 2 | 330.5 | 492.6 | 583.7 | 537.4 |
| EP Class D | 4 | 167.0 | 249.0 | 409.1 | 294.6 |
| LU Class C | 2 | 146.9 | 242.1 | 397.3 | 302.3 |
| LU Class C | 4 | 80.7 | 134.6 | 360.1 | 190.2 |

### 7.2 MANA Interposition Overhead (MANA_noFT ÷ noFT)

| Benchmark | 2 workers | 4 workers |
|---|---|---|
| CG Class C | 3.13× (suspected outlier) | 1.57× |
| EP Class D | 1.49× | 1.49× |
| LU Class C | 1.65× | 1.67× |

EP shows the lowest overhead (~1.49×) because it has negligible inter-process
communication. LU is ~1.66× (structured communication). CG at 4 workers is 1.57×,
consistent with the pattern. The CG 2-worker value (3.13×) is a suspected
single-run transient: the absolute overhead added was 72.9 s at 2w vs only 11.9 s
at 4w — physically inconsistent for a benchmark that communicates more at smaller
scale. Additional repetitions are needed to confirm.

### 7.3 Recovery Phase Breakdown (seconds)

| Benchmark | Workers | Strategy | Phase 1 | Phase 2 | Phase 3 | Total |
|---|---|---|---|---|---|---|
| CG C | 2 | REPLACE | 26.6 | 190.6 | 53.7 | 271.0 |
| CG C | 2 | DEGRADED | 26.6 | 23.1 | 110.3 | 160.0 |
| CG C | 4 | REPLACE | 26.5 | 185.4 | 38.1 | 250.0 |
| CG C | 4 | DEGRADED | 26.6 | 21.7 | 36.9 | 85.2 |
| EP D | 2 | REPLACE | 16.3 | 196.4 | 314.7 | 527.4 |
| EP D | 2 | DEGRADED | 16.3 | 21.7 | 444.4 | 482.4 |
| EP D | 4 | REPLACE | 16.4 | 185.6 | 151.5 | 353.5 |
| EP D | 4 | DEGRADED | 16.3 | 21.6 | 205.0 | 243.0 |
| LU C | 2 | REPLACE | 26.5 | 185.0 | 130.5 | 342.0 |
| LU C | 2 | DEGRADED | 26.6 | 21.7 | 198.9 | 247.2 |
| LU C | 4 | REPLACE | 26.5 | 220.9 | 57.5 | 305.0 |
| LU C | 4 | DEGRADED | 26.5 | 21.6 | 88.8 | 136.9 |

Phase 1 depends on checkpoint size (not strategy): ~16 s for EP, ~26.5 s for LU/CG.
Phase 2 is infrastructure-determined: ~21–23 s for DEGRADED, ~185–221 s for REPLACE.
Phase 3 reflects remaining computation on the post-recovery process count.

### 7.4 Economic Analysis (4-worker cluster)

| Benchmark | noFT on-demand | DEGRADED spot | Savings |
|---|---|---|---|
| CG Class C | $0.0049 | $0.0083 | −67% (more expensive) |
| LU Class C | $0.0191 | $0.0168 | +12% |
| EP Class D | $0.0395 | $0.0260 | +34% |

Spot-backed fault tolerance is economically viable for medium-to-long jobs (LU, EP).
For short jobs (CG), the recovery overhead outweighs the spot discount.
REPLACE is not cost-competitive in any tested benchmark.

---

## 8. Acceptance Gates (Phase 4 Checklist)

| Gate | Description | Status |
|---|---|---|
| G1 | Class calibration completed and frozen | ✅ PASS — CG:C, LU:C, EP:D |
| G2 | Scenario catalog and naming convention frozen | ✅ PASS — S0 (noFT, MANA_noFT) and S1 (REPLACE, DEGRADED) |
| G3 | Lightweight metrics helper implemented | ✅ PASS — `analyze.py` + `[RUN_METRICS]` blocks |
| G4 | Integration tests (IT-1..IT-5) | ⚠️ PARTIAL — smoke tested; formal IT-N suite not run |
| G5 | Full matrix executed | ⚠️ PARTIAL — 24/243 planned points; S2 and 8-worker deferred |
| G6 | Consolidated dataset with no duplicates and no critical missing metrics | ✅ PASS — 24 rows, all fields present, all verifications SUCCESSFUL |
| G7 | Analysis tables answer core questions | ✅ PASS — all 5 sub-questions covered |
| G8 | Phase 4 artifact document produced | ✅ PASS — this document |

---

## 9. Files Produced

| File | Description |
|---|---|
| `TCC/artifacts/phase4/PHASE4_ARTIFACT.md` | This document |
| `TCC/artifacts/phase4/analysis/results_raw.csv` | Flat dataset, one row per run |
| `TCC/artifacts/phase4/analysis/summary.md` | Formatted tables (generated by analyze.py) |
| `TCC/artifacts/phase4/analysis/analysis_report.md` | Full English analysis report |
| `TCC/artifacts/phase4/analysis/analysis_report_ptbr.md` | PT-BR professor update |
| `TCC/artifacts/phase4/analysis/plots/fig1_wall_time.png` | Wall time figure |
| `TCC/artifacts/phase4/analysis/plots/fig2_mana_overhead.png` | MANA overhead figure |
| `TCC/artifacts/phase4/analysis/plots/fig3_recovery_phases.png` | Recovery phases figure |
| `TCC/artifacts/phase4/analysis/plots/fig4_scalability.png` | Scalability figure |
| `TCC/artifacts/phase4/analysis/plots/fig5_cost.png` | Cost comparison figure |
| `TCC/artifacts/phase4/analysis/plots/fig6_replace_vs_degraded.png` | REPLACE vs DEGRADED figure |
| `TCC/analyze.py` | Analysis pipeline script |

---

## 10. Known Limitations and Future Work

1. **Single-run observations:** All 24 data points are single runs. Standard deviation
   cannot be computed. Reported values should be treated as indicative, not statistically
   confirmed. Recommended next step: repeat each configuration 3–5 times.

2. **CG 2-worker MANA overhead outlier:** The 3.13× value at CG/2w is suspected to be
   a transient event (slow checkpoint write, OS scheduling jitter on a ~34 s job). This
   specific configuration should be prioritized for re-runs.

3. **8-worker cluster:** Pending AWS EIP quota increase in us-west-2. Adding 8-worker
   runs would strengthen the scalability conclusions.

4. **Single failure per run (S1 only):** The S2 scenario (two sequential failures on
   different workers) was not tested. It is relevant for assessing REPLACE_RESUME's
   robustness to repeated interruptions within a single job.

5. **Longer workloads:** LU Class D and EP Class E were not tested. These would be
   important for validating the economic argument at larger scale — the spot discount
   becomes more dominant for longer jobs.

---

## 11. File References

- Phase 4 plan: `TCC/artifacts/phase4/PHASE4_PLAN.md`
- Phase 3 artifact: `TCC/artifacts/phase3/PHASE3_ARTIFACT.md`
- Analysis reports: `TCC/artifacts/phase4/analysis/analysis_report.md` (EN),
  `analysis_report_ptbr.md` (PT-BR)
- Project master plan: `TCC/TCC_MANA_INTEGRATION_PLAN.md`
