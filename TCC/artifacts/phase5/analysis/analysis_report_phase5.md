# Phase 5 Analysis Report — Deepening the Fault-Tolerance Evaluation
### MANA Overhead Mechanisms · Checkpoint Size · Failure Timing Sensitivity
### Cluster: AWS m5.xlarge workers (2 / 4 / 8 nodes) · us-west-2

---

## 1. Overview

This report presents the complete Phase 5 experimental results. Phase 5 extends the
Phase 4 pilot campaign in three directions:

| Study | Purpose | Runs |
|---|---|---|
| **5.1 Synthetic MPI overhead** | Isolate whether call frequency or communication imbalance drives MANA overhead | 30 |
| **5.2 Synthetic checkpoint size** | Quantify how checkpoint image size drives Phase 1 (detect → checkpoint) time | 4 |
| **5.3 Failure timing sensitivity** | Measure how failure position in the job lifecycle affects REPLACE vs DEGRADED trade-off | 36 |
| **Baselines (Phase 4 redo)** | noFT and MANA-noFT for all 3 benchmarks × 3 cluster sizes with new instrumentation | 24 |

**Total: 94 successful runs.** All results were collected on AWS m5.xlarge spot workers
(4 vCPUs, 16 GB RAM) in us-west-2. The head node was a t3.large on-demand instance.
Each run used the `auto_test_failure` mechanism for reproducible failure injection.

New in Phase 5: the recovery now instruments **three sub-phases of Phase 2**:
- **Phase 2a** — Slurm drain + job cancel (~5.5 s, identical for both strategies)
- **Phase 2b** — Node reconfig: EC2 respawn for REPLACE (~122–157 s) vs `scontrol DOWN` for DEGRADED (~11 s)
- **Phase 2c** — New Slurm allocation + MANA coordinator setup (~5.3 s, identical for both)

---

## 2. MANA Overhead Without Failures

![MANA overhead per benchmark and worker count](plots/fig1_mana_overhead.png)

**Table 1 — MANA overhead (MANA-noFT vs noFT wall time):**

| Benchmark | 2 workers | 4 workers | 8 workers |
|---|---|---|---|
| CG-C | +233% | +59% | +73% |
| EP-D | +39% | +14% | +22% |
| LU-C | +62% | +56% | +52% |

These numbers should be read carefully. Each figure is the difference between two separate
runs — a noFT run and a MANA-noFT run — executed at different points in time on EC2 spot
instances. Both runs are affected by EC2 infrastructure noise (scheduling jitter, memory
bus variability, burst credit usage), so the measured overhead is the *total difference*
between two noisy measurements, not a clean isolation of MANA's cost.

This matters because the measured overhead does not show a clear pattern: it does not
grow consistently with worker count, and it does not follow a clear pattern across
benchmarks. LU-C is the most stable case (52–62% across all sizes), but even there the
values come from single-run samples per cell. With more repetitions, the true MANA
contribution could be isolated from noise — but that is outside the scope of this phase.

### 2.1 CG-C at 2 Workers — Anomalous Overhead

CG at 2 workers shows +233% overhead (34.6 s → 115.2 s), while the same benchmark at
4 workers shows only +59% (18.7 s → 29.7 s). This is a 6× difference in added overhead
(80.6 s vs 11.0 s) that is physically inconsistent with a benchmark that runs *more*
iterations per process at fewer workers.

All three synthetic studies (Section 4) converge on a ~4–5 s MANA overhead for isolated
programs — far below the 80 s seen here. The most plausible mechanism is a cascading
effect: at 2 workers, CG's matrix is split across 4 MPI processes all on 2 machines.
If MANA's per-call wrappers add a small compute delay per iteration, one process finishes
its work slightly later, causing its communication partner to wait. That wait burns CPU
that the slow process needs, slowing it further — and this amplifies over CG's 75
conjugate-gradient iterations. At 4/8 workers, the matrix partitions differently and
the bottleneck does not appear. Confirming this would require per-rank timing inside the
benchmark, which is outside this study's scope.

### 2.2 EP-D — Unreliable Single-Run Measurement

EP-D adds 131 s overhead at 2 workers, but only 40 s at 4w and 23 s at 8w. EP uses only
collective operations with no irregular P2P communication, so its MANA overhead should be
small and roughly constant. The large 2w figure most likely reflects EC2 infrastructure
noise on a single ~335 s run rather than a reproducible MANA characteristic. EP should
not be used to characterize MANA's overhead mechanism.

### 2.3 LU-C — Most Interpretable Result

LU-C shows 62% at 2w, 56% at 4w, 52% at 8w — the most consistent pattern across worker
counts. The slight downward trend is consistent with MANA's fixed per-process costs being
distributed across more parallel computation. LU is the most reliable benchmark for
overhead characterization: structured communication, stable runtime, consistent
across runs.

---

## 3. Strong Scaling

![Strong scaling — wall time vs worker count](plots/fig6_mana_scalability.png)

**Table 2 — Wall time (seconds) vs worker count, noFT and MANA-noFT:**

