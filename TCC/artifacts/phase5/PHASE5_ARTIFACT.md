# Phase 5 Artifact — Deepening the Analysis
### Synthetic Studies · Failure Timing Sensitivity · Enhanced Phase Instrumentation

Date: 2026-06-07
Project: Integrating MANA Fault Tolerance into HPC@Cloud for AWS Spot Clusters
Repository: hpcac-toolkit

---

## 1. Purpose

This artifact records the completed Phase 5 work: three focused studies that deepen
the Phase 4 pilot results by adding controlled causal experiments (synthetic programs),
broader failure timing coverage (10% / 25% / 50% of job lifetime), an expanded cluster
size (8 workers), and finer-grained phase instrumentation (Phase 2 split into 2a/2b/2c).

**Phase 5 research questions addressed:**

| Question | Study | Answer |
|---|---|---|
| What drives MANA checkpoint overhead — call frequency? | 5.1 synth_calls/synth_p2p | No — overhead is flat (~3 s fixed cost) |
| Does MPI_Wait imbalance explain CG/LU overhead? | 5.1 synth_imbalanced | No — spin doesn't add wall time proportionally |
| How does checkpoint image size affect Phase 1 time? | 5.2 synth_ckpt | Scales with memory; P2 has a hard floor of ~21.6 s |
| When does REPLACE beat DEGRADED? | 5.3 timing variants | Only EP-D at 2w, failure at 10% of run |
| How does cluster size affect the strategy trade-off? | 5.3 × 8w | DEGRADED advantage grows with cluster size |

---

## 2. Scope Alignment (Planned vs Actual)

| Dimension | Planned | Actual |
|---|---|---|
| Benchmarks | EP, LU (timing); CG (base trigger); synth (overhead) | All as planned |
| Cluster sizes | 2w, 4w, 8w | All as planned — 8w included (EIP quota resolved) |
| Failure timing points | 10%, 25%, 50% for EP + LU | All 3 points × 2 benchmarks × 3 sizes = 36 runs ✅ |
| Synthetic programs | synth_calls, synth_p2p, synth_imbalanced, synth_ckpt | All 4 implemented and run ✅ |
| Synth levels | 5 levels × 2 strategies for calls/p2p/imbalanced; 4 sizes for ckpt | All as planned ✅ |
| Phase 2 instrumentation | Split phase2 → phase2a + phase2b + phase2c | Done in watcher.rs + run_task.rs ✅ |
| Result file naming | `{task_tag}_{timestamp}_SUCCESS.txt` | Done in run_task.rs ✅ |
| Repetitions | 1 per cell | 1 per cell (variance estimated from 3 timing variants per FT config) |
| Total runs | 94 | **94** — all completed successfully |

---

## 3. Cluster and Environment Profile

- **AWS Region:** us-west-2
- **Availability Zone:** us-west-2a
- **Head node:** t3.large, on-demand
- **Worker nodes:** m5.xlarge, spot (4 vCPUs, 16 GB RAM)
- **AMI (all nodes):** ami-05eca8f13c2934abf (`mana-worker-fix-20260531`)
- **EFS:** enabled, mounted at `/shared`; checkpoint directory: `/shared/checkpoints`
- **Slurm:** managed by hpcac-toolkit watcher; 2 MPI processes per worker node
- **Pricing used in cost analysis:**
  - m5.xlarge spot: $0.0585/hr (~70% discount vs on-demand $0.1920/hr)
  - t3.large on-demand: $0.0832/hr
- **Run dates:** 2026-05-30 to 2026-06-03

---

## 4. New Infrastructure Implemented in Phase 5

### 4.1 Phase 2 Sub-Phase Split (`watcher.rs` + `run_task.rs`)

Phase 2 (node recovery) was previously reported as a single value. Phase 5 splits it
into three measurable sub-phases with distinct event boundaries:

| Sub-phase | Boundary events | Driver | Typical duration |
|---|---|---|---|
| **phase2a** | `checkpoint_completed` → `node_drained` | Slurm drain + scancel | ~5.5 s (both strategies) |
| **phase2b** | `node_drained` → `node_became_idle` | EC2 provisioning (REPLACE) or `scontrol DOWN` (DEGRADED) | ~122–157 s (REP) / ~11 s (DEG) |
| **phase2c** | `node_became_idle` → `restart_dispatched` | New salloc + MANA coordinator setup | ~5.3 s (both strategies) |

The `node_became_idle` event was added to DEGRADED (it already existed for REPLACE),
making the phase2b/2c boundary symmetric between strategies.

