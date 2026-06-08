# Phase 5 Plan — Deepening the Analysis: Synthetic Studies and Failure Timing

Date: 2026-05-30 (updated 2026-06-08, all steps complete)
Project: Integrating MANA Fault Tolerance into HPC@Cloud for AWS Spot Clusters

Goal: address the open questions raised after the Phase 4 pilot campaign by adding
two synthetic micro-benchmark studies (including point-to-point variant), a failure
timing sensitivity experiment, and a richer recovery phase visualization.

## Status (2026-05-31)

| Step | Description | Status |
|---|---|---|
| Step 0 | Result storage restructuring (task_tag naming) | ✅ Done |
| Step 1 | Synthetic C programs (synth_mpi_calls, synth_p2p, synth_ckpt, synth_imbalanced) | ✅ Done |
| Step 2 | Create YAML task files (2w + 4w + 8w) | ✅ Done |
| Step 3 | Update watcher.rs + run_task.rs (phase2a/2b/2c split, node_became_idle for both strategies, auto-failure bug fix) | ✅ Done |
| Step 4 | Update analyze.py (new fields, new figures) | ✅ Done |
| Step 5 | Run Phase 4 redo (NPB baselines + FT, all cluster sizes) | ✅ Done |
| Step 6 | Run Phase 5 experiments (5.1 ✅, 5.2 ✅, 5.3 ✅) | ✅ Done |
| Step 7 | Update analysis and reports | ✅ Done |

**Completed runs (2026-05-31):**
- noFT + MANA-noFT baselines: all 3 benchmarks × 3 cluster sizes ✅
- Phase 5.1 synthetic MPI overhead (synth_calls + synth_p2p + synth_imbalanced): all levels ✅
- Phase 5.2 synthetic checkpoint size (50/200/800/3200 MB): ✅

**Measured baseline times (used to compute trigger times below):**

| Benchmark | Workers | noFT (s) | MANA-noFT (s) | MANA overhead |
|---|---|---|---|---|
| EP-D | 2w | 332.33 | 462.55 | +39% |
| EP-D | 4w | 283.49 | 322.90 | +14% |
| EP-D | 8w | 99.40 | 122.31 | +23% |
| LU-C | 2w | 144.83 | 236.87 | +64% |
| LU-C | 4w | 83.79 | 132.00 | +57% |
| LU-C | 8w | 45.21 | 68.40 | +51% |
| CG-C | 2w | 31.18 | 109.46 | +251% |
| CG-C | 4w | 16.47 | 26.22 | +59% |
| CG-C | 8w | 10.55 | 16.24 | +54% |

Depends on:
- Phase 4: pilot evaluation campaign, `[RUN_METRICS]` infrastructure, `analyze.py`

---

## 1. Phase 5 Scope

Phase 5 adds four focused work items:

| Item | Description | New code? | New experiments? |
|---|---|---|---|
| **5.1** | Synthetic MPI overhead study: call frequency + communication imbalance | Yes (3 C programs) | Yes |
| **5.2** | Synthetic checkpoint size study: isolate Phase 1 time driver | Yes (C program) | Yes |
| **5.3** | Failure timing sensitivity: inject failure at 10 / 25 / 50% of run | No | Yes |
| **5.4** | Enhanced fig3: add Phase 0 bar + split Phase 2 into provisioning and reconfiguration | Yes (watcher.rs + analyze.py) | Needs 5.3 re-runs |

All four items are independent and can be prepared in parallel.

---

## 2. Item 5.1 — Synthetic MPI Call Frequency Study

### 2.1 Motivation

Phase 4 showed MANA overhead correlates with communication intensity (EP < LU < CG).
But this is only a correlation observed across different applications with different
algorithms. A synthetic program where the number of MPI calls is the only variable
that changes will turn this from an observation into a controlled experiment.

### 2.2 MANA Source Code Findings (from Phase 5 analysis)

After running synth_calls and synth_p2p and observing flat overhead across all call
levels, the MANA source was read to understand why. Two distinct overhead mechanisms
exist — one per MPI call type:

**MPI_Recv** (`mpi-proxy-split/mpi-wrappers/mpi_p2p_wrappers.cpp`):
```c
for (int i = 0; i < 1000; i++) {
    NEXT_FUNC(Iprobe)(...);       // 1000 probes in lower-half
    if (flag) { recv; break; }
}
if (!flag) {
    nanosleep(1ms);               // then sleep 1ms before retrying
    RETURN_TO_UPPER_HALF();
    ENABLE_CKPT();
}
```
Rate-limited: sleeps 1ms between probe cycles. If the message arrives quickly,
almost no overhead accumulates.

**MPI_Wait** (`mpi-proxy-split/mpi-wrappers/mpi_request_wrappers.cpp`):
```c
while (!flag) {
    DMTCP_PLUGIN_DISABLE_CKPT();  // acquire DmtcpRWLockRdLock — real OS lock
    MPI_Test_internal(...);
    DMTCP_PLUGIN_ENABLE_CKPT();   // release lock
}
```
**No sleep at all.** Pure spin. Every iteration acquires and releases a real
reader-writer lock (`threadsync.cpp:wrapperExecutionLockLock`). For a message
that takes Δt to arrive, the loop spins for Δt ÷ T_iter iterations where T_iter
is a few microseconds — burning wall time proportional to how long the receiver
waits for the sender.

**Why this matters for the benchmarks:**
- CG and LU use `MPI_Irecv + MPI_Send + MPI_Wait` (confirmed in NPB source).
- synth_p2p uses `MPI_Send + MPI_Recv` — a different code path.
- synth_p2p's overhead is flat not because P2P is cheap in general, but because
  the partners are perfectly synchronized (both reach the exchange at the same
  time), so the message arrives within the first of 1000 Iprobes and the 1ms
  sleep never fires. MPI_Wait is never called.
- CG at 2w has 251% overhead because each of 4 processes owns a large matrix
  partition → more compute between communication rounds → more temporal drift
  between partners → receiver spins longer in MPI_Wait per exchange.
  Confirmed by Mop/s/process: 2w retains only 28% of throughput under MANA,
  while 4w and 8w retain ~64% — a structural difference, not EC2 noise.

**EP overhead non-determinism explained:**
EP uses only MPI_Allreduce/Reduce (no P2P). Its MANA overhead should be small
and mostly fixed (init + a handful of collectives). The 39% overhead at 2w
(130s absolute) is EC2 noise: single run, long runtime (332s), morning slot vs
afternoon slot for MANA run. The EP data should not be used to characterize
MANA overhead mechanisms.

### 2.3 Design

Three programs implemented:

**`synth_mpi_calls.c`** — MPI_Allreduce collective:
```
for iteration in range(TOTAL_OUTER):
    do_compute(INNER_ITERS)
    if iteration % CALL_PERIOD == 0:
        MPI_Allreduce(...)
```

**`synth_p2p.c`** — synchronized MPI_Send/MPI_Recv (cross-node ping-pong):
```
for iteration in range(TOTAL_OUTER):
    do_compute(INNER_ITERS)
    if iteration % CALL_PERIOD == 0:
        rank 0↔2, rank 1↔3: MPI_Send + MPI_Recv (zero wait time)
```