| Benchmark | noFT 2w | noFT 4w | noFT 8w | Speedup 2→8 | MANA 2w | MANA 4w | MANA 8w |
|---|---|---|---|---|---|---|---|
| CG-C | 34.6 | 18.7 | 12.1 | 2.9× | 115.2 | 29.7 | 20.9 |
| EP-D | 335.0 | 285.8 | 102.3 | 3.3× | 466.5 | 325.9 | 124.9 |
| LU-C | 149.8 | 87.3 | 47.4 | 3.2× | 242.6 | 136.0 | 71.9 |

All three benchmarks scale sub-linearly from 2 to 8 workers (ideal would be 4×). EP and LU
reach 3.2–3.3× speedup, which is reasonable for memory-bound MPI applications at small
cluster sizes. CG's lower scaling (2.9×) reflects its irregular communication pattern.

The key result here is that **MANA-noFT follows the same scaling curve as noFT**. The
overhead does not worsen disproportionately at higher worker counts. This means MANA's
checkpoint infrastructure does not introduce a scaling bottleneck: the per-process
overhead is roughly constant, so larger clusters do not pay more overhead per worker.

---

## 4. MANA Overhead Mechanisms — Synthetic Studies

The real benchmarks (CG, EP, LU) mix computation, communication, and cluster effects in
ways that make it hard to isolate *what* drives MANA overhead. Four synthetic programs
were designed to test one variable at a time.

### 4.1 Call Frequency Study

**What this tests:** MPI_Allreduce is a *collective* operation — every process contributes
a value, and all receive the global result (e.g. the sum) simultaneously. It is the
primary synchronization primitive in EP. The question is whether MANA's overhead grows
as the program calls MPI_Allreduce more frequently.

`synth_calls` does a fixed amount of computation (same total CPU work at every level)
and calls `MPI_Allreduce` at different frequencies, from 0 to 51,200 calls per run:

```c
for (long outer = 0; outer < TOTAL_OUTER; outer++) {
    for (long i = 0; i < INNER_ITERS; i++)
        x = x * 1.0000001 + 1e-10;          /* fixed compute per iteration */
    if (call_period > 0 && outer % call_period == 0) {
        MPI_Allreduce(&x, &result, 1, MPI_DOUBLE, MPI_SUM, MPI_COMM_WORLD);
        x += result * 1e-20;
    }
}
```

`synth_p2p` does the same but uses `MPI_Send + MPI_Recv` pairs instead. `MPI_Send`
blocks the caller until the message is delivered to the partner; `MPI_Recv` blocks the
caller until a message from the partner arrives. Unlike Allreduce, these are
point-to-point between specific process pairs, which is the pattern used by LU
(nearest-neighbor stencil):

```c
if (rank < nprocs / 2) {
    MPI_Send(&x, 1, MPI_DOUBLE, partner, 0, MPI_COMM_WORLD);
    MPI_Recv(&result, 1, MPI_DOUBLE, partner, 0, MPI_COMM_WORLD, &status);
} else {
    MPI_Recv(&result, 1, MPI_DOUBLE, partner, 0, MPI_COMM_WORLD, &status);
    MPI_Send(&x, 1, MPI_DOUBLE, partner, 0, MPI_COMM_WORLD);
}
```

The two branches prevent deadlock: lower-half ranks send first, upper-half ranks receive
first, so no rank ever blocks waiting to send while its partner is also blocked waiting
to send.

![MANA overhead vs MPI call frequency](plots/fig2_synth_calls.png)

**Table 3 — MANA overhead (added seconds) vs call count:**

| Level | Calls | synth_calls overhead | synth_p2p overhead |
|---|---|---|---|
| L0 | 0 | +2.1 s | +4.7 s |
| L1 | 800 | +4.3 s | +4.1 s |
| L2 | 3,200 | +4.4 s | +4.6 s |
| L3 | 12,800 | +4.3 s | +4.2 s |
| L4 | 51,200 | +1.6 s | +2.7 s |

**Finding:** MANA overhead is flat (~3 s) across all call levels and actually *decreases*
at L4. Call count is **not** the driver. The overhead comes from DMTCP infrastructure
startup costs (coordinator handshake, checkpoint-enable/disable wrappers at init/exit)
rather than per-call interception.

Note that `synth_p2p` is also flat despite using a different communication primitive.
This is because both partners reach their Send/Recv exchange at the same moment —
perfectly synchronized. The message arrives within MANA's first poll, so the sleep-and-
retry path never fires. This is a best-case scenario that does not represent CG or LU,
where computation time between communication rounds causes partners to be misaligned.

### 4.2 Communication Imbalance Study

**What this tests:** CG and LU use `MPI_Irecv + MPI_Wait` rather than blocking `MPI_Recv`.
`MPI_Irecv` posts a non-blocking receive request and returns immediately, allowing the
process to do other work. `MPI_Wait` later blocks until the posted receive completes.

MANA's `MPI_Wait` implementation is a tight spin loop — it continuously checks whether
the message has arrived without sleeping:

```c
/* MANA's internal MPI_Wait wrapper (simplified) */
while (!flag) {
    DMTCP_PLUGIN_DISABLE_CKPT();   /* acquire checkpoint read-write lock */
    MPI_Test_internal(..., &flag); /* bypass MANA's wrappers → call real MPI_Test */
    DMTCP_PLUGIN_ENABLE_CKPT();    /* release checkpoint lock */
}
```