New `[RUN_METRICS]` fields: `phase2a_s`, `phase2b_s`, `phase2c_s` (sum = `phase2_s`,
kept for backward compatibility with Phase 4 data).

### 4.2 Result File Naming

Result files are now named `{task_tag}_{timestamp}_SUCCESS.txt` on successful
completion (vs `{task_tag}_{timestamp}.txt` for failed/interrupted runs). This allows
instant identification of valid results from the filesystem without parsing file content.

`analyze.py`'s `rglob("*.txt")` naturally picks up both; only files with a valid
`[RUN_METRICS]` block with `status=SUCCESS` contribute to the dataset.

### 4.3 Auto-Failure Bug Fix

`simulate_cluster_failure` in `resource_manager.rs` previously resolved the target
EC2 instance ID *after* sleeping `warning_time_secs`, which could terminate a newly
respawned replacement instance if recovery completed within the warning window.
Fixed to resolve the instance ID *before* sleeping — only the originally targeted
instance is ever terminated regardless of recovery duration.

### 4.4 Synthetic C Programs

Four programs written in `TCC/synthetic/`:

| Program | MPI pattern | Variable | Levels |
|---|---|---|---|
| `synth_mpi_calls.c` | MPI_Allreduce collective | Call frequency | L0–L4: 0 / 800 / 3200 / 12800 / 51200 calls |
| `synth_p2p.c` | MPI_Send + MPI_Recv (synchronized) | Call frequency | L0–L4: same as above |
| `synth_imbalanced.c` | MPI_Irecv + MPI_Send + MPI_Wait | Sender delay (µs) | L0–L4: 0 / 100 / 1000 / 5000 / 20000 µs |
| `synth_checkpoint_size.c` | MPI_Barrier (minimal) | Memory per process (MB) | 50 / 200 / 800 / 3200 MB |

All synth programs target ~60–120 s noFT runtime to ensure stable overhead measurement.

### 4.5 Enhanced `analyze.py`

`TCC/analyze.py` was significantly rewritten for Phase 5:

- **`parse_task_tag()`**: extended with regex patterns for synth tags
  (`synth_ckpt-{N}mb_...`, `synth_{type}-l{N}_...`) before the NPB pattern
- **Phase fields**: reads `phase2a_s`, `phase2b_s`, `phase2c_s`, `phase0_s` from
  `[RUN_METRICS]`; handles old files missing `phase2c_s` gracefully (None/NaN)
- **`_draw_phase_bars()`**: shared helper for 5-segment stacked bars (P0/P1/Slurm/P2b/P3)
  used by both fig4 and fig5; annotations placed inside bars to avoid overlap

**10 figures produced** (vs 6 in Phase 4):

| Figure | Content |
|---|---|
| `fig1_mana_overhead.png` | MANA overhead % per benchmark and worker count |
| `fig2_synth_calls.png` | Absolute elapsed time vs MPI call level (synth_calls + synth_p2p) |
| `fig2b_synth_imbalanced.png` | Elapsed time vs sender delay level (synth_imbalanced, MANA overhead under MPI_Wait spin) |
| `fig3_synth_ckpt.png` | Phase 1 time vs checkpoint image size (scatter + linear fit) |
| `fig4_timing_phases.png` | Full FT wall time breakdown (P0/P1/Slurm/P2b/P3) by failure timing |
| `fig5_cg_short_job.png` | CG-C phase breakdown — recovery overhead dominates short jobs |
| `fig6_mana_scalability.png` | Wall time vs worker count (strong scaling, noFT + MANA-noFT) |
| `fig7_cost.png` | Cost per run — spot with FT vs on-demand without FT |
| `fig8_strategy_comparison.png` | Total FT wall time: MANA-noFT baseline + REPLACE + DEGRADED |
| `fig9_recovery_overhead_ratio.png` | Recovery overhead as % of total FT wall time |

---

## 5. Experiment Execution Matrix

### 5.1 NPB Baselines + CG FT (Phase 4 Redo)

All runs with new infrastructure (phase2a/2b/2c, task_tag filenames).

