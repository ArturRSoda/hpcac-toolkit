# Phase 10 Artifact — The ADAPTIVE Recovery Strategy
### Heuristic Derivation · N-Independence Validation · Implementation · Integration Testing · Coefficient-Calibration Exploration

Date: 2026-10-06
Project: Integrating MANA Fault Tolerance into HPC@Cloud for AWS Spot Clusters
Repository: hpcac-toolkit

---

## 1. Purpose

This artifact records the completed Phase 10 work: implementing `ADAPTIVE`, a third
fault-tolerance strategy that chooses automatically between `REPLACE_RESUME` and
`DEGRADED_RESUME` at the moment of a fault, using a heuristic derived from the existing
18-point NPB dataset (Phase 5). The phase covers deriving and validating that heuristic,
implementing it in the watcher, testing it end-to-end against real AWS faults, and a
follow-up exploration (requested by the advisor) into whether the heuristic's
coefficient could be determined automatically per application instead of using a single
fixed value.

**Phase 10 research questions addressed:**

| Question | Section | Answer |
|---|---|---|
| Can a single time-based formula replace manual strategy choice? | Heuristic derivation | Yes — a coefficient (0.28) and a fixed 109s threshold classify the 18-point training set 18/18 |
| Does the naive "1/(N−1)" capacity model hold? | Heuristic derivation | No — it only gets 13/18 right; the real coefficient is nearly flat across N |
| Does the heuristic generalize across cluster sizes? | N-independence validation | Yes — confirmed on a held-out synthetic workload (`synth_duration`, 84 runs) at N=2/4/8 |
| Does the real implementation decide correctly on live AWS faults? | Integration testing | Yes — 2 unambiguous cases, 1 case near the formula's own threshold (3/3 correct) |
| Is the coefficient universal across workload profiles? | Integration testing | No — demonstrated directly: a memory-bandwidth-bound workload causes a ~5.8% suboptimal decision |
| Can the coefficient be determined automatically per application? | Coefficient-calibration exploration | Promising direction, not yet validated numerically — closed as future work per advisor's decision |

---

## 2. Scope Alignment (Planned vs Actual)

| Dimension | Planned | Actual |
|---|---|---|
| Heuristic derivation | Derive coefficient from existing 18-point NPB dataset | Done — coefficient 0.28, valid range (0.250, 0.300), 18/18 on training set |
| Implementation | New `ADAPTIVE` strategy in `run_task.rs` + `watcher.rs` | Done — `choose_effective_strategy()`, `estimated_duration_secs` config field, zero-worker guard |
| Smoke testing | End-to-end test on real AWS fault | Done (Etapa 10) — found and fixed a real `elapsed_secs` anchor bug |
| N-independence validation | NPB-only (1 extra point, EP-D f=15%) | **Scope changed**: Etapa 11.1 (NPB f=15%) cancelled — the data needed already existed in the Phase 5 dataset. Replaced with Etapa 11.2: a purpose-built synthetic benchmark (`synth_duration`), 14 configurations × N=2/4/8, 84 runs |
| Integration evaluation | Originally 32 configs × 3 reps = 96 runs | **Scope reduced** (Etapa 12, 2026-09-26): re-derived to 4 targeted cases (6 runs total) once it became clear the original scope was re-measuring already-known REPLACE/DEGRADED outcomes rather than testing new things. Zero-worker case covered by unit tests instead of an AWS cluster |
| Coefficient calibration | Not in original plan | **Added** (Etapa 13, 2026-10-06): advisor asked whether the coefficient could be derived from CPU/memory usage. Explored, tested, partially validated, closed as future work per advisor's decision |
| Consolidated report | Not in original plan | **Added**: full English + PT-BR analysis reports written and iteratively refined for clarity, in addition to the working decision log (`PHASE10_PLAN.md`) |

---

## 3. Cluster and Environment Profile

- **AWS Region:** us-west-2
- **Availability Zone:** us-west-2a
- **Head node:** t3.large, on-demand
- **Worker nodes:** m5.xlarge, spot (2 physical cores / 4 vCPUs via hyperthreading, 16 GB RAM)
- **AMI (all nodes):** ami-05eca8f13c2934abf
- **EFS:** enabled, mounted at `/shared`; checkpoint directory: `/shared/checkpoints`
- **Slurm:** managed by hpcac-toolkit watcher; 2 MPI processes per worker node (NPB, `synth_duration`)
- **Run dates:** 2026-09-12 to 2026-10-06

---

## 4. New Infrastructure Implemented in Phase 10