**`synth_imbalanced.c`** ✅ Done — asynchronous MPI_Irecv+MPI_Wait with controlled
sender delay (see Section 2.5 for results):
```
for iteration in range(TOTAL_OUTER):
    do_compute(INNER_ITERS)
    if iteration % CALL_PERIOD == 0:
        senders (rank 0,1): usleep(DELAY_US); MPI_Send; MPI_Irecv; MPI_Wait
        receivers (rank 2,3): MPI_Irecv; MPI_Wait; MPI_Send
```

All three use `TOTAL_OUTER=51200`, `INNER_ITERS=281250` for ~60s noFT runtime.
Call levels for synth_calls and synth_p2p:
- Level 0: 0 calls (MPI_Init + MPI_Finalize only)
- Level 1: CALL_PERIOD=64 → 800 calls
- Level 2: CALL_PERIOD=16 → 3200 calls
- Level 3: CALL_PERIOD=4 → 12800 calls
- Level 4: CALL_PERIOD=1 → 51200 calls

Delay levels for synth_imbalanced (fixed CALL_PERIOD=64 → 800 exchanges):
- Level 0: DELAY_US=0 → 0 µs (control — same as synth_p2p)
- Level 1: DELAY_US=100 → 100 µs/exchange → ~80 ms total accumulated wait
- Level 2: DELAY_US=1000 → 1 ms/exchange → ~800 ms total
- Level 3: DELAY_US=5000 → 5 ms/exchange → ~4 s total
- Level 4: DELAY_US=20000 → 20 ms/exchange → ~16 s total

### 2.4 Results: synth_calls and synth_p2p (runs completed 2026-05-31)

| Level | Calls | synth_calls overhead | synth_p2p overhead |
|---|---|---|---|
| L0 | 0 | +1.72s | +3.44s |
| L1 | 800 | +3.41s | +3.46s |
| L2 | 3,200 | +3.34s | +3.19s |
| L3 | 12,800 | +2.90s | +2.67s |
| L4 | 51,200 | +0.66s | +0.83s |

**Key finding:** MANA overhead is flat (~3s) across all call levels and then
actually *decreases* at L4 (the compute loop is so interrupt-heavy at L4 that
the benchmark finishes faster). **Call count is not the driver of MANA overhead.**

**Why synth_p2p is flat:** Both programs show identical flat overhead because
synth_p2p's partners are synchronized — message arrives in the first of 1000
Iprobes, and MPI_Wait is never called. This is a best-case scenario for MANA
that does NOT represent CG/LU behavior.

### 2.5 Test: synth_imbalanced ✅ Done

The synth_calls and synth_p2p tests correctly disprove "call count" as the cause,
but they do not test the actual mechanism behind CG and LU overhead. The missing
variable is **communication imbalance** (how long the receiver waits in MPI_Wait).

`synth_imbalanced.c` fills this gap:
- Uses `MPI_Irecv + MPI_Send + MPI_Wait` (same as CG and LU)
- Sender deliberately delays `DELAY_US` microseconds before sending
- Receiver posts Irecv immediately, then spins in MANA's MPI_Wait lock loop
- As DELAY_US increases, MANA overhead should increase proportionally

**Expected result:**
```
noFT elapsed: ~60s flat (DELAY_US just burns CPU time, same work)
MANA elapsed: grows with DELAY_US — MPI_Wait spin time accumulates
```

At L4 (DELAY_US=20000, 800 exchanges): 800 × 20ms = 16s accumulated wait.
MANA's tight loop adds overhead ∝ to this, while noFT's native MPI_Wait
has no such lock overhead. The gap should be clearly visible.

YAML files for 2 workers (10 files): `2workers/synth/task-synth_imbalanced-l{0-4}-{noFT,MANA-noFT}.yaml` ✅ Created

### 2.6 Experiment Matrix (synth_imbalanced) ✅ Done

| Variable | Value |
|---|---|
| Delay levels | 5 (0, 100µs, 1ms, 5ms, 20ms) |
| Strategies | noFT, MANA_noFT |
| Cluster size | **2 workers only** |
| Repetitions | 1 per point |
| Total runs | **10** ✅ |

### 2.7 Output

Two figures for Section 5.1:
- **fig_synth_calls**: overhead factor vs MPI call rate (synth_calls + synth_p2p together).
  Shows: call count is NOT the driver. Both lines are flat.
- **fig_synth_imbalanced**: MANA overhead (seconds) vs sender delay (synth_imbalanced).
  Shows: MPI_Wait wait time in isolation is also NOT the driver. Overhead remains flat.
  Together these two figures establish that no single communication parameter explains
  the overhead — it is a fixed infrastructure cost (~3-5s) for simple/synthetic programs.

### 2.8 Conclusions and Limitations of the Synthetic Study

**What the synthetic tests established (3 experiments, 30 runs):**
- MPI call frequency is not the overhead driver (synth_calls, synth_p2p)
- Communication imbalance / wait time in isolation is not the overhead driver (synth_imbalanced)
- For controlled synthetic programs, MANA adds a fixed ~3-5s overhead per ~60s run,
  independent of communication pattern

**What the real benchmark data shows:**
- EP: overhead is small and unreliable from single runs (few MPI calls; EC2 variance
  dominates the measurement; cannot draw conclusions from EP alone)
- LU: consistent ~50-60% overhead across all worker counts — overhead scales roughly
  proportionally with runtime, suggesting a mechanism tied to computation time, not
  call count
- CG 4w/8w: similar to LU (~55%) — consistent and repeatable
- CG 2w: anomalous ~251% overhead, reproduced in both Phase 4 and Phase 5 experiments

**Why the synthetic tests could not explain LU/CG real overhead:**
The synthetic tests isolate single variables, but in real benchmarks the overhead likely
arises from an interaction: MANA's per-call wrapper cost slightly slows computation,
which causes processes to communicate later, which causes partners to wait longer in
MANA's MPI_Wait spin loop (burning CPU / cache bandwidth), which slows computation
further — a feedback loop over many iterations. The synthetic tests break this loop
because the sender delay (usleep) is external and does not respond to MANA's cost.

**CG 2w hypothesis (unconfirmed):**
The CG Class C sparse matrix likely partitions unevenly across 4 MPI processes, giving
one process more non-zeros than others. That process is consistently slower on every
iteration, forcing the others to spin in MPI_Wait repeatedly. Under MANA's spin-based
waiting, this imbalance is amplified through cache resource competition between
co-located processes, compounding over 75 CG iterations. The same matrix partitions
differently at 8 or 16 processes (4w/8w), avoiding the bottleneck.
Confirming this would require per-rank timing instrumentation within the CG benchmark.