`MPI_Test_internal` is a DMTCP-internal function that calls the real underlying MPI
without going through MANA's own interception layer. This bypass is necessary to avoid
infinite recursion (calling `MPI_Test` from inside a MANA wrapper would re-enter MANA).
The expensive part is not the test itself but the lock acquire/release pair around it:
these run at thousands of iterations per second for however long the message is delayed,
burning CPU while the receiver waits.

`synth_imbalanced` uses `MPI_Irecv + MPI_Wait` with a controlled sender delay to force
the receiver into this spin loop for a known duration. The two roles (`is_sender`) are
necessary to avoid deadlock and to clearly separate who delays from who waits:

- **Sender** (lower-half ranks): sleeps for `DELAY_US`, then sends. This process is the
  one creating the imbalance — it deliberately arrives late at the exchange.
- **Receiver** (upper-half ranks): posts `MPI_Irecv` immediately (no delay), then calls
  `MPI_Wait`. Because the sender is sleeping, the receiver spins inside MANA's wait loop
  for exactly `DELAY_US` — this is the behavior under test.

After the main exchange, the receiver sends a reply and the sender does its own
`Irecv + Wait` for that reply. At that point the receiver has already sent, so the sender
completes instantly — no spin on the sender side.

```c
if (is_sender) {
    if (DELAY_US > 0) usleep(DELAY_US);   /* delay → create imbalance */
    MPI_Send(&x, 1, MPI_DOUBLE, partner, 0, MPI_COMM_WORLD);
    MPI_Irecv(&recv_buf, 1, MPI_DOUBLE, partner, 1, MPI_COMM_WORLD, &request);
    MPI_Wait(&request, &status);          /* reply already sent — completes fast */
} else {
    MPI_Irecv(&recv_buf, 1, MPI_DOUBLE, partner, 0, MPI_COMM_WORLD, &request);
    MPI_Wait(&request, &status);          /* spins for DELAY_US — this is the test */
    MPI_Send(&x, 1, MPI_DOUBLE, partner, 1, MPI_COMM_WORLD);
}
```

![MANA overhead vs sender delay (communication imbalance)](plots/fig2b_synth_imbalanced.png)

**Table 4 — synth_imbalanced results:**

| Level | Sender delay | Accum. wait | noFT elapsed | MANA elapsed | Overhead |
|---|---|---|---|---|---|
| L0 | 0 µs | 0 s | 63.3 s | 67.4 s | +4.1 s |
| L1 | 100 µs | 0.08 s | 62.9 s | 67.9 s | +5.0 s |
| L2 | 1 ms | 0.8 s | 62.9 s | 68.8 s | +5.9 s |
| L3 | 5 ms | 4.0 s | 67.4 s | 72.0 s | +4.6 s |
| L4 | 20 ms | 16.0 s | 78.4 s | 83.1 s | +4.7 s |

**Finding:** MANA overhead stays at ~4–6 s regardless of sender delay. At L3 and L4 the
noFT baseline itself grows (because the program genuinely waits for the slow sender), but
the *difference* between MANA and noFT stays constant. The spin loop does burn CPU, but
not more than the sender's sleep already costs.

**What this means for CG at 2 workers:** All synthetic tests show MANA adds ~4–5 s for
isolated programs. CG's 80 s anomaly at 2w therefore cannot come from call frequency or
simple communication imbalance alone. The missing ingredient is the *cascade*: in CG,
the sender delay is not a fixed external sleep — it is caused by MANA's own wrapper
slightly slowing computation, which feeds back into the next communication round,
amplifying over 75 iterations. This cascade requires both processes to share CPU
resources (possible on the same instance), which the synthetic tests avoid by design
(sender and receiver are on separate nodes). This remains a hypothesis; confirming it
would require per-rank profiling inside the NAS benchmark.

### 4.3 Checkpoint Image Size Study

**What this tests:** How long does Phase 1 (failure detected → checkpoint written to EFS)
take as the process memory footprint grows? `synth_checkpoint_size` allocates a known
amount of memory per MPI process, forces the OS to actually map all pages with `memset`
(so the checkpoint image truly reflects the requested size), and then runs for 120 s
while keeping the buffer live so it stays in the checkpoint:

```c
long n = (MEM_MB * 1024L * 1024L) / sizeof(double);
double *buf = (double *)malloc(n * sizeof(double));
memset(buf, 0x42, n * sizeof(double));   /* force OS to map all pages */

double t_start = MPI_Wtime();
volatile long counter = 0;

while (MPI_Wtime() - t_start < TARGET_SECS) {
    buf[counter % n] += 1.0;    /* touch buffer so it stays live in the checkpoint */
    counter++;
    if (counter % 5000000L == 0)
        MPI_Barrier(MPI_COMM_WORLD);
}
/* MANA injects the checkpoint signal at trigger_after_secs=30 */
```

Without the `memset`, Linux would lazily allocate pages and the checkpoint image would be
smaller than `MEM_MB` — the test would not measure what it claims. Without touching the
buffer in the loop, a compiler or OS could discard the pages before the checkpoint fires.