### 4.1 The `ADAPTIVE` Strategy (`run_task.rs` + `watcher.rs`)

- `fault_tolerance.strategy: "ADAPTIVE"` — new value alongside `NONE`, `REPLACE_RESUME`,
  `DEGRADED_RESUME`.
- `estimated_duration_secs: Option<u64>` — new optional config field (the user-provided
  `T_estimated`). Missing value falls back to `DEGRADED_RESUME` rather than guessing.
- `choose_effective_strategy(remaining_workers, run_started_at, estimated_duration_secs)`
  in `watcher.rs` — the decision function. Checks `remaining_workers == 0` first
  (forces `REPLACE_RESUME` unconditionally), then applies
  `remaining_secs × 0.28 > 109 → REPLACE_RESUME`, else `DEGRADED_RESUME`.
- `run_commands_started_at: Arc<Mutex<Option<String>>>` — shared timestamp, set at the
  start of `run_commands` (not watcher startup), used to anchor `elapsed_secs` correctly.
  Fixes a real bug found during smoke testing (Section 6.1 of the analysis report):
  anchoring to watcher startup included `setup_commands` time, inflating `elapsed_secs`
  beyond what `estimated_duration_secs` assumes.
- `InterruptionEvent` with `event_type="adaptive_decision"` — new persisted event
  recording which strategy `ADAPTIVE` chose and why, for auditability.
- The decision fires after the node is drained (`checkpoint_completed` → `node_drained`)
  and before dispatch, not at the instant the fault is detected — because the code only
  then knows which handler to call. This means `elapsed_secs` at decision time equals
  the fault trigger time plus checkpoint-write and drain duration (~20–40s in the
  configurations tested), a finding from the Etapa 12 boundary-case test.

### 4.2 `synth_duration` Synthetic Benchmark (`TCC/synthetic/synth_duration.c`)

Built because no real NPB configuration other than EP-D at N=2 ever crosses the 389s
decision threshold, making it impossible to test N-independence at N=4/N=8 with real
benchmarks. Each process allocates 50 MB (for a realistic checkpoint image, same
technique as `synth_checkpoint_size` from Phase 5) and runs a **fixed iteration count**
(not fixed wall-clock duration — an earlier wall-clock-loop design was found to be
unusable, since `MPI_Wtime()`-based termination doesn't slow down under CPU
oversubscription). Calibrated on m5.xlarge: 129.5×10⁹ iterations ≈ 600s on one
non-oversubscribed core.

Not designed to be memory-bandwidth-bound on purpose — that emerged as a side effect of
touching the 50MB buffer every iteration, and turned out to be central to the phase's
findings about workload-dependent coefficients.

### 4.3 Coefficient-Calibration Exploration Kernels (`TCC/synthetic/compute_bound.c`, `memory_bound.c`)

Two small standalone (non-MPI) programs built for Etapa 13, to test whether the
oversubscription penalty could be measured directly and cheaply (no hardware
performance counters, no multi-worker cluster) instead of inferred from indirect
CPU/memory metrics. `compute_bound.c` is a register-only floating-point recurrence
(EP-D proxy); `memory_bound.c` is the same loop as `synth_duration.c`, minus MPI.

### 4.4 Analysis Pipeline (`TCC/artifacts/phase10/analysis/`)

- `parse_synth_duration.py` — parses all 85 `synth_duration` result files into
  `synth_duration_raw.csv`.
- `analyze_synth_duration.py` — computes per-point and per-N coefficient statistics
  (`synth_duration_wide.csv`, `coefficient_per_n.csv`) and generates two figures:
  `fig_synth_duration_boundary.png` (measured REPLACE/DEGRADED winner across N=2/4/8)
  and `fig_coefficient_n_independence.png` (measured coefficient vs. the naive
  1/(N−1) model).
- `analysis_report_phase10.md` / `analysis_report_phase10_ptbr.md` — the consolidated
  findings report (English and Portuguese), iteratively restructured for logical flow
  (N-independence validation moved before integration testing, so nothing is used
  before it's introduced) and density (long paragraphs split, dense explanations
  rewritten with concrete worked examples).

---

## 5. Execution Matrix

### 5.1 Heuristic Derivation (no new runs — reused Phase 5 dataset)

| Source | Configurations | Runs | Use |
|---|---|---|---|
| NPB (EP-D, LU-C × N=2/4/8 × f=10/25/50%) | 18 | 108 (existing, 3 reps each) | Train the 0.28 coefficient; ground truth for REPLACE/DEGRADED winner |