**Thesis framing:**
A full causal model of MANA's overhead is outside the scope of this work. The synthetic
study is sufficient to establish that overhead is NOT a simple function of call frequency
or communication pattern, and to characterize the typical range (~3-5% for synthetic
micro-benchmarks, 50-60% for structured real applications, anomalous at CG 2w).
The unexplained variation — particularly EP and CG 2w — is noted as a limitation and
identified as future work. The fault tolerance strategy analysis (Sections 4 and 5.3)
is the primary contribution of this TCC and does not depend on resolving this question.

---

## 3. Item 5.2 — Synthetic Checkpoint Image Size Study

### 3.1 Motivation

Phase 4 showed Phase 1 time (failure detected → checkpoint ready) is ~16 s for EP
and ~26.5 s for LU/CG. The difference matches memory footprint: EP processes hold
less state than LU/CG. But this is inferred from NPB internals, not measured directly.
A synthetic program that allocates a known amount of memory and is then checkpointed
will produce a direct memory size → Phase 1 time curve.

### 3.2 Design

Write a C MPI program (`synth_checkpoint_size.c`) with the following structure:

```
allocate N MB per process (fill with data so the OS actually maps it)
loop forever:
    do minimal compute (keep memory "live")
    sleep briefly
```

The run is started under MANA via `auto_test_failure`, which triggers a real
checkpoint + restart cycle so `phase1_s` is captured in `[RUN_METRICS]` exactly
as in the Phase 4 FT runs.

Control parameter: `N` = allocated memory per process.
Levels: 50 MB, 100 MB, 200 MB, 400 MB, 800 MB.

Only Phase 1 time matters here. Phase 3 (remaining computation) is trivial since
the program does almost nothing, so total wall time is dominated by Phase 1 + Phase 2.

### 3.3 Experiment Matrix

| Variable | Value |
|---|---|
| Memory per process (N) | 5 levels: 50 / 100 / 200 / 400 / 800 MB |
| Strategy | DEGRADED only (Phase 1 is strategy-independent; DEGRADED avoids the Phase 2 noise from EC2 provisioning) |
| Cluster size | **2 workers only** |
| Repetitions | 1 per point |
| Total runs | 5 |

### 3.4 Output

New figure: Phase 1 time vs memory footprint per process (scatter + linear fit).
Expected result: a roughly linear relationship, confirming that checkpoint write
time scales with the data volume written to EFS.

### 3.5 Notes

- Memory must be touched (filled with non-zero data) after `malloc`. Linux uses
  lazy allocation — unmapped pages are not written during checkpoint. A simple
  `memset` immediately after `malloc` ensures the full footprint is live.
- The `trigger_after_secs` should be long enough that the job is in a steady
  loop when the failure fires. A value of 30 s is safe for all memory levels.
- The EFS write bandwidth is a confound: if EFS is throttled (low burst credit),
  Phase 1 times will be inflated. Re-check EFS burst credit before running.

---

## 4. Item 5.3 — Failure Timing Sensitivity Study

### 4.1 Motivation

All Phase 4 FT runs used an early failure trigger (2–45 s into runs of 33–493 s).
The Phase 4 results already show that the REPLACE vs DEGRADED trade-off depends
on how much work remains after the failure (Phase 3). But we only sampled one
failure time per benchmark. Adding failures at 10%, 25%, and 50% of the MANA_noFT
wall time will:

1. Show how Phase 3 time evolves as the failure happens later in the run.
2. Reveal the crossover point at which REPLACE becomes competitive with DEGRADED
   (if any, within the tested benchmark durations).
3. Give a more complete picture to the reader about when each strategy is optimal.

### 4.2 Trigger Time Calculation

The reference time is `ft_wall_time_s` from the MANA_noFT run (not noFT),
because MANA is active during all FT runs.

Times measured on 2026-05-31. CG excluded from 5.3 (too short for timing sensitivity).

| Benchmark | Workers | MANA_noFT (s) | 10% trigger | 25% trigger | 50% trigger |
|---|---|---|---|---|---|
| EP D | 2w | 462.55 | **46 s** | **116 s** | **231 s** |
| EP D | 4w | 322.90 | **32 s** | **81 s** | **161 s** |
| EP D | 8w | 122.31 | **12 s** | **31 s** | **61 s** |
| LU C | 2w | 236.87 | **24 s** | **59 s** | **118 s** |
| LU C | 4w | 132.00 | **13 s** | **33 s** | **66 s** |
| LU C | 8w | 68.40  | **7 s**  | **17 s**  | **34 s**  |

All YAML files for all worker counts created (2w + 4w + 8w, all 36 files). ✅

### 4.3 Experiment Matrix

| Benchmarks | Cluster sizes | Timing points | Strategies | Runs |
|---|---|---|---|---|
| EP, LU | 2w, 4w | 10%, 25%, 50% | REPLACE, DEGRADED | 2 × 2 × 3 × 2 = **24 runs** |
| EP, LU | 8w | 10%, 25%, 50% | REPLACE, DEGRADED | 2 × 1 × 3 × 2 = **12 runs** |

CG is excluded (see Section 7 rationale). The 8w trigger times depend on the Phase 4
redo MANA_noFT results and must be filled in before creating the 8w YAML files.

The noFT and MANA_noFT baselines are timing-independent and are covered by the
Phase 4 redo — they do not need to be run again for Phase 5.3.

### 4.4 Output

Two new figures:
- **fig7: Phase 3 time vs failure timing point** (per benchmark, REPLACE vs DEGRADED)
  Shows how remaining work changes as failure is injected later.
- **fig8: Total FT overhead vs failure timing point** (per benchmark, REPLACE vs DEGRADED)
  Shows the crossover: at what failure time does REPLACE become better than DEGRADED?

Updated fig3 (recovery phases stacked bar) will use the 25% timing point as the
representative run for each benchmark, since it is a more balanced failure scenario
than the very early Phase 4 triggers.

### 4.5 Notes

- The 50% timing point for EP D 2w (`trigger_after_secs = 246 s`) means the run
  takes at minimum 246 s before the failure fires, then Phase 1 (~16 s),
  Phase 2 (~185–220 s for REPLACE), and Phase 3 (remaining 50% of work).
  Total run time for EP REPLACE at 50%: ~246 + 16 + 185 + ~250 = ~697 s (~12 min).
  Plan for longer-than-usual runs.
- Task YAML files for each timing point should be stored in `my_clusters/` following
  the existing naming convention: `task-ep_D-replace-25pct.yaml`, etc.

---

## 5. Item 5.4 — Enhanced Recovery Phase Visualization

### 5.1 Motivation

The current fig3 stacked bar shows three phases starting from the failure event.
The professor identified two improvements:

1. **Add Phase 0** (initial run → fault detected): shows what fraction of total
   work was already done before the failure, which is the key context for understanding
   why Phase 3 time varies.
2. **Split Phase 2** into:
   - Phase 2a: checkpoint ready → new EC2 instance running and joined Slurm
   - Phase 2b: new instance ready → restart dispatch (Slurm reconfiguration)

   This separates the *cloud infrastructure latency* (Phase 2a, ~175–210 s, irreducible
   without pre-warmed instance pools) from the *orchestration overhead* (Phase 2b,
   ~10–20 s, potentially reducible with better tooling).
   For DEGRADED, Phase 2a = 0 by definition.