![Phase 1 time vs checkpoint image size](plots/fig3_synth_ckpt.png)

**Table 5 — Phase 1 and Phase 2 times vs memory per process:**

| Memory/process | Phase 1 (s) | Phase 2 (s) | Notes |
|---|---|---|---|
| 50 MB | 16.4 s | 21.6 s | small image |
| 200 MB | 27.3 s | 22.0 s | — |
| 800 MB | 47.0 s | 21.7 s | — |
| 3,200 MB | 138.7 s | 21.6 s | approaches EFS bandwidth limit |

**Finding 1:** Phase 1 time scales **linearly** with memory per process. The linear fit
in Figure 3 confirms this. The practical implication is that Phase 1 time is predictable:
a user who knows their job's per-process memory footprint can estimate how long a
checkpoint write will take.

**Finding 2:** Phase 2 is **constant at ~21.6 s regardless of checkpoint size**. This
is the Slurm reconfiguration floor: drain + cancel (~5.5 s) + scontrol DOWN (~10 s) +
new Slurm allocation + MANA coordinator setup (~5.3 s). This lower bound cannot be
reduced without changing the Slurm orchestration.

**Connection to real benchmarks:** EP-D processes allocate less memory than LU-C or CG-C,
which matches Phase 1 times measured in FT runs: ~16–17 s for EP vs ~26–37 s for LU and
CG. The checkpoint size study confirms the difference is memory footprint, not benchmark
complexity.

---

## 5. Recovery Phase Analysis — CG Short-Job Case

![CG short-job FT wall time breakdown](plots/fig5_cg_short_job.png)

CG-C is excluded from the failure timing sensitivity study (Section 6) because its
10–35 s noFT runtime makes timing variants meaningless. However, it is the clearest
illustration of a key structural fact: **for short jobs, recovery overhead dominates
total wall time**, and all the phase mechanics are visible in their simplest form.

**Table 6 — CG-C FT wall time breakdown (base trigger ~3 s):**

| Workers | Strategy | P0 | P1 | Slurm (P2a+P2c) | P2b | P3 | Total |
|---|---|---|---|---|---|---|---|
| 2w | REPLACE | 3.0 s | 26.5 s | 10.8 s | 157.0 s | 42.0 s | **243.8 s** |
| 2w | DEGRADED | 3.0 s | 26.4 s | 10.7 s | 10.9 s | 109.5 s | **165.8 s** |
| 4w | REPLACE | 3.0 s | 26.6 s | 10.7 s | 125.2 s | 26.4 s | **201.1 s** |
| 4w | DEGRADED | 3.0 s | 26.5 s | 11.0 s | 10.9 s | 42.0 s | **100.9 s** |
| 8w | REPLACE | 3.0 s | 16.7 s | 10.8 s | 122.1 s | 21.1 s | **203.1 s** |
| 8w | DEGRADED | 3.0 s | 16.7 s | 10.8 s | 10.9 s | 26.4 s | **98.3 s** |

**Reading the phases:**

- **P0 (pre-failure):** 3 s across all rows — the failure was triggered early and is the
  same for both strategies. This is set by `auto_failure_trigger_secs`.
- **P1 (checkpoint write):** ~26.5 s at 2w/4w, rises to 36.7 s at 8w because more
  processes = larger total checkpoint image on EFS. Identical for both strategies at the
  same cluster size — confirms that P1 depends on memory footprint, not strategy.
- **Slurm+MANA overhead (P2a+P2c):** ~10.7–10.8 s, constant across all configurations
  and both strategies. This is the fixed coordination cost for Slurm and MANA to agree on
  the new cluster configuration.
- **P2b (node reconfig):** The key differentiator. REPLACE pays 122–157 s for EC2 to
  terminate and re-provision an instance. DEGRADED pays ~11 s for `scontrol DOWN` to
  mark the failed node as unavailable. This ~111–146 s gap is the direct cost of the
  replace strategy.
- **P3 (remaining computation):** Larger for DEGRADED because it continues with one fewer
  worker. At 2w this is significant (109.5 s vs 42.0 s — DEGRADED loses half its
  capacity), but at 4w and 8w the extra P3 cost is smaller (42.0 vs 26.4 at 4w,
  26.4 vs 21.1 at 8w).

For very short jobs like CG, the FT overhead is 3–17× the original job duration.
MANA-based fault tolerance is most cost-effective for long-running jobs where recovery
phases represent a small fraction of total wall time.

---

## 6. Failure Timing Analysis — Phase-by-Phase Study

![Full FT wall time by failure timing](plots/fig4_timing_phases.png)

This is the central analysis of Phase 5. EP-D and LU-C were each run with failure
injected at 10%, 25%, and 50% of the MANA-noFT wall time, at 2, 4, and 8 workers.
Rather than focusing only on which strategy wins each case, this section examines what
each phase reveals about the recovery mechanics.

### 6.1 Phase 0 — Pre-Failure Computation

Phase 0 grows as the failure trigger moves from 10% to 50%: the job runs longer before
the failure fires. P0 is **identical for REPLACE and DEGRADED** within each timing group,
since the failure is triggered by the same `auto_failure_trigger_secs` setting. This is
visible in Figure 4 as the equal-height teal base on each bar pair.