### 5.2 N-Independence Validation (Etapa 11.2 — `synth_duration`)

| N | f% tested | Configs | Runs (3 reps × 2 strategies) |
|---|---|---|---|
| 2 | 5, 30, 40, 50, 65, 75% | 6 | 36 |
| 4 | 5, 50, 65, 75% | 4 | 24 |
| 8 | 5, 50, 65, 75% | 4 | 24 |
| **Total** | | **14** | **84** |

### 5.3 Integration Testing (Etapa 12 — reduced scope)

| Case | Config | Runs |
|---|---|---|
| Clear REPLACE | EP-D, N=2 | 1 |
| Clear DEGRADED | EP-D, N=4 | 1 |
| Near formula threshold | EP-D, N=2, calibrated to `remaining_secs≈389s` | 3 |
| Coefficient-mismatch demonstration | `synth_duration`, N=2, f=50% | 1 |
| Zero-worker guard | Unit tests (`cargo test`), no AWS cost | 5 tests |
| **Total new AWS runs** | | **6** |

### 5.4 Coefficient-Calibration Exploration (Etapa 13)

| Test | Config | Runs |
|---|---|---|
| `perf stat` feasibility | m5.xlarge, hardware counters | 1 (manual check, not a task run) |
| Density calibration, idle sibling core | `compute_bound` + `memory_bound`, densities 1–4 | 1 task (16 process-executions) |
| Density calibration, busy sibling core (turbo-boost control) | Same, with decoy load on the other core | 1 task (16 + 2 decoy process-executions) |

### 5.5 Grand Total

| Group | Runs |
|---|---|
| NPB training set (existing, reused) | 108 |
| `synth_duration` N-independence validation | 84 |
| ADAPTIVE integration testing (AWS) | 6 |
| Zero-worker guard (unit tests) | 5 tests |
| Coefficient-calibration exploration | 2 tasks |
| **New AWS runs in Phase 10** | **92** |

---

## 6. Results Summary

### 6.1 The Heuristic

A single empirical coefficient (0.28, valid range 0.250–0.300 against the training set)
and a fixed 109s threshold predict the correct recovery strategy without needing to
know the cluster size:

```
remaining_secs = T_estimated − elapsed_secs
REPLACE_RESUME  when  remaining_secs × 0.28 > 109
DEGRADED_RESUME when  remaining_secs × 0.28 ≤ 109
```

The naive "1/(N−1)" capacity model was tested first and only scored 13/18. Two
compensating effects explain the flat, N-independent coefficient: REPLACE's P3
consistently runs faster than a cold-start model predicts (pulls the coefficient up),
and DEGRADED's oversubscription penalty is smaller than linear scaling assumes, because
the surviving worker's idle hardware threads absorb much of the extra load (pulls the
coefficient down, more strongly).

### 6.2 N-Independence (`synth_duration`)

| N | Measured coefficient | Naive model 1/(N−1) |
|---|---|---|
| 2 | 0.506 ± 0.020 | 1.000 |
| 4 | 0.519 ± 0.017 | 0.333 |
| 8 | 0.523 ± 0.031 | 0.143 |

The crossover sits at the same `remaining_secs≈210s` (f≈65%) for N=2, 4, and 8 — the
only point in the entire 14-config matrix where REPLACE/DEGRADED time ranges genuinely
overlap, at any N. A concrete, code-verified factor (integer rounding in
`ntasks_per_node = ceil(process_count / worker_count)`) explains why N=4 and N=8 land
on nearly the same coefficient; it does not explain why N=2 — the heaviest
oversubscription of the three — ends up with the *lowest* coefficient. That remains an
open, honestly flagged question.

### 6.3 Integration Testing

All cases matched their prediction. The coefficient-mismatch case is the most
important result: `ADAPTIVE` run on `synth_duration` (N=2, f=50%) chose
`DEGRADED_RESUME` exactly as its formula predicts (penalty_est≈73–78, under the 109s
threshold) — but the known ground truth shows REPLACE is the true winner there by a
clear margin. ADAPTIVE's actual result (666.1s) landed in the DEGRADED range, not the
faster REPLACE range (629.7s), a **36.4s (~5.8%) cost**. The implementation is not at
fault — it applied its formula exactly as designed. This is a demonstrated, quantified
limitation of the model itself, not inferred from a hunch.

### 6.4 Coefficient-Calibration Exploration (Etapa 13)