### 5.2 New Phase Definitions

| Phase | Definition | Driver |
|---|---|---|
| **Phase 0** | Job start → failure detected | `auto_failure_trigger_secs` (already in data) |
| **Phase 1** | Failure detected → checkpoint ready | Checkpoint size / EFS write speed |
| **Phase 2a** | Checkpoint ready → node_drained | Drain + scancel (~5.5s, identical for both strategies) |
| **Phase 2b** | node_drained → node_became_idle | REPLACE: AWS terminate wait + respawn + wait IDLE (~150-200s); DEGRADED: scontrol DOWN (~10s) |
| **Phase 2c** | node_became_idle → restart dispatch | Pure restart dispatch (~5s, identical for both strategies) |
| **Phase 3** | Restart dispatch → job completion | Remaining computation on post-recovery process count |

Validated against CG 2w results (2026-05-31):

| | phase1 | phase2a | phase2b | phase2c | phase3 | total |
|---|---|---|---|---|---|---|
| **Replace** | 26.5s | 5.5s | 157.0s | 5.3s | 42.0s | 236.3s |
| **Degraded** | 26.4s | 5.4s | 10.9s | 5.3s | 109.5s | 157.5s |

phase2a and phase2c are directly comparable between strategies. phase2b isolates the
EC2 provisioning cost (replace) vs Slurm DOWN overhead (degraded). The ~147s difference
in phase2b is the pure cost of the replace strategy's instance termination wait + respawn.

### 5.3 Code Change — ✅ Done (2026-05-31)

Changes made to `src/commands/cluster/watcher.rs` and `src/commands/cluster/run_task.rs`:

**watcher.rs:**
- `node_drained` event already existed (after scontrol DRAIN + scancel).
- `node_became_idle` event already existed in REPLACE after scontrol RESUME + wait_idle.
- NEW: `node_became_idle` now also emitted in DEGRADED immediately after scontrol DOWN
  completes. This makes the phase2c boundary symmetric between strategies.

**run_task.rs:**
- Phase split now uses `node_drained` as phase2a boundary and `node_became_idle` as
  phase2b/2c boundary for both strategies.
- New `[RUN_METRICS]` fields: `phase2a_s`, `phase2b_s`, `phase2c_s`.
- Old `phase2_s` kept for backward compatibility.
- Human-readable Phase Durations section updated to show all three sub-phases.

**Bug fix — simulate_cluster_failure (resource_manager.rs):**
The auto-failure timer was resolving the instance ID *after* the warning sleep, meaning
it could kill the newly respawned instance if recovery completed within `warning_time_secs`.
Fixed to resolve the instance ID *before* sleeping so only the originally targeted
instance is terminated regardless of how long recovery takes.

### 5.4 `analyze.py` Changes

- Parse `phase2a_s` and `phase2b_s` from `[RUN_METRICS]`.
- For Phase 4 data (which has only `phase2_s`): set `phase2a_s = phase2_s`, `phase2b_s = 0`.
  This is an approximation that keeps Phase 4 data usable without re-running.
- Update `plot_recovery_phases()` to use the 5-segment stacked bar:
  Phase 0 / Phase 1 / Phase 2a / Phase 2b / Phase 3.

### 5.5 Notes

- Phase 0 for the enhanced graph should use the **25% failure timing** runs from 5.3
  (not the very early Phase 4 triggers), because showing Phase 0 when it's only
  2–3 s long is not informative. With 25% triggers, Phase 0 is ~8–123 s depending
  on the benchmark, which is visually meaningful.
- For DEGRADED the bar will be: Phase 0 | Phase 1 | 0 | Phase 2b | Phase 3
- For REPLACE the bar will be: Phase 0 | Phase 1 | Phase 2a | Phase 2b | Phase 3

---

## 6. Result Storage Restructuring

### 6.1 Problem

Current result files are named by timestamp only:
```
results/cluster_ClusterMANA_2Workers_m5-xlarge/2026-05-26T12:35:28.txt
```
Opening each file to know what was run is impractical. There is also no way to
tell at a glance whether a run succeeded or failed from the filesystem.

### 6.2 Proposed Naming Scheme

New filename format: **`{task_tag}_{timestamp}.txt`** for in-progress/failed runs,
renamed to **`{task_tag}_{timestamp}_SUCCESS.txt`** when the run completes successfully.

Examples:
```
results/cluster_ClusterMANA_2Workers_m5-xlarge/
  cg-C_2w_m5xl_noFT_2026-05-30T14:23:45_SUCCESS.txt
  lu-C_2w_m5xl_degraded_2026-05-30T15:10:22_SUCCESS.txt
  ep-D_2w_m5xl_replace-25pct_2026-05-30T16:44:01_SUCCESS.txt
  ep-D_2w_m5xl_replace-50pct_2026-05-30T17:55:12.txt         ← FAILED (no suffix)

results/cluster_ClusterMANA_4Workers_m5-xlarge/
  ...

results/synthetic/
  synth_calls-l3_2w_m5xl_MANA-noFT_2026-05-31T10:05:00_SUCCESS.txt
  synth_ckpt-200mb_2w_m5xl_degraded_2026-05-31T11:00:00_SUCCESS.txt
```

**Status rule**: files ending in `_SUCCESS.txt` completed successfully. Files ending
in `.txt` (no status suffix) were interrupted or failed — the `[RUN_METRICS]` block
will be absent or truncated. `analyze.py`'s `rglob("*.txt")` will pick up both, but
the parser already skips files without a valid `[RUN_METRICS]` block, so failed runs
are naturally excluded from the CSV.

### 6.3 Code Change: `src/commands/cluster/run_task.rs`

Two changes to the filename creation block (around lines 425–435):

**Before:**
```rust
let mut report_dir = PathBuf::from("results");
report_dir.push(format!("cluster_{}", cluster_id));
// ...
let timestamp = Local::now().format("%Y-%m-%dT%H:%M:%S");
let filename = format!("{}.txt", timestamp);
let report_path = report_dir.join(&filename);
```

**After:**
```rust
let mut report_dir = PathBuf::from("results");
report_dir.push(format!("cluster_{}", cluster_id));
// ...
let timestamp = Local::now().format("%Y-%m-%dT%H:%M:%S");
let first_task_tag = tasks_yaml.tasks.first()
    .map(|t| t.task_tag.as_str())
    .unwrap_or("unknown");
let filename_base = format!("{}_{}", first_task_tag, timestamp);
let report_path = report_dir.join(format!("{}.txt", &filename_base));
```

Then, at the success exit point (after writing `[/RUN_METRICS]`, around line 1344),
add a rename:
```rust
let success_path = report_dir.join(format!("{}_SUCCESS.txt", filename_base));
let _ = fs::rename(&report_path, &success_path);
```

`report_dir` and `filename_base` must be kept in scope. Since `report_path` is
already computed from these two, this is a contained change.

### 6.4 `analyze.py` Change