The growing P0 also explains why later failures generally produce lower total wall times
for both strategies: more productive work was done before the failure, leaving less
work for P3.

### 6.2 Phase 1 — Checkpoint Write

**P1 is constant within each benchmark × worker-count combination, and equal across
strategies.** It depends only on the process memory footprint at the time of failure.

| Configuration | P1 (approx.) |
|---|---|
| EP-D, 2w / 4w / 8w | ~16 s |
| LU-C, 2w / 4w / 8w | ~27 s |
| CG-C, 2w / 4w / 8w | ~26 s |

This confirms the synthetic checkpoint study: Phase 1 is a function of memory footprint,
not of failure timing, cluster size, or strategy. It is the predictable fixed cost of
writing the checkpoint image to EFS.

### 6.3 Phase 2a + Phase 2c — Slurm and MANA Coordination

These two sub-phases (drain + cancel, then new Slurm allocation + MANA coordinator
restart) together take ~10.8–11.0 s for all configurations and both strategies. They
represent the overhead of communicating with the cluster control plane to reconfigure
Slurm for the restart. This cost is essentially the same regardless of whether the node
is being replaced or dropped, making it strategy-independent.

### 6.4 Phase 2b — Node Reconfiguration (The Differentiator)

**P2b is where REPLACE and DEGRADED fundamentally diverge:**

- **REPLACE:** must wait for EC2 to terminate the spot instance, provision a new one,
  boot it, install Slurm, and bring it to IDLE state. This takes 122–157 s and is driven
  by AWS EC2 provisioning latency — independent of cluster size or failure timing.
- **DEGRADED:** only runs `scontrol update NodeName=... State=DOWN` to mark the failed
  node as unavailable. The remaining nodes continue without waiting for anything. This
  takes ~11 s.

The P2b gap (~111–146 s) is **the single most important factor** in this study. It
determines which strategy wins in most scenarios.

**Table 7 — P2b times across configurations:**

| Benchmark | Workers | P2b REPLACE | P2b DEGRADED | Gap |
|---|---|---|---|---|
| EP-D | 2w | ~125 s | ~11 s | ~114 s |
| EP-D | 4w | ~118 s | ~11 s | ~107 s |
| EP-D | 8w | ~157 s | ~11 s | ~146 s |
| LU-C | 2w | ~124 s | ~11 s | ~113 s |
| LU-C | 4w | ~126 s | ~11 s | ~115 s |
| LU-C | 8w | ~188 s | ~11 s | ~177 s |

### 6.5 Phase 3 — Remaining Computation

P3 is the most timing-sensitive phase. It is determined by two factors:

1. **How much work remains** at the time of failure — shrinks as the failure trigger moves
   from 10% to 50%.
2. **Capacity after recovery** — REPLACE restarts with the original N workers; DEGRADED
   restarts with N−1 workers, so its P3 takes proportionally longer.

The P3 gap between strategies depends on cluster size:

- **At 2 workers:** losing 1 of 2 workers means DEGRADED runs P3 at ~2× the REPLACE
  time (for EP's embarrassingly-parallel workload). This is a large penalty.
- **At 4 workers:** losing 1 of 4 means ~33% slower — smaller penalty.
- **At 8 workers:** losing 1 of 8 means ~14% slower — almost negligible.

This is why the P2b gap matters more at larger cluster sizes: DEGRADED's P3 penalty
shrinks faster than REPLACE's P2b cost stays constant.

### 6.6 EP-D Full Phase Breakdown

**Table 8 — EP-D mean phase times by configuration (seconds):**

| Workers | Trigger | Strategy | P0 | P1 | P2a+2c | P2b | P3 | Total |
|---|---|---|---|---|---|---|---|---|
| 2w | 10% (~46s) | REPLACE | 46 | 20 | 11 | 125 | 318 | **519** |
| 2w | 10% (~46s) | DEGRADED | 46 | 16 | 11 | 11 | 453 | **537** |
| 2w | 25% (~116s) | REPLACE | 116 | 20 | 11 | 125 | 278 | **549** |
| 2w | 25% (~116s) | DEGRADED | 116 | 16 | 11 | 11 | 378 | **533** |
| 2w | 50% (~231s) | REPLACE | 231 | 20 | 11 | 125 | 201 | **588** |
| 2w | 50% (~231s) | DEGRADED | 231 | 16 | 11 | 11 | 270 | **538** |
| 4w | 10% (~32s) | REPLACE | 32 | 16 | 11 | 118 | 137 | **314** |
| 4w | 10% (~32s) | DEGRADED | 32 | 16 | 11 | 11 | 219 | **289** |
| 8w | avg | REPLACE | ~35 | ~20 | 13 | 157 | ~83 | **307** |
| 8w | avg | DEGRADED | ~35 | ~16 | 11 | 11 | ~105 | **177** |

At 2w/10%, P3_DEGRADED (453 s) − P3_REPLACE (318 s) = 135 s > P2b gap (114 s), so
REPLACE wins by a narrow margin. At every other configuration, the P2b gap exceeds the
P3 penalty and DEGRADED wins.

### 6.7 LU-C Full Phase Breakdown

**Table 9 — LU-C mean phase times by configuration (seconds):**

| Workers | Trigger | Strategy | P0 | P1 | P2a+2c | P2b | P3 | Total |
|---|---|---|---|---|---|---|---|---|
| 2w | 10% (~24s) | REPLACE | 24 | 27 | 11 | 124 | 214 | **370** |
| 2w | 10% (~24s) | DEGRADED | 24 | 27 | 11 | 11 | 233 | **305** |
| 2w | 50% (~121s) | REPLACE | 121 | 27 | 11 | 124 | 66 | **329** |
| 2w | 50% (~121s) | DEGRADED | 121 | 27 | 11 | 11 | 130 | **299** |
| 4w | avg | REPLACE | ~37 | ~30 | 11 | 126 | ~73 | **277** |
| 4w | avg | DEGRADED | ~37 | ~27 | 11 | 11 | ~108 | **194** |
| 8w | avg | REPLACE | ~19 | ~27 | 11 | 188 | ~58 | **303** |
| 8w | avg | DEGRADED | ~19 | ~27 | 11 | 11 | ~73 | **140** |

LU shows no crossover at any configuration. Unlike EP (embarrassingly parallel),
LU's structured stencil communication means DEGRADED's P3 penalty for losing one worker
is smaller — the remaining workers have better cache locality with fewer neighbors.
DEGRADED wins at all 9 tested LU configurations.

---

## 7. Strategy Comparison — Tendencies and Trade-offs

![REPLACE vs DEGRADED total wall time with MANA-noFT baseline](plots/fig8_strategy_comparison.png)

### 7.1 Scope of This Study

An important constraint shaped which workloads were tested: all runs had to complete
their checkpoint before the spot instance's 2-minute interruption warning expired. This
means we deliberately studied short-to-medium jobs — long enough to show meaningful FT
behavior, but bounded by checkpoint write time. Very long workloads (hours-long HPC jobs)
were not evaluated.

This limitation is actually useful context for interpreting the results: all findings
below apply to the short-to-medium regime. The tendency analysis in the following
sections projects what would happen for longer jobs.

### 7.2 Fixed vs Variable Costs of Recovery

The phase study reveals a key structural property of both strategies: most of the
recovery cost is **fixed** — it does not depend on how long the job runs or when the
failure occurs.

| Phase | Cost | Varies with? |
|---|---|---|
| P1 (checkpoint write) | ~16–37 s | memory footprint only |
| P2a + P2c (Slurm/MANA coordination) | ~11 s | nothing — always the same |
| P2b REPLACE (EC2 provisioning) | ~122–157 s | AWS latency only |
| P2b DEGRADED (scontrol DOWN) | ~11 s | nothing — always the same |
| P3 (remaining computation) | varies | failure timing + capacity loss |

The only phase that varies with job length is P3. For both strategies, P3 shrinks as
the failure occurs later in the job. This means: **the longer a job runs before failing,
the smaller the fraction of wall time spent on recovery** — regardless of strategy.

### 7.3 Tendency for Longer Jobs

Since P1, P2a, and P2c are constant, and P2b is fixed per strategy, the total recovery
overhead as a fraction of job wall time *decreases* as jobs get longer. Section 8 shows
this numerically — at 50% trigger, DEGRADED at 4w recovers with only 49% overhead.

For REPLACE, the fixed P2b cost of ~122–157 s is always present. For short jobs (total
wall time ~100–300 s, as tested here), this represents 40–80% of the total. For
hypothetical very long jobs (several hours), it would shrink to a small percentage — but
REPLACE would still always pay it.

For DEGRADED, the P3 penalty grows with job length. At 2 workers, running with half the
capacity doubles remaining computation time. For a job that runs for hours with an early
failure, this penalty would be large and sustained. **The crossover point** — where
REPLACE becomes better than DEGRADED — would occur when:

```
P2b_REPLACE − P2b_DEGRADED  <  P3_DEGRADED − P3_REPLACE
  (~114–177 s fixed gap)     <  (depends on job length × capacity fraction lost)
```

At 2 workers, where 50% capacity is lost, this crossover happens for early failures in
long jobs. At 4+ workers (≤ 25% capacity loss), this crossover is much harder to reach.

### 7.4 Multiple Failures

Only single-failure scenarios were tested in this study. For workloads that may experience
multiple successive spot interruptions, DEGRADED and REPLACE behave differently:

- **REPLACE** always returns to the original cluster size. Each failure costs the same
  fixed P2b (~122–157 s). Multiple failures add that cost repeatedly, but the cluster
  capacity never degrades.
- **DEGRADED** continues with fewer workers after each failure. A second failure on an
  already-degraded cluster removes another worker, and P3 grows proportionally. For
  clusters starting at 4w or 8w, this is tolerable for one or two failures. For 2w
  clusters, a second failure would leave only one worker, which may be unable to
  continue depending on the MPI decomposition.

This is an untested regime, but the tendency is clear: **REPLACE is more robust under
repeated failures**, while **DEGRADED's advantage erodes with each additional failure**.

### 7.5 Summary: When to Use Each Strategy

| Scenario | Recommended strategy | Reason |
|---|---|---|
| Short job (< 1–2 min noFT) | Neither — noFT on-demand may be preferable | Recovery overhead dominates; spot savings don't cover FT cost |
| Medium/long job, ≥ 4 workers | **DEGRADED** | Capacity loss ≤ 25%; P2b advantage decisive; wins in all tested cases |
| Medium job, 2 workers, single failure | **DEGRADED** (usually) | Wins at 25%/50% timing; only loses at 10% for EP-type workloads |
| Long job, 2 workers, early failure expected | **REPLACE** | At 2w, capacity loss is severe; REPLACE avoids the P3 penalty |
| Multiple failures expected | **REPLACE** | Capacity never degrades; each failure costs the same |

---

## 8. Recovery Overhead Ratio

![Recovery overhead as % of total FT wall time](plots/fig9_recovery_overhead_ratio.png)

The recovery overhead ratio measures what fraction of total FT wall time was caused by
the failure and its recovery — everything except the pre-failure computation (P0):

`recovery_overhead_pct = (ft_wall_time − P0) / ft_wall_time × 100`

**Table 10 — Recovery overhead as % of total FT wall time (EP-D):**

| Workers | Strategy | 10% | 25% | 50% |
|---|---|---|---|---|
| 2w | REPLACE | 91% | 79% | 61% |
| 2w | DEGRADED | 91% | 78% | 57% |
| 4w | REPLACE | 90% | 79% | 60% |
| 4w | DEGRADED | 89% | 72% | 49% |
| 8w | REPLACE | 93% | 88% | 82% |
| 8w | DEGRADED | 93% | 82% | 65% |

**Key observations:**

1. **Early failure → high overhead ratio for both strategies.** When failure fires at 10%,
   recovery accounts for ~90% of total wall time regardless of strategy. P0 is small so
   nearly all elapsed time is spent on recovery phases.

2. **Later failure → lower overhead ratio.** At 50% trigger, DEGRADED at 4w has only 49%
   overhead — more than half the total wall time is productive computation. This is the
   strongest economic argument for fault tolerance: with good timing, recovery is a minor
   fraction of the job.

3. **At 8w, REPLACE overhead stays above 80% even at 50%.** Because the job is short
   (noFT ~102 s at 8w), the fixed EC2 provisioning cost (~157 s) represents most of the
   total. This confirms the finding from CG: for short jobs on large clusters, REPLACE is
   very expensive relative to job duration.

4. **DEGRADED and REPLACE have nearly identical overhead at 10%** for all configurations.
   The strategies diverge at 25% and 50%, where REPLACE's P2b contributes a growing
   fraction of remaining time.

The ratio curves in Figure 9 show a clear downward trend from 10% to 50% for all
configurations. Extrapolating: for jobs that take much longer than tested here, the ratio
would continue to fall, eventually making FT overhead negligible as a fraction of total
time.

---

## 9. Economic Analysis

Figure 7 shows cost per run broken down by fault timing (10%, 25%, 50%) for each
benchmark, strategy, and worker count. The cost model compares two scenarios:

- **noFT** must run on **on-demand** instances — a spot interruption without fault
  tolerance loses all progress and requires a full restart from scratch.
- **REPLACE and DEGRADED** can use **spot instances** (~70% cheaper) because MANA
  handles interruptions automatically and the job resumes from checkpoint.

![Cost per run — spot with FT vs on-demand without FT](plots/fig7_cost.png)

**Table 11 — Estimated cost per run (USD), 25% fault timing (representative):**

| Benchmark | Workers | noFT on-demand | REPLACE spot | DEGRADED spot | REPLACE vs noFT | DEGRADED vs noFT |
|---|---|---|---|---|---|---|
| CG-C | 2w | $0.0045 | $0.0136 | $0.0092 | +202% costlier | +105% costlier |
| CG-C | 4w | $0.0044 | $0.0177 | $0.0089 | +301% costlier | +101% costlier |
| CG-C | 8w | $0.0055 | $0.0311 | $0.0150 | +470% costlier | +175% costlier |
| EP-D | 2w | $0.0435 | $0.0305 | $0.0297 | **−30% saving** | **−32% saving** |
| EP-D | 4w | $0.0676 | $0.0339 | $0.0260 | **−50% saving** | **−62% saving** |
| EP-D | 8w | $0.0460 | $0.0555 | $0.0277 | +21% costlier | **−40% saving** |
| LU-C | 2w | $0.0194 | $0.0193 | $0.0169 | ≈0% break-even | −13% saving |
| LU-C | 4w | $0.0206 | $0.0249 | $0.0173 | +21% costlier | −16% saving |
| LU-C | 8w | $0.0213 | $0.0470 | $0.0208 | +121% costlier | ≈0% break-even |

**Key findings:**

1. **CG-C: FT is never economically justified.** Recovery overhead (103–311 s) is 6–57×
   the base job duration (12–35 s). Spot pricing cannot compensate. Short jobs have no
   economic case for MANA-based FT.

2. **EP-D DEGRADED saves 32–62% vs noFT on-demand at all worker counts.**
   EP scales well: 8 workers runs 2.8× faster than 4 workers, making 8w on-demand even
   cheaper than 4w on-demand ($0.046 vs $0.068). DEGRADED at any scale beats on-demand.

3. **EP-D REPLACE at 8 workers costs more than noFT on-demand.** EP-D at 8 workers
   runs only ~102 s; REPLACE adds ~260 s of EC2 provisioning. The recovery time overwhelms
   the spot discount at this scale. Fault timing matters: REPLACE at 10% costs $0.040
   (cheaper than noFT), but at 25% costs $0.055 (more expensive). Timing is significant.

4. **LU-C DEGRADED saves 13–16% at 2w and 4w but breaks even at 8w.**
   LU-C at 8 workers runs only 47 s. Even DEGRADED adds ~89 s (total ~136 s), and eight
   spot workers cost just enough to match eight on-demand workers at 47 s.

5. **LU-C REPLACE costs 2× more than noFT on-demand at 8 workers.** With a 47 s base
   job and ~260 s REPLACE recovery, the runtime is 6× longer. Spot discount (~3.3×
   cheaper per-hour) is insufficient. For short LU-C runs at scale, noFT on-demand is
   the economically superior choice.

6. **Fault timing affects cost significantly for FT strategies.** Earlier failures (10%)
   mean less P0 compute but the same fixed recovery cost, so total cost is dominated by
   recovery. Later failures (50%) amortize recovery over more productive P0 time. This
   effect is largest for REPLACE at 8 workers (see Figure 7).

---

## 10. Conclusions

### 10.1 MANA Overhead Characterization

The measured overhead (Table 1) varies between 14% and 233% across benchmarks and
cluster sizes, without a consistent pattern. Both the noFT and MANA-noFT measurements
are subject to EC2 infrastructure noise, so the computed difference is a rough estimate
rather than a clean MANA characterization. The synthetic studies establish that MANA
adds a fixed ~4–5 s infrastructure overhead per run for isolated programs, but this
floor is overwhelmed in the real benchmarks by benchmark-specific effects.

LU-C (52–62%, stable across worker counts) is the most reliable estimate.
CG-C at 2 workers (+233%) is likely a cascading imbalance effect specific to that matrix
partition. EP-D overhead is unreliable from single-run measurements. More repetitions per
cell would be needed for a rigorous overhead characterization.

### 10.2 Checkpoint Write Time Scales Linearly with Memory

Phase 1 time scales linearly with per-process memory footprint (confirmed by the
synthetic checkpoint study). This allows P1 to be predicted from job memory requirements.
Phase 2 has a hard lower bound of ~21.6 s from Slurm and MANA coordination that cannot
be reduced without redesigning the orchestration layer.

### 10.3 Recovery Phase Mechanics

The per-phase breakdown shows clear structure:

- **P1, P2a, P2c** are fixed costs, determined by memory footprint and cluster
  coordination time respectively. Neither strategy can avoid them.
- **P2b** is the primary differentiator: ~11 s for DEGRADED vs ~122–157 s for REPLACE.
  This gap drives most of the results.
- **P3** is the secondary factor: REPLACE restarts at full capacity; DEGRADED continues
  with reduced capacity. The P3 penalty is largest at 2 workers (50% capacity loss) and
  negligible at 8 workers (12.5% capacity loss).

### 10.4 Strategy Recommendations

**DEGRADED_RESUME is the recommended default** for the workloads evaluated (medium-length
jobs, 4–8 workers). It wins decisively in all tested configurations at 4w and 8w, and
in most configurations at 2w.

**REPLACE_RESUME is better** when: (a) the cluster has only 2 workers and early failure
is expected (EP-type workloads), or (b) multiple successive failures are expected and
maintaining full cluster capacity is critical.

**For very long jobs** (hours), the tendency favors REPLACE at 2 workers as the P3
penalty accumulates. At 4+ workers the P2b gap is large enough that DEGRADED would
likely still win even for long runs.

**For very short jobs** (< 1–2 min noFT), neither strategy is economically justified —
recovery overhead exceeds the spot pricing discount.

### 10.5 Limitations and Future Work

1. **Single repetition per cell** for most configurations. Statistical confidence
   requires 3–5 runs per cell to separate EC2 noise from MANA behavior.
2. **CG-C anomaly at 2 workers** is unresolved. Per-rank timing instrumentation inside
   the NAS benchmark would confirm or refute the cascading imbalance hypothesis.
3. **Multiple successive failures** were not tested. DEGRADED's capacity degradation
   under repeated failures is a critical open question.
4. **Only m5.xlarge instances** were evaluated. Different instance types would change
   Phase 1 times and MANA overhead factors.
5. **EFS as the checkpoint store** introduces variable latency. A high-performance
   parallel filesystem would reduce Phase 1 times substantially.
6. **No very long workloads** were tested. The crossover point where REPLACE beats
   DEGRADED at 2 workers for long jobs was not directly measured.

---

*Report generated from 94 valid runs collected 2026-05-30 to 2026-06-03.*
*All figures generated by `TCC/analyze.py`. Raw data: `results_raw.csv`. Aggregated stats: `summary.md`.*