| Benchmark | Workers | noFT | MANA-noFT | REPLACE | DEGRADED |
|---|---|---|---|---|---|
| CG-C | 2 | ✅ | ✅ | ✅ (trigger 3 s) | ✅ (trigger 3 s) |
| CG-C | 4 | ✅ | ✅ | ✅ (trigger 3 s) | ✅ (trigger 3 s) |
| CG-C | 8 | ✅ | ✅ | ✅ (trigger 3 s) | ✅ (trigger 3 s) |
| EP-D | 2 | ✅ | ✅ | — (covered by 5.3) | — (covered by 5.3) |
| EP-D | 4 | ✅ | ✅ | — | — |
| EP-D | 8 | ✅ | ✅ | — | — |
| LU-C | 2 | ✅ | ✅ | — | — |
| LU-C | 4 | ✅ | ✅ | — | — |
| LU-C | 8 | ✅ | ✅ | — | — |

**Subtotal: 24 runs**

### 5.2 Phase 5.1 — Synthetic MPI Overhead (2 workers only)

| Program | Levels | noFT | MANA-noFT |
|---|---|---|---|
| synth_calls | L0–L4 (0/800/3200/12800/51200 calls) | ✅ × 5 | ✅ × 5 |
| synth_p2p | L0–L4 (same) | ✅ × 5 | ✅ × 5 |
| synth_imbalanced | L0–L4 (0/100µs/1ms/5ms/20ms delay) | ✅ × 5 | ✅ × 5 |

**Subtotal: 30 runs**

### 5.3 Phase 5.2 — Synthetic Checkpoint Size (2 workers, DEGRADED only)

| Memory/process | trigger_after_secs | Status |
|---|---|---|
| 50 MB | 30 s | ✅ |
| 200 MB | 30 s | ✅ |
| 800 MB | 30 s | ✅ |
| 3200 MB | 30 s | ✅ |

**Subtotal: 4 runs**

### 5.4 Phase 5.3 — Failure Timing Sensitivity (EP + LU, all worker counts)

Trigger times based on 10% / 25% / 50% of MANA-noFT wall time.

| Workers | Benchmark | Trigger 10% | Trigger 25% | Trigger 50% | REPLACE ×3 | DEGRADED ×3 |
|---|---|---|---|---|---|---|
| 2w | EP-D | 46 s | 116 s | 231 s | ✅ | ✅ |
| 2w | LU-C | 24 s | 59 s | 118 s | ✅ | ✅ |
| 4w | EP-D | 32 s | 81 s | 151 s | ✅ | ✅ |
| 4w | LU-C | 13 s | 33 s | 66 s | ✅ | ✅ |
| 8w | EP-D | 12 s | 31 s | 61 s | ✅ | ✅ |
| 8w | LU-C | 7 s | 17 s | 34 s | ✅ | ✅ |

**Subtotal: 36 runs**

### 5.5 Grand Total

| Group | Runs | Valid |
|---|---|---|
| NPB baselines + CG FT (Phase 4 redo) | 24 | 24 |
| Synthetic MPI overhead (5.1) | 30 | 30 |
| Synthetic checkpoint size (5.2) | 4 | 4 |
| Failure timing sensitivity (5.3) | 36 | 36 |
| **Total** | **94** | **94** |

All 94 runs completed with NPB `Verification SUCCESSFUL` or synth program exit code 0.
Zero invalid runs in the Phase 5 dataset.

---

## 6. Results Summary

### 6.1 MANA Overhead Without Failures

| Benchmark | 2 workers | 4 workers | 8 workers |
|---|---|---|---|
| CG-C | +233% ⚠ (anomalous — cascading imbalance hypothesis) | +59% | +73% |
| EP-D | +39% (single run, EC2 variance likely) | +14% | +22% |
| LU-C | +62% | +56% | +52% |

LU-C shows the most consistent and representative overhead: ~52–62% stable across all
cluster sizes. EP overhead is unreliable from single runs. CG-2w is an anomaly
confirmed to not be a general MANA weakness (CG-4w and CG-8w are consistent with LU).

### 6.2 Synthetic Studies Key Findings