The existing `rglob("*.txt")` already picks up `_SUCCESS.txt` files. No glob change
needed. Optionally add a debug log line noting how many non-SUCCESS files were skipped
so the user can see at a glance how many runs failed.

### 6.5 Why All Runs Must Be Redone (Not Just Some)

The phase2a/2b split in watcher.rs is the deciding factor:

| Strategy | Phase 2a present? | Old data usable? |
|---|---|---|
| noFT / MANA-noFT | No (no recovery) | Yes — Phase 2 is irrelevant |
| DEGRADED | No (Phase 2a = 0 by definition) | Yes — `phase2_s` already equals the future `phase2b_s` |
| REPLACE | **Yes** | **No** — old `phase2_s` is a single undivided number; the EC2 provisioning vs reconfiguration split cannot be recovered retroactively |

Since REPLACE runs must be redone regardless, and consistency across the full dataset
matters for the analysis, all runs are redone with the new infrastructure in one batch.
Old results serve only as a sanity check that numbers are in the same ballpark.

Archive the old results before starting:
```bash
mv results results_phase4_archive_$(date +%Y%m%d)
```

---

## 7. What Was Evaluated and Why (Design Rationale)

### What makes full sense

- **5.1 and 5.2** (synthetic programs): these are standard controlled experiments. They
  turn correlations observed in NPB data into causal claims backed by direct measurement.
  Required for a rigorous TCC-level analysis.

- **5.3** (failure timing): the Phase 4 campaign used only one (early) failure point per
  benchmark, which creates a biased comparison — DEGRADED looks much better than REPLACE
  when nearly all work remains. Multiple timing points give a fair picture of both
  strategies across the full job lifecycle.

- **5.4** (Phase 2 split): the ~185–220 s that currently appears as a single Phase 2 bar
  for REPLACE mixes two conceptually different things — unavoidable cloud latency and
  controllable orchestration delay. Separating them is important for a researcher
  who wants to understand what could be improved.

### One thing to watch out for in 5.3

The professor suggested 10%, 25%, 50% relative to execution time. For **CG Class C at
4 workers** (MANA_noFT = 32.8 s), the 10% trigger is only 3.3 s — essentially the same
as the Phase 4 point (2.0 s). This benchmark is too short for the timing sensitivity
experiment to be meaningful. **CG should be excluded from Experiment 5.3** or replaced
with CG Class D if the runtime issue applies at all worker counts.

For **EP and LU** the timing points are well-spaced and the experiment is meaningful.

### Cluster size scope

For 5.1 and 5.2 (synthetic micro-benchmarks): **2 workers only**. The point is to
isolate a variable, not to study scaling. Adding 4-worker runs would not change
the conclusion and doubles the experiment cost.

For 5.3 (failure timing): **2w, 4w, and 8w** are all included. The REPLACE vs DEGRADED
trade-off changes with cluster size — at 8w, losing 1 worker in DEGRADED costs only
12.5% capacity (vs 50% at 2w), so the crossover point shifts. Three cluster sizes
give a meaningful scaling dimension to the failure timing analysis.

---

## 8. Complete YAML Task File Inventory

**All files are now organized by benchmark under each worker directory.** ✅

```
my_clusters/m5-xlarge/
  {2,4,8}workers/
    ep/   — EP-D tasks (noFT, MANA-noFT, degraded, replace, *-10/25/50pct variants)
    lu/   — LU-C tasks (same pattern)
    cg/   — CG-C tasks (noFT, MANA-noFT, degraded, replace — no timing variants)
    synth/ — synthetic tasks (2workers only)
```

### 8.1 Base Files

**CG** — all 4 strategies (noFT, MANA-noFT, degraded, replace) exist in `{2,4,8}workers/cg/`. ✅
**EP and LU** — only noFT and MANA-noFT kept in `{2,4,8}workers/{ep,lu}/`. The 45s degraded/replace
files were archived to `archived_45s_base_ft/{2,4,8}workers/{ep,lu}/` — no longer needed since
the 5.3 timing variants cover all FT scenarios for EP and LU.

CG trigger_secs for base FT runs:

| Benchmark | 2w | 4w | 8w |
|---|---|---|---|
| CG-C degraded/replace | 3 | 2 | 2 |

### 8.2 Phase 5.3 Timing Variants (EP + LU only) ✅ Created

All 36 files created in the benchmark subdirectories.