- `perf stat` hardware performance counters are **not available** on m5.xlarge (or any
  non-`.metal` AWS instance type) — confirmed directly, not assumed. `perf list hw` and
  `perf list cache` return empty; every counter reports `<not supported>`.
- An alternative requiring no hardware counters was derived analytically: measure the
  throughput ratio `r` when 2 processes share one physical core (exactly what N=2
  DEGRADED creates), then `coef ≈ (1-r)/r`. Checks out algebraically against known
  endpoints (`r=0.5` → `coef=1.0`, the naive model; `r=1` → `coef=0`, no penalty).
- Tested on two synthetic kernels at densities 1–4 processes per core. Results were
  cleanly interpretable: the compute-bound kernel showed zero penalty until exceeding 2
  processes (the physical hyperthread limit), then penalty matching a simple `2/N`
  division almost exactly; the memory-bound kernel showed penalty starting immediately
  at density 2, following `1/N` throughout — consistent with it already saturating the
  path to memory with a single process.
- The predicted coefficient from this test (`≈0.95` for the memory-bound kernel) did
  **not** match the known ground-truth coefficient for `synth_duration` (`≈0.5`). A
  follow-up control test (keeping the sibling physical core busy throughout, to rule
  out a turbo-boost confound inflating the idle-core baseline) showed **no measurable
  difference** (<1% across all densities) — that specific hypothesis is ruled out with
  evidence, not left as a guess.
- Advisor's decision (2026-10-06, via Slack): **stop here, sufficient for the TCC.**
  Documented as a promising, partially-validated direction for future work rather than
  a finished methodology.

---

## 7. Acceptance Gates

| Gate | Description | Status |
|---|---|---|
| G1 | Heuristic formula validated empirically against all 18 points of the training CSV; `T_MANA_noFT` extracted and documented | ✅ PASS |
| G2 | `cargo build --release` passes with no errors after all code changes | ✅ PASS |
| G3 | Smoke test: `ADAPTIVE` chooses correctly in both scenarios on real AWS | ✅ PASS (after fixing the `elapsed_secs` anchor bug) |
| G4 | Etapa 11.2: N-independence confirmed at N=2, 4, 8 (14 configs × 3 reps = 84 runs) | ✅ PASS |
| G5 | Integration evaluation (reduced scope): clear cases, boundary case (3/3 correct), zero-worker guard, coefficient-mismatch demonstration | ✅ PASS |
| G6 | Etapa 13: `perf stat` tested and ruled out with evidence; `coef=(1-r)/r` derived and tested on 2 kernels; turbo-boost hypothesis tested and ruled out; advisor's decision to close the exploration | ✅ PASS |
| G7 | Phase 10 artifact document produced | ✅ PASS — this document |

---

## 8. Files Produced

### Analysis Outputs

| File | Description |
|---|---|
| `TCC/artifacts/phase10/analysis/synth_duration_raw.csv` | Flat dataset, one row per `synth_duration` run (85 rows) |
| `TCC/artifacts/phase10/analysis/synth_duration_wide.csv` | Per-configuration REPLACE/DEGRADED comparison, coefficient, winner, overlap |
| `TCC/artifacts/phase10/analysis/coefficient_per_n.csv` | Coefficient mean ± std per cluster size |
| `TCC/artifacts/phase10/analysis/analysis_report_phase10.md` | Full English analysis report |
| `TCC/artifacts/phase10/analysis/analysis_report_phase10_ptbr.md` | PT-BR analysis report |
| `TCC/artifacts/phase10/heuristic.md` | Formal heuristic documentation (formula, edge cases) |

### Figures

| File | Content |
|---|---|
| `imgs/fig_adaptive_boundary.png` | Decision boundary against the 18-point NPB training set |
| `imgs/fig_synth_duration_boundary.png` | Measured REPLACE/DEGRADED winner across N=2/4/8 (`synth_duration`) |
| `imgs/fig_coefficient_n_independence.png` | Measured coefficient vs. the naive 1/(N−1) model |

### Code Changes

| File | Change |
|---|---|
| `src/commands/cluster/run_task.rs` | `estimated_duration_secs` field; `"ADAPTIVE"` validation; `run_commands_started_at` shared timestamp |
| `src/commands/cluster/watcher.rs` | `choose_effective_strategy()`; `ADAPTIVE` dispatch branch; `adaptive_decision` event; zero-worker guard |
| `TCC/synthetic/synth_duration.c` | Fixed-iteration-count synthetic benchmark for N-independence testing |
| `TCC/synthetic/compute_bound.c` | Compute-bound kernel for the coefficient-calibration exploration (Etapa 13) |
| `TCC/synthetic/memory_bound.c` | Memory-bound kernel for the coefficient-calibration exploration (Etapa 13) |
| `TCC/synthetic/Makefile` | Build target for `synth_duration` |