| Study | Finding |
|---|---|
| synth_calls + synth_p2p | Overhead ~3.3–3.5 s flat regardless of call count (0 to 51200). Call frequency is **not** the overhead driver. |
| synth_imbalanced | Overhead flat ~3.4 s at low delay; approaches 0 at high delay (noFT slows down to meet MANA's floor). MPI_Wait spin time is **not** the overhead driver in isolation. |
| synth_ckpt | Phase 1 scales with memory: 16 s at 50 MB → 139 s at 3200 MB. Phase 2 constant at ~21.6 s (Slurm reconfiguration floor — cannot be reduced without redesign). |

### 6.3 Phase 2b Comparison (the Strategy-Defining Phase)

| Strategy | Phase 2b (node reconfig) | Mechanism |
|---|---|---|
| REPLACE | 122–157 s across all configs | AWS EC2 terminate → respawn → boot → AMI init → Slurm join |
| DEGRADED | ~11 s across all configs | `scontrol update state=DOWN` |
| **Difference** | **~111–146 s** | Fixed cloud provisioning latency, irreducible without pre-warmed pools |

### 6.4 Failure Timing Sensitivity — REPLACE vs DEGRADED

**EP-D total FT wall time (seconds):**

| Workers | Timing | REPLACE | DEGRADED | Winner |
|---|---|---|---|---|
| 2w | 10% (46 s) | 519 | 537 | REPLACE (−18 s) |
| 2w | 25% (116 s) | 549 | 533 | DEGRADED (−16 s) |
| 2w | 50% (231 s) | 588 | 538 | DEGRADED (−50 s) |
| 4w | 10% (32 s) | 314 | 289 | DEGRADED (−25 s) |
| 4w | 25% (81 s) | 385 | 295 | DEGRADED (−90 s) |
| 4w | 50% (151 s) | 375 | 296 | DEGRADED (−79 s) |
| 8w | avg | 307 ±51 | 177 ±4 | DEGRADED (≈−130 s) |

**LU-C total FT wall time (seconds, averages across timing points):**

| Workers | REPLACE avg | DEGRADED avg | DEGRADED advantage |
|---|---|---|---|
| 2w | 349 ±20 | 302 ±2 | −47 s |
| 4w | 277 ±6 | 194 ±3 | −83 s |
| 8w | 303 ±22 | 140 ±6 | −163 s |

**Verdict: DEGRADED wins in 17 of 18 tested configurations.** The single exception is
EP-D at 2 workers, failure at 10% — the only case where the 50% capacity loss from
dropping one of two workers produces a P3 penalty larger than REPLACE's ~123 s P2b cost.

### 6.5 CG Short-Job Case

For CG-C (noFT ~12–35 s depending on workers), recovery overhead dominates total wall
time. DEGRADED still wins over REPLACE by 78–105 s at all cluster sizes, but both
strategies take 8–17× longer than the original job.

| Workers | REPLACE | DEGRADED | noFT reference |
|---|---|---|---|
| 2w | 243.8 s | 165.8 s | 34.6 s |
| 4w | 201.1 s | 100.9 s | 18.7 s |
| 8w | 203.1 s | 98.3 s | 12.1 s |

### 6.6 Economic Analysis

For medium-to-long jobs, spot workers + DEGRADED saves 31–78% vs on-demand + noFT.
CG is the exception: job too short for spot discount to offset recovery overhead.

| Benchmark | Workers | noFT on-demand | DEGRADED spot | Saving |
|---|---|---|---|---|
| CG-C | 4w | $0.0044 | $0.0089 | −102% (more expensive) |
| LU-C | 4w | $0.0309 | $0.0171 | **+45%** |
| EP-D | 4w | $0.0676 | $0.0259 | **+62%** |
| EP-D | 8w | $0.1244 | $0.0271 | **+78%** |

---

## 7. Acceptance Gates

| Gate | Description | Status |
|---|---|---|
| G1 | synth_calls + synth_p2p compile and run under noFT and MANA | ✅ PASS |
| G2 | Overhead vs call rate measured — result: flat, call count is not the driver | ✅ PASS |
| G3 | synth_ckpt produces measurable phase1_s variation (16–139 s across 50–3200 MB) | ✅ PASS |
| G4 | Phase 1 vs size measured — non-linear (fixed overhead + bandwidth cap) | ✅ PASS |
| G5 | Failure timing runs complete for EP and LU at all 3 points (2w+4w+8w) | ✅ PASS — 36 runs done |
| G6 | [RUN_METRICS] blocks contain phase2a_s, phase2b_s, phase2c_s | ✅ PASS |
| G7 | fig4 shows all 5 segments (P0/P1/Slurm/P2b/P3) | ✅ PASS |
| G8 | fig8 shows REPLACE vs DEGRADED vs MANA-noFT baseline across failure timings | ✅ PASS |
| G9 | Phase 5 artifact document produced | ✅ PASS — this document |

---

## 8. Files Produced

### Analysis Outputs

| File | Description |
|---|---|
| `TCC/artifacts/phase5/analysis/results_raw.csv` | Flat dataset, one row per run (94 rows) |
| `TCC/artifacts/phase5/analysis/summary.md` | Aggregated tables per benchmark/strategy/workers |
| `TCC/artifacts/phase5/analysis/analysis_report_phase5.md` | Full English analysis report with figures |
| `TCC/artifacts/phase5/analysis/analysis_report_phase5_ptbr.md` | PT-BR analysis report (simplified language) |

### Figures

| File | Content |
|---|---|
| `plots/fig1_mana_overhead.png` | MANA overhead % per benchmark and worker count |
| `plots/fig2_synth_calls.png` | Elapsed time vs call level for synth_calls + synth_p2p |
| `plots/fig2b_synth_imbalanced.png` | Elapsed time vs sender delay for synth_imbalanced |
| `plots/fig3_synth_ckpt.png` | Phase 1 time vs checkpoint image size |
| `plots/fig4_timing_phases.png` | Full FT wall time breakdown by failure timing (EP + LU) |
| `plots/fig5_cg_short_job.png` | CG-C phase breakdown (REPLACE vs DEGRADED, all worker counts) |
| `plots/fig6_mana_scalability.png` | Strong scaling — wall time vs worker count |
| `plots/fig7_cost.png` | Cost per run — spot+FT vs on-demand+noFT |
| `plots/fig8_strategy_comparison.png` | Total wall time: MANA-noFT + REPLACE + DEGRADED by timing |
| `plots/fig9_recovery_overhead_ratio.png` | Recovery overhead as % of total FT wall time |

### Code Changes

| File | Change |
|---|---|
| `src/commands/cluster/watcher.rs` | Added `node_became_idle` event to DEGRADED path for symmetric phase2c |
| `src/commands/cluster/run_task.rs` | phase2a/2b/2c fields; task_tag filename; auto-failure instance ID fix |
| `src/commands/cluster/resource_manager.rs` | Auto-failure bug fix: capture instance ID before warning sleep |
| `TCC/analyze.py` | Full rewrite of parsing and plotting for Phase 5 fields and 9 figures |
| `TCC/synthetic/synth_mpi_calls.c` | Synthetic MPI collective overhead program |
| `TCC/synthetic/synth_p2p.c` | Synthetic P2P overhead program |
| `TCC/synthetic/synth_imbalanced.c` | Synthetic MPI_Wait imbalance program |
| `TCC/synthetic/synth_checkpoint_size.c` | Synthetic checkpoint size program |

### Task YAML Files

| Location | Count | Description |
|---|---|---|
| `my_clusters/m5-xlarge/{2,4,8}workers/{ep,lu,cg}/` | 48 | NPB task files (noFT, MANA-noFT, FT variants) |
| `my_clusters/m5-xlarge/2workers/synth/` | 34 | Synthetic task files (all programs × all levels) |

---

## 9. Known Limitations and Future Work

1. **Single repetition per cell.** All 94 runs are single observations. Variance is
   estimated where timing variants provide 3 data points per FT config (EP and LU),
   but baseline configurations (noFT, MANA-noFT, CG FT) have no repetitions.
   Standard deviation cannot be computed for most cells.

2. **CG-2w overhead anomaly unresolved.** The 233% overhead is reproducible (also
   observed in Phase 4) but unexplained. The cascading CPU competition hypothesis
   would require per-rank instrumentation inside the NAS CG benchmark to confirm.

3. **Single failure per run.** The watcher handles one recovery cycle at a time
   (sequential). Multi-failure scenarios were not tested. This is a known architectural
   limitation documented as future work in the implementation.

4. **Only m5.xlarge instances.** Different instance types (more memory, GPU, HPC-optimized)
   would produce different Phase 1 times and MANA overhead factors.

5. **EFS as checkpoint store.** EFS introduces variable latency depending on burst
   credits. A high-performance parallel filesystem (Lustre via FSx) would reduce
   Phase 1 times substantially and remove the bandwidth saturation seen at 3200 MB.

6. **No longer workloads.** LU Class D and EP Class E were not tested. The economic
   argument strengthens further for longer jobs where spot discount dominates.

7. **No very long workloads tested.** LU Class D and EP Class E were not evaluated.
   The economic case and crossover point between strategies would be clearer with
   longer jobs where recovery overhead represents a smaller fraction of total time.

---

## 10. File References

- Phase 5 plan: `TCC/artifacts/phase5/PHASE5_PLAN.md`
- Phase 5 analysis report (EN): `TCC/artifacts/phase5/analysis/analysis_report_phase5.md`
- Phase 4 artifact: `TCC/artifacts/phase4/PHASE4_ARTIFACT.md`
- Analysis pipeline: `TCC/analyze.py`
- Project master plan: `TCC/TCC_MANA_INTEGRATION_PLAN.md`