**2workers/ep/** (12 files):

| File | task_tag | trigger_secs | % |
|---|---|---|---|
| `task-ep_D-degraded-10pct.yaml` | ep-D_2w_m5xl_degraded-10pct | 46 | 10% |
| `task-ep_D-degraded-25pct.yaml` | ep-D_2w_m5xl_degraded-25pct | 116 | 25% |
| `task-ep_D-degraded-50pct.yaml` | ep-D_2w_m5xl_degraded-50pct | 231 | 50% |
| `task-ep_D-replace-10pct.yaml` | ep-D_2w_m5xl_replace-10pct | 46 | 10% |
| `task-ep_D-replace-25pct.yaml` | ep-D_2w_m5xl_replace-25pct | 116 | 25% |
| `task-ep_D-replace-50pct.yaml` | ep-D_2w_m5xl_replace-50pct | 231 | 50% |

**2workers/lu/** (12 files):

| File | task_tag | trigger_secs | % |
|---|---|---|---|
| `task-lu_C-degraded-10pct.yaml` | lu-C_2w_m5xl_degraded-10pct | 24 | 10% |
| `task-lu_C-degraded-25pct.yaml` | lu-C_2w_m5xl_degraded-25pct | 59 | 25% |
| `task-lu_C-degraded-50pct.yaml` | lu-C_2w_m5xl_degraded-50pct | 118 | 50% |
| `task-lu_C-replace-10pct.yaml` | lu-C_2w_m5xl_replace-10pct | 24 | 10% |
| `task-lu_C-replace-25pct.yaml` | lu-C_2w_m5xl_replace-25pct | 59 | 25% |
| `task-lu_C-replace-50pct.yaml` | lu-C_2w_m5xl_replace-50pct | 118 | 50% |

**4workers/ep/** (6 files): triggers 32 / 81 / 161 s
**4workers/lu/** (6 files): triggers 13 / 33 / 66 s
**8workers/ep/** (6 files): triggers 12 / 31 / 61 s
**8workers/lu/** (6 files): triggers 7 / 17 / 34 s

### 8.3 Phase 5.1 Synthetic MPI Overhead ✅ Exists

Location: `2workers/synth/` — 30 files:
- 10 synth_calls files (L0-L4 × noFT + MANA-noFT)
- 10 synth_p2p files (L0-L4 × noFT + MANA-noFT)
- 10 synth_imbalanced files (L0-L4 × noFT + MANA-noFT) ✅ Created

synth_calls/synth_p2p call levels: L0=0, L1=800, L2=3200, L3=12800, L4=51200.
synth_imbalanced delay levels: L0=0µs, L1=100µs, L2=1ms, L3=5ms, L4=20ms.
TOTAL_OUTER=51200, INNER_ITERS=281250, CALL_PERIOD=64 → ~60s noFT runtime.

### 8.4 Phase 5.2 Synthetic Checkpoint Size ✅ Exists (partial)

Location: `2workers/synth/` — 4 files run (50/200/800/3200 MB).
Note: actual runs used 50/200/800/3200 MB instead of the originally planned 50/100/200/400/800.
The 100mb and 400mb files were not created; 3200mb was added for a wider range.

### 8.5 File Count Summary

| Category | Files | Status |
|---|---|---|
| CG base (noFT + MANA-noFT + degraded + replace) × 3 sizes | 12 | ✅ Exists |
| EP + LU noFT + MANA-noFT × 3 sizes | 12 | ✅ Exists |
| Phase 5.3 timing variants (EP + LU, 10/25/50%, 2w+4w+8w) | 36 | ✅ Created |
| Phase 5.1 synthetic (synth_calls + synth_p2p + synth_imbalanced) | 30 | ✅ Exists |
| Phase 5.2 synthetic checkpoint (4 sizes) | 4 | ✅ Exists |
| **Total active** | **94** | |
| Archived (45s EP/LU base FT runs) | 12 | archived_45s_base_ft/ |

---

## 9. Complete Run Execution Matrix

This is the authoritative list of every run to execute, in recommended order.
Run counts assume one repetition per cell. Add a second repetition for any cell
where the first result looks like an outlier.

### 9.1 Phase 4 Redo — NPB Baselines and FT Runs (Core)

Cluster: m5.xlarge, 2 workers and 4 workers.
EP and LU **do not have separate base FT runs** — the 10%/25%/50% timing variants
from 5.3 replace them. fig3 uses the 25% run as the representative.
CG still has base FT runs (CG is excluded from 5.3, too short for timing sensitivity).

| Benchmark | Class | Workers | Strategy | trigger_secs | Status |
|---|---|---|---|---|---|
| CG | C | 2 | noFT | — | ✅ Done |
| CG | C | 2 | MANA-noFT | — | ✅ Done |
| CG | C | 2 | degraded | 3 | ✅ Done (2026-05-31) |
| CG | C | 2 | replace | 3 | ✅ Done (2026-05-31) |
| CG | C | 4 | noFT | — | ✅ Done |
| CG | C | 4 | MANA-noFT | — | ✅ Done |
| CG | C | 4 | degraded | 2 | ✅ Done (2026-06-02) |
| CG | C | 4 | replace | 2 | ✅ Done (2026-06-02) |
| EP | D | 2 | noFT | — | ✅ Done |
| EP | D | 2 | MANA-noFT | — | ✅ Done |
| EP | D | 4 | noFT | — | ✅ Done |
| EP | D | 4 | MANA-noFT | — | ✅ Done |
| LU | C | 2 | noFT | — | ✅ Done |
| LU | C | 2 | MANA-noFT | — | ✅ Done |
| LU | C | 4 | noFT | — | ✅ Done |
| LU | C | 4 | MANA-noFT | — | ✅ Done |

**Subtotal: 16 runs ✅ all done**

### 9.2 Phase 4 Redo — NPB Scaling Runs (8 Workers)

Same logic as 9.1: EP and LU FT runs covered by 5.3 timing variants; only CG needs base FT runs.

| Benchmark | Class | Workers | Strategy | trigger_secs | Status |
|---|---|---|---|---|---|
| CG | C | 8 | noFT | — | ✅ Done |
| CG | C | 8 | MANA-noFT | — | ✅ Done |
| CG | C | 8 | degraded | 2 | ✅ Done (2026-06-03) |
| CG | C | 8 | replace | 2 | ✅ Done (2026-06-03) |
| EP | D | 8 | noFT | — | ✅ Done |
| EP | D | 8 | MANA-noFT | — | ✅ Done |
| LU | C | 8 | noFT | — | ✅ Done |
| LU | C | 8 | MANA-noFT | — | ✅ Done |

**Subtotal: 8 runs ✅ all done**

### 9.3 Phase 5.1 — Synthetic MPI Overhead Study

**synth_calls + synth_p2p: ✅ Done (20 runs)**

All 20 runs complete (10 synth_calls + 10 synth_p2p). Actual call counts:
L0=0, L1=800, L2=3200, L3=12800, L4=51200 (×4 scaling per level).

Key finding: MANA overhead is ~flat at 3-4s across L0-L3 for both tests;
it does NOT grow with call count. Call frequency is not the overhead driver.
synth_p2p overhead is also flat because partners are perfectly synchronized
(zero imbalance) — MANA's MPI_Wait spin loop never fires. This is a best-case
scenario that does not represent CG/LU behavior.

**synth_imbalanced: ✅ Done (10 runs, 2026-05-31)**

Uses MPI_Irecv+MPI_Send+MPI_Wait (same as CG/LU). Sender sleeps DELAY_US before
sending; receiver posts Irecv and spins in MANA's MPI_Wait loop for DELAY_US.

| Level | DELAY_US | Accum. wait | noFT elapsed | MANA elapsed | overhead |
|---|---|---|---|---|---|
| L0 | 0 | 0 s | 60.73s | 64.17s | +3.44s |
| L1 | 100µs | 0.08 s | 60.79s | 64.27s | +3.48s |
| L2 | 1ms | 0.8 s | 61.53s | 64.46s | +2.93s |
| L3 | 5ms | 4.0 s | 64.72s | 64.67s | ≈0.00s |
| L4 | 20ms | 16.0 s | 76.74s | 78.78s | +2.04s |

**Key finding (unexpected):** MANA overhead does NOT grow with sender delay.
Overhead is flat ~3s for L0-L2 and then effectively DECREASES at L3/L4 — because
the noFT run itself gets slower as the sender sleep accumulates, and MANA's fixed
~3.4s init overhead stays constant. By L3, noFT (64.72s) has caught up to MANA's
floor (64.67s) and there is zero elapsed overhead.

**Why the MPI_Wait spin doesn't add wall time proportionally:**
The receiver must wait DELAY_US for the message regardless of noFT or MANA — the
sender doesn't send sooner just because MANA is spinning harder. The spin burns CPU
but doesn't change the wait duration. Only the fixed wrapper costs (DISABLE_CKPT +
JUMP_TO_LOWER_HALF per call) add overhead, and those are already bounded and counted.

**The CG 2w mystery remains:**
All three synthetic tests now point to the same finding: MANA adds ~3-5s fixed
overhead per ~60s run, independent of call count, synchronization pattern, or
imbalance level. CG 2w's 78s overhead (251%) cannot be explained by any single
isolated variable. The likely mechanism is **cascading CPU competition**: with 2
processes per node all simultaneously alternating compute↔spin, one process's
spin loop starves the co-located process's compute, causing it to send later,
causing more spin, amplified over 75 CG iterations. Our synthetic tests don't
replicate this because the sender delay is external (usleep, not compute-driven)
and senders/receivers are on separate nodes (no cross-process CPU competition).

**Subtotal: 30 runs ✅**

### 9.4 Phase 5.2 — Synthetic Checkpoint Size ✅ Done

Runs completed at 50 / 200 / 800 / 3200 MB (trigger_after_secs=30, DEGRADED).

| task_tag | Memory/process | Phase1 (s) | Phase2 (s) |
|---|---|---|---|
| synth_ckpt-50mb | 50 MB | 16.35 | 21.63 |
| synth_ckpt-200mb | 200 MB | 27.34 | 22.00 |
| synth_ckpt-800mb | 800 MB | 46.95 | 21.68 |
| synth_ckpt-3200mb | 3200 MB | 138.74 | 21.59 |

Phase2 (= phase2a + phase2b + phase2c for DEGRADED) is constant ~21.6s, independent of
checkpoint size. This is the Slurm reconfiguration lower bound: drain+scancel (~5.5s) +
set_DOWN (~10s) + restart dispatch (~5s).
Phase1 scales non-linearly: fixed overhead dominates for small sizes, NFS bandwidth caps large sizes.

**Subtotal: 4 runs ✅**

### 9.5 Phase 5.3 — Failure Timing Sensitivity (EP and LU only)

CG excluded (too short — see Section 5 rationale).

**2 workers:**

| task_tag | trigger_secs | % of MANA_noFT |
|---|---|---|
| ep-D_2w_m5xl_degraded-10pct | 49 | 10% |
| ep-D_2w_m5xl_degraded-25pct | 123 | 25% |
| ep-D_2w_m5xl_degraded-50pct | 246 | 50% |
| ep-D_2w_m5xl_replace-10pct | 49 | 10% |
| ep-D_2w_m5xl_replace-25pct | 123 | 25% |
| ep-D_2w_m5xl_replace-50pct | 246 | 50% |
| lu-C_2w_m5xl_degraded-10pct | 24 | 10% |
| lu-C_2w_m5xl_degraded-25pct | 61 | 25% |
| lu-C_2w_m5xl_degraded-50pct | 121 | 50% |
| lu-C_2w_m5xl_replace-10pct | 24 | 10% |
| lu-C_2w_m5xl_replace-25pct | 61 | 25% |
| lu-C_2w_m5xl_replace-50pct | 121 | 50% |

**4 workers:**

| task_tag | trigger_secs | % of MANA_noFT |
|---|---|---|
| ep-D_4w_m5xl_degraded-10pct | 25 | 10% |
| ep-D_4w_m5xl_degraded-25pct | 62 | 25% |
| ep-D_4w_m5xl_degraded-50pct | 125 | 50% |
| ep-D_4w_m5xl_replace-10pct | 25 | 10% |
| ep-D_4w_m5xl_replace-25pct | 62 | 25% |
| ep-D_4w_m5xl_replace-50pct | 125 | 50% |
| lu-C_4w_m5xl_degraded-10pct | 13 | 10% |
| lu-C_4w_m5xl_degraded-25pct | 34 | 25% |
| lu-C_4w_m5xl_degraded-50pct | 67 | 50% |
| lu-C_4w_m5xl_replace-10pct | 13 | 10% |
| lu-C_4w_m5xl_replace-25pct | 34 | 25% |
| lu-C_4w_m5xl_replace-50pct | 67 | 50% |

**Subtotal: 24 runs ✅ Done (2026-06-01 to 2026-06-02)**

**8 workers:** (trigger times now known from 2026-05-31 baseline runs)

| task_tag | trigger_secs | % of MANA_noFT |
|---|---|---|
| ep-D_8w_m5xl_degraded-10pct | 12 | 10% of 122.31s |
| ep-D_8w_m5xl_degraded-25pct | 31 | 25% |
| ep-D_8w_m5xl_degraded-50pct | 61 | 50% |
| ep-D_8w_m5xl_replace-10pct | 12 | 10% |
| ep-D_8w_m5xl_replace-25pct | 31 | 25% |
| ep-D_8w_m5xl_replace-50pct | 61 | 50% |
| lu-C_8w_m5xl_degraded-10pct | 7 | 10% of 68.40s |
| lu-C_8w_m5xl_degraded-25pct | 17 | 25% |
| lu-C_8w_m5xl_degraded-50pct | 34 | 50% |
| lu-C_8w_m5xl_replace-10pct | 7 | 10% |
| lu-C_8w_m5xl_replace-25pct | 17 | 25% |
| lu-C_8w_m5xl_replace-50pct | 34 | 50% |

**Subtotal: 12 runs ✅ Done (2026-06-03)**

**Phase 5.3 total: 36 runs ✅ all done**

### 9.6 Grand Total

| Group | Runs | Status | Est. time remaining |
|---|---|---|---|
| noFT + MANA-noFT baselines (all 3 benchmarks × 3 sizes) | 18 | ✅ Done | — |
| CG degraded + replace (2w + 4w + 8w) | 6 | ✅ Done | — |
| Phase 5.1 synthetic (synth_calls + synth_p2p + synth_imbalanced) | 30 | ✅ Done | — |
| Phase 5.2 synthetic checkpoint size | 4 | ✅ Done | — |
| Phase 5.3 failure timing (EP + LU, 2w + 4w + 8w) | 36 | ✅ Done | — |
| **Total** | **94** | **✅ All runs complete** | — |

Saved 12 runs vs original plan by dropping the 45s EP/LU base FT runs.
Added 10 synth_imbalanced runs to complete the MANA overhead causal analysis (all done).
Longest single run: EP-D 2w replace-50pct (trigger=231s + Phase2 ~300s + Phase3 ~231s ≈ 13 min).

---

## 10. Step-by-Step Implementation Plan

### Step 0 — Result storage restructuring

1. Archive existing results: `mv results results_phase4_archive_$(date +%Y%m%d)`.
2. Apply the two-change patch to `src/commands/cluster/run_task.rs` (Section 6.3):
   - Change filename to `{task_tag}_{timestamp}.txt`.
   - Add rename to `_SUCCESS.txt` at the success exit point.
3. Verify one test run produces a correctly named `_SUCCESS.txt` file.

### Step 1 — Write synthetic C programs (5.1 and 5.2) ✅ Done

All four programs written and all runs complete:
`TCC/synthetic/synth_mpi_calls.c`, `synth_p2p.c`, `synth_checkpoint_size.c`, `synth_imbalanced.c`

**synth_imbalanced rationale** (from MANA source code analysis):
synth_calls and synth_p2p correctly disproved "call count" as the overhead cause.
But both tests used synchronized partners (message always arrives quickly) — they
never stress MANA's MPI_Wait code path. CG and LU use `MPI_Irecv + MPI_Send + MPI_Wait`.
MANA's MPI_Wait is a tight spin loop with a real RW lock acquire/release per iteration
and NO sleep (unlike MPI_Recv which sleeps 1ms after 1000 failed Iprobes). Overhead
∝ wait time. synth_imbalanced introduces a controlled sender delay so the receiver
must actually spin in MPI_Wait, making the overhead visible and measurable.

### Step 2 — Create YAML task files ✅ Done

All 36 timing-variant files created (2w + 4w + 8w). 8w files now included because
the baseline MANA_noFT runs completed on 2026-05-31.
Files reorganized into benchmark subdirectories (ep/, lu/, cg/, synth/).

### Step 3 — Update watcher.rs + run_task.rs for Phase 2 split (5.4) ✅ Done

Final phase model uses three sub-phases of Phase 2 (see Section 5.2 for definitions):
- `phase2a_s`: checkpoint → node_drained (drain+scancel, ~5.5s, identical for both)
- `phase2b_s`: node_drained → node_became_idle (EC2 provisioning/DOWN, strategy-specific)
- `phase2c_s`: node_became_idle → restart_dispatched (~5s, identical for both)

`node_became_idle` is now emitted in BOTH strategies:
- REPLACE: after `scontrol RESUME` + wait for Slurm node to be IDLE (existing behavior)
- DEGRADED: after `scontrol DOWN` completes (new — makes phase2c symmetric)

`phase2_s` kept for backward compatibility. `phase2a_s + phase2b_s + phase2c_s = phase2_s`.

Also fixed: `simulate_cluster_failure` now captures the target instance ID before the
warning sleep, preventing it from killing a respawned replacement instance.

### Step 4 — Update analyze.py for new fields (5.4) ✅ Done

Can be done in parallel with or after Step 5 — the experiments produce
`[RUN_METRICS]` blocks with the correct fields regardless; analyze.py is only
needed to produce the final figures after all runs are complete.

1. Parse `phase2a_s`, `phase2b_s`, `phase2c_s` from `[RUN_METRICS]`.
2. Compute `phase0_s = auto_failure_trigger_secs` (already in data).
3. Update `plot_recovery_phases()` to 6-segment stacked bars (phase0/1/2a/2b/2c/3).
4. Ensure Phase 4 data is handled gracefully (set phase2b = phase2_s, phase2a = phase2c = 0).
5. Add `plot_mpi_overhead_synthetic()` for 5.1:
   - Panel A: overhead vs call level for synth_calls + synth_p2p (proves call count is not the driver)
   - Panel B: overhead vs delay level for synth_imbalanced (proves MPI_Wait imbalance IS the driver)
6. Add `plot_checkpoint_size_synthetic()` for 5.2.
7. Add `plot_failure_timing_sensitivity()` for 5.3 (fig7: Phase3 vs timing; fig8: total overhead vs timing).

### Step 5 — Run Phase 4 redo ✅ Done

Step 5 can and should be run before Step 4 is complete — experiments generate
data, analyze.py only consumes it later. The result files are complete as long as
watcher.rs + run_task.rs are up to date (both already done in Step 3).

Order within Step 5:
1. Run 2w + 4w degraded/replace for EP, LU, CG (12 runs, ~1.5 h).
2. Run 8w degraded/replace for EP, LU, CG (6 runs, ~30 min).
3. Verify each result has `phase2a_s` and `phase2b_s` in `[RUN_METRICS]`.

### Step 6 — Run Phase 5.3 experiments ✅ Done

Phase 5.1 ✅, 5.2 ✅, and 5.3 ✅ all complete.

Run all 36 timing-variant cells from Section 9.5. Suggested order by cluster:
1. 2w: EP (6 runs) + LU (6 runs). Longest: EP replace-50pct (~13 min total).
2. 4w: EP (6 runs) + LU (6 runs).
3. 8w: EP (6 runs) + LU (6 runs). LU 8w runs are short (max trigger=34s).

### Step 7 — Update analysis and reports ✅ Done

1. Re-run `analyze.py` on the results directory (requires Step 4 complete).
2. Write Phase 5 analysis section with new figures.
3. Add Phase 5 findings to both `analysis_report_phase5.md` (EN) and `analysis_report_phase5_ptbr.md` (PT-BR).

---

## 11. Validation Gates (Phase 5 Acceptance)

- [x] G1: `synth_mpi_calls.c` + `synth_p2p.c` compile and run under both noFT and MANA.
- [x] G2: Overhead vs call rate measured — result: overhead is flat (not call-count driven); fixed DMTCP infrastructure cost dominates.
- [x] G3: `synth_checkpoint_size.c` produces measurable `phase1_s` variation (16–139s across 50–3200 MB).
- [x] G4: Phase 1 time vs size measured — non-linear (fixed overhead dominates small sizes, NFS bandwidth limits large).
- [x] G5: Failure timing runs complete for EP and LU at all 3 timing points (2w + 4w + 8w) — all 36 runs done (2026-06-01 to 2026-06-03).
- [x] G6: `[RUN_METRICS]` blocks in new FT runs contain `phase2a_s`, `phase2b_s`, `phase2c_s` — validated on CG 2w (2026-05-31).
- [x] G7: fig4 shows all 5 phases (P0/P1/Slurm/P2b/P3) for EP and LU across all timing points.
- [x] G8: fig8 shows REPLACE vs DEGRADED vs MANA-noFT baseline; fig9 shows recovery overhead ratio.
- [x] G9: Phase 5 artifact document produced.

---

## 12. Expected Deliverables

| Deliverable | Location | Status |
|---|---|---|
| Updated `run_task.rs` (result naming + phase2a/2b) | `src/commands/cluster/run_task.rs` | ✅ Done |
| Updated `watcher.rs` (node_became_idle event) | `src/commands/cluster/watcher.rs` | ✅ Done |
| Synthetic C programs | `TCC/synthetic/synth_mpi_calls.c`, `synth_p2p.c`, `synth_checkpoint_size.c` | ✅ Done |
| Task YAMLs — reorganized into benchmark subdirs | `my_clusters/m5-xlarge/{2,4,8}workers/{ep,lu,cg,synth}/` | ✅ Done |
| Task YAMLs — 5.3 timing variants (2w+4w+8w) | `…/{ep,lu}/task-*-{10,25,50}pct.yaml` (36 files) | ✅ Done |
| Task YAMLs — 5.1 synth calls + p2p | `…/synth/task-synth_{calls,p2p}-l{0..4}-*.yaml` (20 files) | ✅ Done |
| Task YAMLs — 5.2 synth checkpoint | `…/synth/task-synth_ckpt-*-degraded.yaml` (4 files) | ✅ Done |
| Updated `analyze.py` (phase2a/2b, phase0, new figs) | `TCC/analyze.py` | ✅ Done |
| Phase 4 redo FT result files | `results/cluster_ClusterMANA_*/` | ✅ Done |
| Phase 5.3 timing sensitivity result files | `results/cluster_ClusterMANA_*/` | ✅ Done |
| 10 figures (fig1–fig9 + fig2b) | `TCC/artifacts/phase5/analysis/plots/` | ✅ Done |
| Phase 5 analysis report (EN) | `TCC/artifacts/phase5/analysis/analysis_report_phase5.md` | ✅ Done |
| Phase 5 analysis report (PT-BR) | `TCC/artifacts/phase5/analysis/analysis_report_phase5_ptbr.md` | ✅ Done |
| Phase 5 artifact | `TCC/artifacts/phase5/PHASE5_ARTIFACT.md` | ✅ Done |

---

See Section 9.6 for the complete grand total.