### Task YAML Files

| Location | Count | Description |
|---|---|---|
| `my_clusters/m5-xlarge/2workers/ep/task-ep_D-adaptive-smoketest-*.yaml` | 3 | `ADAPTIVE` smoke/boundary tests |
| `my_clusters/m5-xlarge/4workers/ep/task-ep_D-adaptive-smoketest-degraded.yaml` | 1 | Clear-DEGRADED integration test |
| `my_clusters/m5-xlarge/{2,4,8}workers/synth/task-synth_duration-f<F>pct-<strategy>.yaml` | 16 | `synth_duration` N-independence matrix |
| `my_clusters/m5-xlarge/2workers/synth/task-synth_duration-f50pct-adaptive.yaml` | 1 | Coefficient-mismatch demonstration |
| `my_clusters/m5-xlarge/2workers/synth/task-calib-core-sharing.yaml` | 1 | Etapa 13 density-calibration test (both idle- and busy-sibling-core variants) |

---

## 9. Known Limitations and Future Work

1. **Validation is confined to short-to-medium jobs.** Every run behind this heuristic
   has `remaining_secs` at most ~717s; multi-hour production jobs were never tested.
   The linear `penalty ≈ coef × remaining_secs` relationship has a plausible structural
   justification for holding at larger scale (it is a per-unit-time capacity ratio, not
   tied to absolute duration), but that is an argument for extrapolation, not evidence
   of it.

2. **The coefficient is not workload-universal.** Calibrated on register/compute-bound
   NPB benchmarks (EP-D, LU-C). A memory-bandwidth-bound workload has a measurably
   different true coefficient (~0.5 vs. 0.28), demonstrated to cause a real, quantified
   suboptimal decision (~5.8% slower than optimal) when the shipped formula is applied
   to such a workload.

3. **The mechanisms behind the coefficient are inferred, not directly measured, and
   incomplete.** Why the coefficient differs in magnitude between NPB and
   `synth_duration` (hyperthreading absorbing contention differently), and why it stays
   flat across N instead of declining, are both observed patterns, not confirmed with
   hardware performance counters (which, per Etapa 13, are not available on these
   instances anyway).

4. **Automatic per-application coefficient calibration (Etapa 13) is a promising,
   partially-validated direction, not a finished methodology.** Concrete next steps
   identified but not executed: test with real EP-D/LU-C binaries instead of proxy
   kernels (smaller NPB class, to keep calibration short); test a memory-bound decoy
   load on the sibling core instead of a compute-bound one, to isolate whole-chip
   memory-bandwidth contention specifically; isolate the warm-restart effect's own
   contribution to the real measured coefficient, since the clean microbenchmark has
   no checkpoint/restart step at all.

5. **`T_estimated` must be supplied by the user.** An explicit scope decision made
   before implementation, not a gap discovered during testing. Historical-execution-based
   estimation is future work.

6. **Multiple successive failures were not tested** for `ADAPTIVE` specifically. The
   formula recomputes `remaining_secs` from scratch at each fault, so it adapts
   automatically as elapsed time grows, but this was not exercised experimentally. This
   also bounds Etapa 13's calibration method specifically: it was only derived and
   tested for the single-fault, 2-processes-per-core regime; accumulated failures that
   push oversubscription past 2-per-core enter a qualitatively different regime (OS
   time-slicing instead of hyperthread sharing) with no calibration or formula of its
   own yet.

---

## 10. File References

- Phase 10 plan (full decision log): `TCC/artifacts/phase10/PHASE10_PLAN.md`
- Phase 10 analysis report (EN): `TCC/artifacts/phase10/analysis/analysis_report_phase10.md`
- Phase 10 analysis report (PT-BR): `TCC/artifacts/phase10/analysis/analysis_report_phase10_ptbr.md`
- Phase 5 artifact (source of the 18-point training dataset): `TCC/artifacts/phase5/PHASE5_ARTIFACT.md`
- Phase 6 TCC monograph (existing version, not yet updated with Phase 10 findings): `TCC/artifacts/phase6/lapesd-thesis/body.tex`
- Analysis pipeline: `TCC/artifacts/phase10/analysis/analyze_synth_duration.py`, `parse_synth_duration.py`
- Project master plan: `TCC/TCC_MANA_INTEGRATION_PLAN.md`
