# Phase 5 Analysis Report: Deepening the Fault-Tolerance Evaluation
### MANA Overhead Mechanisms · Checkpoint Size · Failure Timing Sensitivity
### Cluster: AWS m5.xlarge workers (2 / 4 / 8 nodes) · us-west-2

---

## 1. Overview

This report presents the complete Phase 5 experimental results. Phase 5 extends the
Phase 4 pilot campaign in three directions:

| Study | Purpose | Runs |
|---|---|---|
| **5.1 Synthetic MPI overhead** | Isolate whether call frequency or communication imbalance drives MANA overhead | 90 |
| **5.2 Synthetic checkpoint size** | Quantify how checkpoint image size drives Phase 1 (detect to checkpoint) time | 12 |
| **5.3 Failure timing sensitivity** | Measure how failure position in the job lifecycle affects REPLACE vs DEGRADED trade-off | 107 |
| **Baselines (Phase 4 redo)** | noFT and MANA-noFT for all 3 benchmarks × 3 cluster sizes with new instrumentation | 54 |

**Total: 282 valid runs**, collected over 3 repetitions per configuration.
All results were collected on AWS m5.xlarge spot workers in us-west-2. The head node was
a t3.large on-demand instance. Each run used the `auto_test_failure` mechanism for
reproducible failure injection. Values in this report are expressed as mean ± standard
deviation across repetitions.

**Table 0: Cluster instance specifications:**

| Role | Instance | vCPUs | RAM | OS | On-demand (us-west-2) | Spot (us-west-2) | Spot discount |
|---|---|---|---|---|---|---|---|
| Worker (FT runs) | m5.xlarge | 4 | 16 GB | Amazon Linux 2 | $0.192/hr | $0.0585/hr | ~70% |
| Head node | t3.large | 2 | 8 GB | Amazon Linux 2 | $0.0832/hr | (on-demand only) | n/a |

Spot prices reflect observed averages in us-west-2 during the experiment window (queried
via `aws ec2 describe-spot-price-history`). Spot instances can be interrupted with a
2-minute warning, which is the interruption scenario this study tests.

New in Phase 5: the recovery now instruments **three sub-phases of Phase 2**:
- **Phase 2a** (Slurm drain and job cancel, ~5.5 s, identical for both strategies)
- **Phase 2b** (node reconfig: EC2 respawn for REPLACE, ~120-130 s; or `scontrol DOWN` for DEGRADED, ~11 s)
- **Phase 2c** (new Slurm allocation and MANA coordinator setup, ~5.3 s, identical for both)

---

## 2. MANA Overhead Without Failures

![MANA overhead per benchmark and worker count](plots/fig1_mana_overhead.png)

**Table 1: MANA overhead (MANA-noFT vs noFT wall time, N=3 per cell):**

| Benchmark | 2 workers | 4 workers | 8 workers |
|---|---|---|---|
| CG-C | +222% (34.1 to 109.8 s) | +55% (20.2 to 31.2 s) | +71% (12.2 to 20.9 s) |
| EP-D | +45% (334.3 to 484.5 s) | +47% (169.7 to 249.0 s) | +27% (99.1 to 125.7 s) |
| LU-C | +63% (147.8 to 241.2 s) | +63% (83.1 to 135.1 s) | +53% (47.6 to 72.7 s) |

Each overhead figure is the difference between two separate runs (a noFT run and a
MANA-noFT run) executed at different points in time on EC2 spot instances. With 3
repetitions per cell, within-group variability is now measurable. The noFT standard
deviations are small (under 4 s), confirming stable baselines. The MANA-noFT runs
show somewhat more variability (up to ±15.6 s for EP-D at 2w), consistent with
MANA's checkpoint infrastructure adding non-deterministic synchronization at startup
and shutdown.

The overhead values have no consistent pattern across benchmarks or worker counts, and
are much larger than the ~4-5 s overhead measured by the synthetic studies (Section 4).
This gap between synthetic and real-benchmark overhead is examined in Section 2.4.

### 2.1 CG-C at 2 Workers: Anomalous Overhead

CG at 2 workers shows +222% overhead (34.1 s to 109.8 s), while at 4 workers it shows
only +55% (20.2 s to 31.2 s). The 3-repetition means are stable (std ≤ 4.6 s),
confirming this is reproducible.

The most plausible mechanism is a CPU competition cascade triggered by MANA's spin-loop
wait behavior. The key difference between noFT and MANA runs is in how each handles
`MPI_Wait`: in noFT, `MPI_Wait` is a native blocking call that yields the CPU to the OS
scheduler; in MANA, `MPI_Wait` is implemented as a tight spin loop that continuously
polls for the message while holding a checkpoint lock (Section 4.2), actively burning
CPU cycles throughout the wait.

At 2 workers, two MPI processes run on the same physical machine. When one process spins
waiting for a message from its co-located partner, it consumes CPU that the partner needs
to finish its computation. This slows the partner, which delays the message, which
extends the spin time on the waiting process, which burns more CPU, compounding over
CG's 75 conjugate-gradient iterations. In the noFT case, the waiting process yields its
CPU to the OS, so the computing partner is not slowed down, and no cascade forms.

**Why does MANA use a spin loop instead of a blocking call?** A native blocking
`MPI_Wait` (or the `select()`/`poll()` system call it relies on) puts the process to
sleep inside the OS kernel. While asleep, the process holds DMTCP's checkpoint read-lock,
preventing any checkpoint signal from being processed. If MANA allowed this, a process
blocked waiting for a slow message would hold the lock indefinitely, and the checkpoint
mechanism could never proceed. The spin loop is the design trade-off that solves this:
between each polling iteration, MANA calls `DMTCP_PLUGIN_ENABLE_CKPT()` to release the
lock before immediately re-acquiring it with `DMTCP_PLUGIN_DISABLE_CKPT()`. During that
brief window between iterations, the checkpoint handler can preempt the process and save
its state cleanly. The cost is that the process remains in user space continuously burning
CPU rather than yielding to the OS scheduler. This is a fundamental requirement of
DMTCP-based checkpointing: safe checkpoints require user-space control over exactly when
they can fire.

The `synth_imbalanced` test (Section 4.2) was designed to measure whether this spin-loop
CPU burn translates to observable overhead when a communication partner is delayed. That
test places sender and receiver on separate nodes, so cross-process CPU competition is
impossible by design. The cascade at CG 2w was only identified after comparing the
synthetic results with the real benchmark data: the synthetic test confirmed that spin-loop
overhead alone (with separated nodes) does not exceed ~4-5 s, making the 75 s surplus at
2w inexplicable without the co-location factor.

At 4 and 8 workers, CG's processes are spread across more machines, reducing the chance
of co-located processes competing for CPU during waits.

This cascade hypothesis cannot be directly confirmed without per-rank profiling inside
the NAS benchmark, which is outside this study's scope.

### 2.2 EP-D: Overhead Decreases with More Workers

With N=3 repetitions per cell, EP-D overhead is consistent at 27-47%, with small
standard deviations (±0.7-15.6 s on wall times of 100-485 s). The absolute overhead
decreases from 150 s at 2w to 79 s at 4w to 27 s at 8w, a 5.5x reduction as worker
count increases 4x.

Importantly, this trend is the **opposite** of what a fixed startup-cost explanation
would predict. A fixed overhead of, say, 30 s would represent only 9% of EP's 334 s noFT
run at 2w, but 30% of its 99 s noFT run at 8w. Shorter jobs would show higher overhead
percentages from any fixed cost. What we observe is the reverse: overhead is highest
(45-47%) for the longest runs and lowest (27%) for the shortest.

No mechanistic explanation for this trend has been identified from the current data.
The observation is reproducible (N=3 per cell), confirming it is a real characteristic
of EP under MANA rather than measurement noise.

### 2.3 LU-C: Most Consistent Overhead

LU-C shows 63% at 2w, 63% at 4w, 53% at 8w, the most stable pattern of the three
benchmarks. The overhead stays roughly proportional to computation time across worker
counts. The slight decrease at 8w (53% vs 63%) has no mechanistic explanation identified
from the current data, the same situation as §2.2. The observation is reproducible (N=3
per cell, noFT std ≤ 3.7 s), but the data alone does not point to a cause.

LU is the most reliable benchmark for overhead characterization: structured communication,
stable runtime, and low within-cell variability.

### 2.4 The Overhead Gap: What the Synthetic Tests Do and Do Not Tell Us

The synthetic studies (Section 4) consistently measure ~4-5 s of MANA overhead for
isolated programs, regardless of call frequency or communication imbalance. The real NPB
benchmarks show 27-222%. This large gap is an important unresolved finding.

The synthetic tests establish two things: (1) the overhead is not driven by MPI call
frequency, and (2) it is not driven by simple communication imbalance between two isolated
processes on separate nodes. What they cannot test is any mechanism that emerges from the
combination of computation, communication, and resource sharing that occurs in real
benchmark runs.

The CG at 2w case has a plausible mechanism (spin-loop CPU competition between co-located
processes, Section 2.1). For the general overhead level seen across all benchmarks
(27-222%), no clean explanation is available from this study. Proposed mechanisms were
considered but each raises contradictions with the observed data:

- **Initialization and teardown scaling with rank count.** If MANA's setup cost grew with
  the number of MPI ranks, overhead would increase at higher worker counts (more ranks).
  The observed trend for EP and LU is the opposite.
- **Communication and computation interleaving.** The hypothesis that MANA's overhead
  depends on how frequently computation phases are interrupted by MPI calls is precisely
  what the call-frequency synthetic test measures. The test showed no such effect.

The honest conclusion is that the synthetic studies narrow the search space for overhead
mechanisms but do not identify the root cause in real benchmarks. Isolating the
individual contributors would require profiling MANA's internal state (lock contention
patterns, coordinator activity, per-phase timing) during actual benchmark runs. This is
left as future work.

---

## 3. Strong Scaling

![Strong scaling: wall time vs worker count](plots/fig6_mana_scalability.png)

**Table 2: Wall time (seconds) vs worker count, noFT and MANA-noFT (mean ± std, N=3):**

| Benchmark | noFT 2w | noFT 4w | noFT 8w | Speedup 2→8 | MANA 2w | MANA 4w | MANA 8w |
|---|---|---|---|---|---|---|---|
| CG-C | 34.1±0.7 | 20.2±1.3 | 12.2±0.1 | 2.8× | 109.8±4.6 | 31.2±1.3 | 20.9±0.0 |
| EP-D | 334.3±0.7 | 169.7±0.9 | 99.1±3.2 | 3.4× | 484.5±15.6 | 249.0±2.0 | 125.7±0.9 |
| LU-C | 147.8±1.8 | 83.1±3.7 | 47.6±0.3 | 3.1× | 241.2±1.2 | 135.1±1.7 | 72.7±1.3 |

All three benchmarks scale sub-linearly from 2 to 8 workers (ideal would be 4×). EP and LU
reach 3.1-3.4×, reasonable for memory-bound MPI applications at small cluster sizes.
CG's 2.8× reflects its irregular communication pattern.

The key result: **MANA-noFT follows the same scaling curve as noFT**. The overhead does
not worsen at higher worker counts, meaning MANA's checkpoint infrastructure does not
introduce a scaling bottleneck.

---

## 4. MANA Overhead Mechanisms: Synthetic Studies

The real benchmarks mix computation, communication, and cluster effects in ways that make
it hard to isolate what drives MANA overhead. Four synthetic programs were designed to
test one variable at a time.

### 4.1 Call Frequency Study

**Why this test was designed:** CG and LU make thousands of MPI calls per run. Before
investigating any other mechanism, the simplest possible explanation had to be ruled out:
that MANA's per-call wrapper overhead simply accumulates with call count, producing higher
overhead for benchmarks that communicate more frequently. `synth_calls` tests this
directly by running a fixed workload while varying the number of MPI calls from 0 to
51,200.

**What this tests:** Whether MANA overhead grows as the program calls MPI operations more
frequently. `synth_calls` does a fixed amount of computation and calls `MPI_Allreduce`
(EP's primary operation) at different frequencies, from 0 to 51,200 calls per run.
`synth_p2p` does the same but uses `MPI_Send + MPI_Recv` pairs (LU's pattern):

```c
/* synth_calls: collective operations */
for (long outer = 0; outer < TOTAL_OUTER; outer++) {
    for (long i = 0; i < INNER_ITERS; i++)
        x = x * 1.0000001 + 1e-10;
    if (call_period > 0 && outer % call_period == 0) {
        MPI_Allreduce(&x, &result, 1, MPI_DOUBLE, MPI_SUM, MPI_COMM_WORLD);
        x += result * 1e-20;
    }
}

/* synth_p2p: point-to-point operations (deadlock-safe) */
if (rank < nprocs / 2) {
    MPI_Send(&x, 1, MPI_DOUBLE, partner, 0, MPI_COMM_WORLD);
    MPI_Recv(&result, 1, MPI_DOUBLE, partner, 0, MPI_COMM_WORLD, &status);
} else {
    MPI_Recv(&result, 1, MPI_DOUBLE, partner, 0, MPI_COMM_WORLD, &status);
    MPI_Send(&x, 1, MPI_DOUBLE, partner, 0, MPI_COMM_WORLD);
}
```

![MANA overhead vs MPI call frequency](plots/fig2_synth_calls.png)

**Table 3: MANA overhead (added seconds) vs call count (mean ± std, N=3):**

| Level | Calls | synth_calls overhead | synth_p2p overhead |
|---|---|---|---|
| L0 | 0 | +3.5 s | +4.4 s |
| L1 | 800 | +4.8 s | +4.3 s |
| L2 | 3,200 | +4.5 s | +5.9 s |
| L3 | 12,800 | +4.7 s | +3.8 s |
| L4 | 51,200 | +4.3 s | +2.2 s |

**Finding:** MANA overhead is flat (~3.5-5 s) regardless of call count. Call frequency is
not the driver. The overhead comes from DMTCP infrastructure startup and teardown costs
at program initialization and exit. The `synth_p2p` overhead is also flat; the slight
decline at L4 occurs because both partners reach the exchange at the same moment
(perfectly synchronized), so MANA's sleep-and-retry path never fires.

### 4.2 Communication Imbalance Study

**Why this test was designed:** CG and LU use `MPI_Irecv + MPI_Wait` for their
communication exchanges. As described in Section 2.1, MANA implements `MPI_Wait` as a
tight spin loop that burns CPU while waiting for a message. The question this test asks
is: if one side of the exchange arrives late (sender delay), does MANA's spin loop amplify
the overhead beyond what the delay itself costs? Isolating this effect required a
synthetic program where the sender delay is controlled precisely and the spin loop is the
only variable.

**What this tests:** CG and LU use `MPI_Irecv + MPI_Wait` rather than blocking
`MPI_Recv`. MANA's `MPI_Wait` is a tight spin loop:

```c
/* MANA's internal MPI_Wait wrapper (simplified) */
while (!flag) {
    DMTCP_PLUGIN_DISABLE_CKPT();   /* acquire checkpoint read-write lock */
    MPI_Test_internal(..., &flag); /* call real MPI_Test, bypassing MANA wrappers */
    DMTCP_PLUGIN_ENABLE_CKPT();    /* release checkpoint lock */
}
```

The lock acquire/release pair runs at thousands of iterations per second for however long
the message is delayed, burning CPU while the receiver waits.

`synth_imbalanced` forces the receiver into this spin loop using a controlled sender
delay. Sender and receiver are on separate nodes by design:

```c
if (is_sender) {
    if (DELAY_US > 0) usleep(DELAY_US);
    MPI_Send(&x, 1, MPI_DOUBLE, partner, 0, MPI_COMM_WORLD);
    MPI_Irecv(&recv_buf, 1, MPI_DOUBLE, partner, 1, MPI_COMM_WORLD, &request);
    MPI_Wait(&request, &status);  /* reply already sent, completes immediately */
} else {
    MPI_Irecv(&recv_buf, 1, MPI_DOUBLE, partner, 0, MPI_COMM_WORLD, &request);
    MPI_Wait(&request, &status);  /* spins for DELAY_US (this is the test) */
    MPI_Send(&x, 1, MPI_DOUBLE, partner, 1, MPI_COMM_WORLD);
}
```

![MANA overhead vs sender delay (communication imbalance)](plots/fig2b_synth_imbalanced.png)

**Table 4: synth_imbalanced results (mean ± std, N=3):**

| Level | Sender delay | noFT elapsed | MANA elapsed | Overhead |
|---|---|---|---|---|
| L0 | 0 µs | 63.5±0.6 s | 67.6±0.5 s | +4.1 s |
| L1 | 100 µs | 63.2±0.3 s | 67.6±0.4 s | +4.5 s |
| L2 | 1 ms | 63.4±0.8 s | 68.1±0.6 s | +4.7 s |
| L3 | 5 ms | 67.6±0.1 s | 72.0±0.2 s | +4.4 s |
| L4 | 20 ms | 78.7±0.3 s | 83.6±1.2 s | +4.9 s |

**Finding:** MANA overhead is constant at ~4-5 s regardless of sender delay. The spin
loop does burn CPU during the wait, but this does not add to the overhead *beyond* what
the wait itself already costs in noFT. The noFT baseline grows at L3/L4 (the program is
genuinely waiting for the slow sender), but the MANA-noFT grows by the same amount.

This study isolates sender and receiver on separate nodes, so the spin loop cannot
compete with the sender's computation for CPU resources. This is the key limitation
discussed in Section 2.1: the cascade at CG 2w requires co-located processes, a
scenario the synthetic study cannot reproduce.

### 4.3 Checkpoint Image Size Study

**What this tests:** How long does Phase 1 (failure detected to checkpoint written to EFS)
take as the process memory footprint grows? `synth_checkpoint_size` allocates a known
amount of memory per process, forces the OS to map all pages with `memset`, and keeps the
buffer active throughout the run:

```c
long n = (MEM_MB * 1024L * 1024L) / sizeof(double);
double *buf = (double *)malloc(n * sizeof(double));
memset(buf, 0x42, n * sizeof(double));   /* force OS to map all pages */

double t_start = MPI_Wtime();
volatile long counter = 0;
while (MPI_Wtime() - t_start < TARGET_SECS) {
    buf[counter % n] += 1.0;   /* keep buffer live in checkpoint */
    counter++;
    if (counter % 5000000L == 0) MPI_Barrier(MPI_COMM_WORLD);
}
/* MANA injects the checkpoint signal at trigger_after_secs=30 */
```

![Phase 1 time vs checkpoint image size](plots/fig3_synth_ckpt.png)

**Table 5: Phase 1 time vs memory per process (mean ± std, N=3):**

| Memory/process | Phase 1 | Notes |
|---|---|---|
| 50 MB | 16.4 ± 0.1 s | small image |
| 200 MB | 26.9 ± 0.4 s | |
| 800 MB | 47.0 ± 0.2 s | |
| 3,200 MB | 139.2 ± 0.4 s | approaches EFS burst ceiling |

**Finding 1:** Phase 1 time scales **linearly** with memory per process. Standard
deviations are very small (≤ 0.4 s), confirming that EFS write bandwidth is stable across
repetitions. Phase 1 is predictable from a job's memory requirements.

**Finding 2:** The 3,200 MB data point approaches the EFS bandwidth ceiling.

**What is EFS bandwidth?** Amazon EFS (Elastic File System) is the shared network
filesystem mounted on all worker instances and used here as the checkpoint store. Every
MPI process writes its full memory image to EFS when Phase 1 begins. EFS Bursting
Throughput (the default mode) provides a maximum burst bandwidth of **100 MiB/s
(~105 MB/s)** for file systems smaller than 1 TiB, plus burst credits that accumulate
over time. This ceiling is fixed by AWS and applies to the aggregate writes across all
clients. It can be raised only by switching to Provisioned Throughput (a paid upgrade) or
by using a high-performance parallel filesystem instead of EFS.

In our experiment, each MPI process writes independently to EFS in Phase 1. With 4
processes writing simultaneously (2 workers × 2 processes each at the 2-worker
configuration), the aggregate demand at 3,200 MB per process is approximately
3,200 / 139.2 × 4 ≈ **92 MB/s**, reaching ~88% of the 105 MB/s burst ceiling. At
larger memory footprints or more simultaneous writers, the ceiling would be exceeded
and Phase 1 times would grow faster than linearly.

**Implication for fault tolerance:** If per-process memory exceeds approximately 3-4 GB,
Phase 1 would take longer than the 2-minute EC2 spot instance interruption warning,
meaning the checkpoint would not finish before the instance is terminated. MANA-based
fault tolerance on EFS is therefore not suitable for memory-intensive jobs unless
Provisioned Throughput is enabled or a faster shared filesystem is used.

**Connection to real benchmarks:** EP-D processes allocate less memory than LU-C or
CG-C, matching the Phase 1 times in FT runs: ~16-20 s for EP vs ~26-37 s for LU and CG.
The checkpoint size study confirms the difference is memory footprint, not benchmark
complexity.

---

## 5. Recovery Phase Analysis: CG Short-Job Case

![CG short-job FT wall time breakdown](plots/fig5_cg_short_job.png)

CG-C is excluded from the failure timing sensitivity study (Section 6) because its
10-35 s noFT runtime makes timing variants meaningless. It is, however, the clearest
illustration of the key structural fact: **for short jobs, recovery overhead dominates
total wall time**, and all the phase mechanics are visible in their simplest form.

**Table 6: CG-C FT wall time breakdown (base trigger ~3 s, mean ± std, N=3):**

| Workers | Strategy | P0 | P1 | Slurm (P2a+P2c) | P2b | P3 | Total |
|---|---|---|---|---|---|---|---|
| 2w | REPLACE | 3.0 s | 26.6±0.0 s | 10.9 s | 120.7±2.6 s | 40.2±3.0 s | **206.4±4.5 s** |
| 2w | DEGRADED | 3.0 s | 26.5±0.1 s | 10.8 s | 10.9±0.0 s | 113.1±3.1 s | **168.6±2.7 s** |
| 4w | REPLACE | 3.0 s | 26.7±0.3 s | 10.8 s | 123.3±1.9 s | 26.4±0.0 s | **196.2±5.0 s** |
| 4w | DEGRADED | 3.0 s | 26.5±0.0 s | 10.8 s | 10.9±0.0 s | 35.0±6.1 s | **92.6±7.6 s** |
| 8w | REPLACE | 3.0 s | 36.7±0.1 s | 10.8 s | 119.7±2.1 s | 24.6±6.0 s | **203.2±4.8 s** |
| 8w | DEGRADED | 3.0 s | 36.9±0.2 s | 11.1 s | 11.3±0.4 s | 24.8±2.9 s | **96.7±3.0 s** |

**Reading the phases:**

- **P0 (pre-failure):** 3 s across all rows; the failure is triggered very early and is
  identical for both strategies.
- **P1 (checkpoint write):** ~26.5 s at 2w and 4w, rises to ~36.8 s at 8w, and is
  identical for both strategies at the same cluster size. The increase at 8w is specific
  to CG: its irregular sparse communication means each process maintains per-rank MPI
  buffers and communication state for all its communication partners. As worker count
  grows, each process has more partners, increasing its per-process checkpoint size.
  EP and LU do not show this effect because EP uses only collective operations (no
  per-rank state) and LU has fixed nearest-neighbor communication (constant per-rank
  state regardless of cluster size).
- **Slurm and MANA overhead (P2a+P2c):** ~10.8-11.1 s, constant across all configurations
  and both strategies.
- **P2b (node reconfig):** REPLACE pays ~120-123 s for EC2 respawn; DEGRADED pays ~11 s
  for `scontrol DOWN`. The ~109-112 s gap is the direct cost of the REPLACE strategy.
- **P3 (remaining computation):** Larger for DEGRADED because it continues with one fewer
  worker. At 2w this is significant (113.1 s vs 40.2 s); at 4w and 8w the extra P3 cost
  is much smaller as the capacity fraction lost is smaller.

For very short jobs like CG, the FT overhead is 3-17× the original job duration.
MANA-based fault tolerance is most cost-effective for long-running jobs where recovery
phases represent a small fraction of total wall time.

---

## 6. Failure Timing Analysis: Phase-by-Phase Study

![Full FT wall time by failure timing](plots/fig4_timing_phases.png)

This is the central analysis of Phase 5. EP-D and LU-C were each run with failure
injected at 10%, 25%, and 50% of the MANA-noFT wall time, at 2, 4, and 8 workers.

### 6.1 Phase 0: Pre-Failure Computation

Phase 0 grows as the failure trigger moves from 10% to 50%. P0 is **identical for
REPLACE and DEGRADED** within each timing group, since the failure is triggered by the
same `auto_failure_trigger_secs` setting. This is visible in Figure 4 as the equal-height
base on each bar pair.

### 6.2 Phase 1: Checkpoint Write

P1 is constant within each benchmark × worker-count combination, and equal across
strategies. It depends only on the per-process memory footprint at the time of failure:

| Configuration | P1 |
|---|---|
| EP-D, any worker count | ~16-20 s |
| LU-C, any worker count | ~26.6 s |
| CG-C, 2w and 4w | ~26.5-26.7 s |
| CG-C, 8w | ~36.8 s |

EP and LU show constant P1 regardless of cluster size: EP uses only collective operations
with no per-rank MPI state, and LU has fixed nearest-neighbor communication. CG at 8w is
the exception, as explained in Section 5: more communication partners mean a larger
per-process checkpoint image. This is consistent with the linear relationship confirmed
by the synthetic checkpoint study.

### 6.3 Phase 2a and Phase 2c: Slurm and MANA Coordination

These two sub-phases together take ~10.8-11.0 s for all configurations and both
strategies. This is the overhead of communicating with the cluster control plane to
reconfigure Slurm for the restart. The cost is the same whether the node is being
replaced or dropped.

### 6.4 Phase 2b: Node Reconfiguration (The Differentiator)

P2b is where REPLACE and DEGRADED fundamentally diverge:

- **REPLACE:** waits for EC2 to terminate the spot instance, provision a new one, boot
  it, install Slurm, and bring it to IDLE state. This takes ~111-132 s and is driven by
  AWS EC2 provisioning latency, independent of cluster size or failure timing.
- **DEGRADED:** only runs `scontrol update NodeName=... State=DOWN` to mark the failed
  node as unavailable. This takes ~11 s.

The P2b gap (~100-121 s) is **the single most important factor** in this study.

**Table 7: P2b times across configurations (mean ± std, pooled across fault timings):**

| Benchmark | Workers | P2b REPLACE | P2b DEGRADED | Gap |
|---|---|---|---|---|
| EP-D | 2w | 120±8 s | 11±0 s | ~109 s |
| EP-D | 4w | 118±9 s | 11±0 s | ~107 s |
| EP-D | 8w | 128±7 s | 11±0 s | ~117 s |
| LU-C | 2w | 122±4 s | 11±0 s | ~111 s |
| LU-C | 4w | 123±6 s | 11±0 s | ~112 s |
| LU-C | 8w | 126±10 s | 11±0 s | ~115 s |

### 6.5 Phase 3: Remaining Computation

P3 is determined by how much work remains at the time of failure (shrinks as fault
trigger moves from 10% to 50%) and by capacity after recovery (REPLACE restarts at N
workers; DEGRADED at N-1 workers).

The P3 gap between strategies shrinks as cluster size grows:
- At 2 workers, losing 1 of 2 means DEGRADED runs P3 with ~50% fewer resources.
- At 4 workers, losing 1 of 4 means ~25% more P3 time.
- At 8 workers, losing 1 of 8 means ~12% more P3 time.

This structural property means DEGRADED's advantage tends to strengthen at larger cluster
sizes: the P2b gap stays roughly constant (~107-117 s), while DEGRADED's P3 penalty
shrinks as the fraction of lost capacity decreases.

### 6.6 Bar Height Patterns: Why Totals Are Approximately Constant

Two patterns stand out in Figure 4 across both benchmarks:

**DEGRADED bars are roughly the same height regardless of fault timing.** As the trigger
moves from 10% to 50%, P0 grows (more pre-failure productive work) while P3 shrinks
(less remaining work). These two effects partially cancel. The reason the cancellation is
not perfect is that DEGRADED's P3 runs at reduced capacity (N-1 workers), so P3 decreases
more slowly than P0 increases in absolute terms. In other words: even though there is
less work remaining after a later fault, each unit of that remaining work takes longer
because the cluster is weaker. These two competing effects (less work but slower
processing) result in a roughly flat total wall time across fault timings.

A natural question arises: if the fault occurs later, the cluster has less time running
at reduced capacity, so shouldn't the bar get shorter? The answer is that the cluster
runs at full capacity only during P0; *all* of the remaining computation (P3) runs at
reduced capacity regardless of when the fault occurs. A later fault means less P3 work,
but that work is still done at the same reduced speed. The P3 savings from less work are
roughly offset by the P3 overhead from lost capacity, producing flat totals.

**REPLACE bars also tend to be roughly constant but show more variation.** For REPLACE,
P0 and P3 both run at full N-worker capacity, so in principle P0+P3 should equal the
full MANA-noFT job time and total wall time should be constant at MANA_noft + fixed
recovery. In practice, the data shows that P0+P3 is not constant: it increases with
fault timing.

**Table: EP-D and LU-C REPLACE: P3 compared to expected (MANA-noFT minus P0):**

| Benchmark | Workers | Fault | P0 | Actual P3 | Expected P3 | Gap | Gap% |
|---|---|---|---|---|---|---|---|
| EP-D | 4w | 10% | 32 s | 152 s | 217 s | 65 s | **30%** |
| EP-D | 4w | 25% | 81 s | 122 s | 168 s | 46 s | **27%** |
| EP-D | 4w | 50% | 151 s | 70 s | 98 s | 28 s | **29%** |
| EP-D | 8w | 10% | 12 s | 87 s | 114 s | 27 s | **23%** |
| EP-D | 8w | 25% | 31 s | 75 s | 95 s | 20 s | **21%** |
| EP-D | 8w | 50% | 61 s | 51 s | 65 s | 14 s | **22%** |
| LU-C | 4w | 10% | 13 s | 81 s | 122 s | 41 s | **34%** |
| LU-C | 4w | 25% | 33 s | 68 s | 102 s | 34 s | **33%** |
| LU-C | 4w | 50% | 66 s | 47 s | 69 s | 22 s | **32%** |
| LU-C | 8w | 10% | 7 s | 53 s | 66 s | 13 s | **20%** |
| LU-C | 8w | 25% | 17 s | 47 s | 56 s | 8 s | **15%** |
| LU-C | 8w | 50% | 34 s | 37 s | 39 s | 2 s | **5%** |

**Expected P3** = MANA-noFT minus P0 (what P3 would be if the restarted job ran at the
same speed as the fault-free MANA baseline). **Gap** = Expected P3 minus Actual P3 (how
many seconds faster P3 actually ran). **Gap%** = Gap / Expected P3.

P0+P3 is always less than MANA-noFT, and the gap in absolute seconds shrinks as fault
timing increases because there is less remaining work (smaller Expected P3). The gap as a
**percentage of Expected P3 is roughly constant**: ~22-30% for EP and ~32-34% for LU at
4w. This is a key observation: a constant percentage means P3 consistently runs ~25-33%
faster than expected from a cold start, regardless of how much work remains. This behavior
is consistent with a warm-restart effect, where the restarted P3 benefits from OS
page-table entries and memory pages that were already populated during P0. The benefit
scales with the amount of remaining work (constant fraction), not as a fixed time bonus.

The one exception is LU at 8w: at 50% fault, only 37 s of P3 remains, and the gap
collapses to 2 s (5%). This is consistent with the warm-restart hypothesis: a very short
P3 accesses only a small portion of the working set, leaving little room for cache warmth
from P0 to provide a benefit.

The practical consequence is that REPLACE bar heights grow slightly with fault timing
(because P0 grows while P3 shrinks less than expected), and are additionally affected by
P2b EC2 provisioning noise (std ~7-15 s per configuration).

### 6.7 EP-D Full Phase Breakdown

**Table 8: EP-D mean phase times by configuration (seconds, N=3 per cell):**

| Workers | Trigger | Strategy | P0 | P1 | P2a+2c | P2b | P3 | Total |
|---|---|---|---|---|---|---|---|---|
| 2w | 10% (~46s) | REPLACE | 46 | 16 | 11 | 116 | 322 | **516±4** |
| 2w | 10% (~46s) | DEGRADED | 46 | 16 | 11 | 11 | 446 | **535±3** |
| 2w | 25% (~116s) | REPLACE | 116 | 20 | 11 | 122 | 270 | **543±14** |
| 2w | 25% (~116s) | DEGRADED | 116 | 16 | 11 | 11 | 373 | **532±4** |
| 2w | 50% (~231s) | REPLACE | 231 | 16 | 11 | 122 | 188 | **573±13** |
| 2w | 50% (~231s) | DEGRADED | 231 | 20 | 11 | 11 | 262 | **541±6** |
| 4w | 10% (~32s) | REPLACE | 32 | 16 | 11 | 111 | 152 | **330±14** |
| 4w | 10% (~32s) | DEGRADED | 32 | 16 | 11 | 11 | 218 | **294±4** |
| 4w | 25% (~81s) | REPLACE | 81 | 16 | 11 | 121 | 122 | **358±1** |
| 4w | 25% (~81s) | DEGRADED | 81 | 20 | 11 | 11 | 170 | **302±3** |
| 4w | 50% (~151s) | REPLACE | 151 | 16 | 11 | 123 | 70 | **378±3** |
| 4w | 50% (~151s) | DEGRADED | 151 | 17 | 11 | 11 | 100 | **299±2** |
| 8w | 10% (~12s) | REPLACE | 12 | 16 | 11 | 132 | 87 | **269±7** |
| 8w | 10% (~12s) | DEGRADED | 12 | 20 | 11 | 11 | 117 | **179±8** |
| 8w | 25% (~31s) | REPLACE | 31 | 16 | 11 | 126 | 75 | **270±15** |
| 8w | 25% (~31s) | DEGRADED | 31 | 16 | 11 | 11 | 105 | **182±1** |
| 8w | 50% (~61s) | REPLACE | 61 | 20 | 12 | 127 | 51 | **282±15** |
| 8w | 50% (~61s) | DEGRADED | 61 | 16 | 11 | 11 | 75 | **184±13** |

At 2w/10%, DEGRADED's P3 penalty (446-322 = 124 s) exceeds the P2b gap (116-11 = 105 s),
so REPLACE is faster by ~19 s. At every other EP configuration, the P2b gap exceeds the
P3 penalty and DEGRADED is faster. As cluster size grows the margin widens: at 8w the
gap between strategies is consistently ~90 s across all fault timings.

### 6.8 LU-C Full Phase Breakdown

**Table 9: LU-C mean phase times by configuration (seconds, N=3 per cell):**

| Workers | Trigger | Strategy | P0 | P1 | P2a+2c | P2b | P3 | Total |
|---|---|---|---|---|---|---|---|---|
| 2w | 10% (~24s) | REPLACE | 24 | 27 | 11 | 121 | 141 | **329±7** |
| 2w | 10% (~24s) | DEGRADED | 24 | 27 | 11 | 11 | 228 | **304±3** |
| 2w | 25% (~59s) | REPLACE | 59 | 27 | 11 | 123 | 117 | **341±5** |
| 2w | 25% (~59s) | DEGRADED | 59 | 27 | 11 | 11 | 192 | **303±3** |
| 2w | 50% (~118s) | REPLACE | 118 | 27 | 11 | 123 | 85 | **369±7** |
| 2w | 50% (~118s) | DEGRADED | 118 | 27 | 11 | 11 | 131 | **302±1** |
| 4w | 10% (~13s) | REPLACE | 13 | 27 | 11 | 129 | 81 | **266±5** |
| 4w | 10% (~13s) | DEGRADED | 13 | 27 | 11 | 11 | 125 | **193±1** |
| 4w | 25% (~33s) | REPLACE | 33 | 27 | 11 | 121 | 68 | **267±6** |
| 4w | 25% (~33s) | DEGRADED | 33 | 27 | 11 | 11 | 105 | **194±3** |
| 4w | 50% (~66s) | REPLACE | 66 | 26 | 11 | 119 | 47 | **275±0** |
| 4w | 50% (~66s) | DEGRADED | 66 | 27 | 11 | 11 | 72 | **193±2** |
| 8w | 10% (~7s) | REPLACE | 7 | 26 | 11 | 122 | 53 | **231±4** |
| 8w | 10% (~7s) | DEGRADED | 7 | 27 | 11 | 11 | 69 | **136±1** |
| 8w | 25% (~17s) | REPLACE | 17 | 27 | 11 | 137 | 47 | **247±9** |
| 8w | 25% (~17s) | DEGRADED | 17 | 27 | 11 | 11 | 61 | **136±1** |
| 8w | 50% (~34s) | REPLACE | 34 | 27 | 11 | 120 | 37 | **236±3** |
| 8w | 50% (~34s) | DEGRADED | 34 | 27 | 11 | 11 | 47 | **141±5** |

LU shows no crossover at any configuration: DEGRADED is faster across all 9 tested cases.
The advantage grows with cluster size: at 8w DEGRADED wins by ~90-95 s vs ~25 s at 2w.
LU's structured stencil communication means the P3 penalty for losing one worker is
smaller than for EP, since the remaining workers have a more favorable communication
pattern with fewer neighbors.

---

## 7. Strategy Comparison: Tendencies and Trade-offs

![REPLACE vs DEGRADED total wall time with MANA-noFT baseline](plots/fig8_strategy_comparison.png)

### 7.1 Scope of This Study

All runs were designed to complete their checkpoint before the spot instance's 2-minute
interruption warning expired, meaning only short-to-medium jobs were evaluated. Very long
workloads (hours-long HPC jobs) were not tested. The tendency analysis below uses
structural reasoning to project what would happen for longer jobs.

### 7.2 Fixed vs Variable Costs of Recovery

Most of the recovery cost is **fixed** and does not depend on how long the job runs or
when the failure occurs:

| Phase | Cost | Varies with |
|---|---|---|
| P1 (checkpoint write) | 16-37 s | memory footprint only |
| P2a + P2c (Slurm/MANA coordination) | ~11 s | nothing |
| P2b REPLACE (EC2 provisioning) | ~111-132 s | AWS latency only |
| P2b DEGRADED (scontrol DOWN) | ~11 s | nothing |
| P3 (remaining computation) | varies | failure timing and capacity loss |

The only phase that varies with job length is P3. For both strategies, P3 shrinks as
the failure occurs later in the job.

### 7.3 When Each Strategy Tends to Win

The results show a clear structural condition for DEGRADED to be faster than REPLACE.
DEGRADED saves ~109-117 s in P2b, but pays a penalty in P3 by running with N-1 workers.
For a perfectly parallel workload, this P3 penalty is:

```
P3 penalty = remaining_work / (N-1)  -  remaining_work / N
           = remaining_work / [N × (N-1)]
```

DEGRADED wins when the P2b savings exceed this penalty:

```
P2b_gap (≈109 s)  >  remaining_work / [N × (N-1)]
→  remaining_work  <  109 × N × (N-1)
```

For each cluster size, the crossover point in remaining work is:

| Workers (N) | Crossover (remaining work) | Example: DEGRADED wins if fault at... |
|---|---|---|
| N = 2 | 109 × 1 = **109 s** (~1.8 min) | > 96% through the job (most scenarios: REPLACE wins) |
| N = 4 | 109 × 3 = **327 s** (~5.5 min) | > 87% through the job |
| N = 8 | 109 × 7 = **763 s** (~12.7 min) | > 94% through the job |

All jobs tested in this study had remaining P3 work well below these thresholds
(maximum ~320 s for EP-D at 2w with 10% fault timing), which is why DEGRADED wins in
most tested cases. For hour-long or multi-hour jobs, remaining work would far exceed
these thresholds at any realistic fault timing, and REPLACE would tend to be faster.

This analysis has an important practical implication: **the advantage of DEGRADED is not
about job length per se, but about how much work remains when the fault occurs.** For the
short jobs evaluated here, remaining work is small enough that DEGRADED's P3 penalty
never accumulates to exceed the P2b savings. For long jobs, even a late fault (say 90%
through a 4-hour job) leaves 24 minutes of remaining work, far above the 8w crossover
of 12.7 minutes. REPLACE would win there too.

### 7.4 Multiple Failures

Only single-failure scenarios were tested. For workloads that may experience multiple
successive spot interruptions:

- **REPLACE** always returns to the original cluster size. Each failure costs the same
  fixed P2b (~111-132 s). Cluster capacity never degrades.
- **DEGRADED** continues with fewer workers after each failure. For clusters starting at
  4w or 8w, one or two failures are tolerable. For 2w clusters, a second failure would
  leave only one worker, which may be unable to continue depending on the MPI decomposition.

The tendency is clear: REPLACE is more robust under repeated failures.

### 7.5 Summary: When to Use Each Strategy

| Scenario | Strategy | Reason |
|---|---|---|
| Short job (< 1-2 min noFT) | Neither | Recovery overhead dominates; spot savings do not cover FT cost |
| Medium job (100-500 s) with 4+ workers | **DEGRADED** | Remaining work stays below the crossover threshold; P2b gap is decisive |
| Medium job (100-500 s) with 2 workers | **REPLACE** (usually) | At 2w, DEGRADED only wins when less than ~109 s remain; only the 25%/50% fault cases qualify |
| Long job (> ~30 min) at any worker count | **REPLACE** | Remaining work at realistic fault timings exceeds the crossover threshold for all cluster sizes |
| Multiple failures expected | **REPLACE** | Capacity never degrades; each failure costs the same fixed P2b |

---

## 8. Recovery Overhead Ratio

![Recovery overhead as % of total FT wall time](plots/fig9_recovery_overhead_ratio.png)

The recovery overhead ratio measures what fraction of total FT wall time was not
pre-failure productive computation:

`recovery_overhead_pct = (ft_wall_time − P0) / ft_wall_time × 100`

**Why exclude P0?** P0 is pre-failure computation: work that was successfully done and
that a checkpoint preserves. It is not overhead in any meaningful sense: it would have
been executed whether or not the job ever failed. Everything after P0 (the checkpoint
write, the recovery logistics, and the restarted computation in P3) is time the job would
not have spent in a fault-free run. The ratio measures what fraction of total wall time
that cost represents.

**Why does the ratio decrease as fault timing increases, even though bar heights are
roughly constant?**

The bars being roughly the same height means total wall time barely changes across fault
timings. But within that roughly constant total, P0 grows as the failure is triggered
later. A larger P0 means a larger fraction of the total was useful pre-failure work, so
the remaining fraction (everything after P0) is smaller. Concretely for DEGRADED at 4w:

- At 10% fault timing: total ≈ 294 s, P0 ≈ 32 s → ratio = (294-32)/294 = **89%**
- At 50% fault timing: total ≈ 299 s, P0 ≈ 151 s → ratio = (299-151)/299 = **49%**

The total barely changed (294 s vs 299 s), but P0 nearly quintupled. The ratio does not
decrease because recovery takes less time; it decreases because a larger share of the
constant total was productive pre-failure work.

**Table 10: Recovery overhead as % of total FT wall time (EP-D, mean ± std, N=3):**

| Workers | Strategy | 10% | 25% | 50% |
|---|---|---|---|---|
| 2w | REPLACE | 91.1±0.1% | 78.6±0.5% | 59.6±0.9% |
| 2w | DEGRADED | 91.4±0.0% | 78.2±0.2% | 57.3±0.4% |
| 4w | REPLACE | 90.3±0.4% | 77.4±0.1% | 60.1±0.3% |
| 4w | DEGRADED | 89.1±0.2% | 73.2±0.2% | 49.4±0.4% |
| 8w | REPLACE | 95.5±0.1% | 88.5±0.6% | 78.3±1.1% |
| 8w | DEGRADED | 93.3±0.3% | 83.0±0.1% | 66.7±2.2% |

**Key observations:**

1. **Early failure produces high overhead ratio for both strategies.** When failure fires
   at 10%, recovery accounts for ~90% of total wall time. P0 is small, so nearly all
   elapsed time is spent on recovery.

2. **Later failure produces lower overhead ratio.** At 50% trigger, DEGRADED at 4w has
   only 49% overhead, meaning more than half the total wall time is useful pre-failure
   computation. This is the strongest economic argument for fault tolerance on short jobs:
   a late fault makes recovery a minor fraction of total time.

3. **At 8w, REPLACE overhead stays above 78% even at 50%.** Because the job is short
   (noFT ~99 s at 8w), the fixed EC2 provisioning cost (~128 s) represents most of the
   total regardless of fault timing. This is the same short-job structural problem
   identified in the CG case.

4. **DEGRADED and REPLACE have nearly identical overhead at 10%** for all configurations.
   The strategies diverge at 25% and 50%, where REPLACE's large P2b contributes a growing
   fraction of remaining time.

---

## 9. Economic Analysis

Figure 7 shows cost per run broken down by fault timing for each benchmark, strategy,
and worker count. The cost model compares two scenarios:

- **noFT** must run on **on-demand** instances. A spot interruption loses all progress
  and requires a full restart from scratch.
- **REPLACE and DEGRADED** can use **spot instances** (~70% cheaper per hour) because
  MANA handles interruptions automatically and the job resumes from checkpoint.

![Cost per run: spot with FT vs on-demand without FT](plots/fig7_cost.png)

**Table 11: Estimated cost per run (USD), averaged across fault timings (mean ± std, N=3):**

| Benchmark | Workers | noFT on-demand | REPLACE spot | DEGRADED spot | REPLACE vs noFT | DEGRADED vs noFT |
|---|---|---|---|---|---|---|
| CG-C | 2w | $0.0044 | $0.0115 | $0.0094 | +159% costlier | +112% costlier |
| CG-C | 4w | $0.0048 | $0.0173 | $0.0082 | +262% costlier | +71% costlier |
| CG-C | 8w | $0.0055 | $0.0311 | $0.0148 | +467% costlier | +170% costlier |
| EP-D | 2w | $0.0434 | $0.0303 | $0.0298 | **-30% saving** | **-31% saving** |
| EP-D | 4w | $0.0401 | $0.0313 | $0.0263 | **-22% saving** | **-34% saving** |
| EP-D | 8w | $0.0446 | $0.0419 | $0.0278 | **-6% saving** | **-38% saving** |
| LU-C | 2w | $0.0192 | $0.0193 | $0.0168 | ~0% (break-even) | **-12% saving** |
| LU-C | 4w | $0.0196 | $0.0238 | $0.0170 | +21% costlier | **-13% saving** |
| LU-C | 8w | $0.0214 | $0.0365 | $0.0211 | +71% costlier | ~0% (break-even) |

### 9.1 Configuration-Level Findings

1. **CG-C: FT is never economically justified.** Recovery overhead (92-206 s) is 3-17×
   the base job duration (12-35 s). Spot pricing cannot compensate. Short jobs have no
   economic case for MANA-based FT.

2. **EP-D DEGRADED saves 31-38% vs noFT on-demand at all worker counts.** The saving
   is consistent across cluster sizes. At 8w, REPLACE saves only ~6% while DEGRADED
   saves 38%: EP-D's noFT job is only ~99 s at 8w, and REPLACE adds ~260 s of EC2
   provisioning that eats most of the spot discount.

3. **LU-C DEGRADED saves 12-13% at 2w and 4w but breaks even at 8w.** LU-C at 8
   workers runs only ~47 s noFT. Even DEGRADED's short recovery adds ~89 s total
   (~136 s FT wall time), which at 8 spot workers costs about the same as running 47 s
   on 8 on-demand workers. The 1.6% saving is within noise.

4. **LU-C REPLACE costs 21-71% more than noFT on-demand at 4w and 8w.** With a short
   base job and ~260 s REPLACE recovery, the spot discount cannot compensate.

5. **Fault timing affects cost significantly for REPLACE.** Earlier failures (10%) mean
   the same fixed recovery on a shorter productive run, increasing total cost. Later
   failures (50%) amortize recovery over more P0 time. This effect is largest for REPLACE
   at 8 workers (see Figure 7).

### 9.2 Economic Tendency: When Does Spot FT Actually Pay?

The performance analysis (Section 7.3) showed that DEGRADED wins only when remaining
work is below the crossover threshold of 109 × (N-1) seconds. The same logic applies to
economics: the spot discount is only large enough to offset the recovery cost when the
total runtime extension from recovery is modest.

The economic benefit of fault tolerance comes from running at spot rates (~70% cheaper)
instead of on-demand. The cost of fault tolerance is the extended runtime from recovery
phases. For DEGRADED specifically, this extended runtime comes from P1, P2, and (for
short jobs) a small P3 penalty, all of which are modest for the short-to-medium jobs
tested here.

The pattern across benchmarks reveals two distinct regimes:

**Regime 1: Short jobs (CG-C, and LU-C at 8w):** The base job is so short that even a
single recovery event extends total runtime by 2-17×. No spot discount can compensate
for a 2× runtime extension if it only provides a 70% per-hour discount. FT is not
economically viable regardless of strategy.

**Regime 2: Medium jobs (EP-D, LU-C at 2w/4w):** The base job runs long enough that
the spot discount on the full job outweighs the recovery extension. DEGRADED saves 12-38%
because its recovery is short (P2b fixed at ~11 s) and the spot discount applies to a
significant amount of compute time. REPLACE saves less because its P2b (~120 s) consumes
a larger portion of the spot-rate time.

**For hypothetical long jobs:** The crossover analysis in Section 7.3 shows that
DEGRADED's P3 penalty accumulates with remaining work. For a 4-hour job at 8w with a
50% fault, DEGRADED would run the remaining 2 hours with 7/8 capacity, adding ~17
minutes of extra computation. That extra time, billed at spot rates, costs more than the
P2b savings DEGRADED provides. REPLACE would be both faster and cheaper for such a job.
Paradoxically, the longer the job runs, the more likely REPLACE is the economically
correct choice, even though REPLACE's P2b cost is fixed and large.

The important practical nuance is that spot interruptions are not predictable; a job
cannot choose its fault timing. For short jobs, DEGRADED is a safe economic bet because
the crossover threshold is easy to stay under. For long jobs, REPLACE is the safer choice
because there is no fault timing at which DEGRADED's P3 penalty remains acceptable.

---

## 10. Conclusions

### 10.1 MANA Overhead Characterization

The measured overhead varies between 27% and 222% across benchmarks and cluster sizes
(Table 1). With N=3 repetitions per cell, these figures are stable: the variability is
not EC2 noise but a reproducible property of each configuration. The synthetic studies
establish that MANA adds ~4-5 s for isolated programs (call frequency and simple
imbalance are not the primary drivers), but the mechanism behind the much larger overhead
in real NPB benchmarks remains unresolved (Section 2.4).

LU-C (52-63%) is the most reliable overhead estimate. CG-C at 2w (+222%) is likely a
spin-loop CPU competition cascade between co-located processes. EP-D overhead decreases
with more workers (45-47% at 2w/4w, 27% at 8w) in a pattern that is reproducible but
lacks a mechanistic explanation from this study's data.

### 10.2 Checkpoint Write Time Scales Linearly with Memory

Phase 1 time scales linearly with per-process memory footprint, with low standard
deviation (≤ 0.4 s), confirming that EFS write bandwidth is stable and Phase 1 is
predictable. The AWS EFS Bursting Throughput ceiling (~105 MB/s) limits the feasibility
of MANA-based FT for memory-intensive jobs: per-process footprints beyond ~3-4 GB would
exceed the 2-minute EC2 spot warning before the checkpoint completes.

### 10.3 Recovery Phase Mechanics

The per-phase breakdown shows clear structure:

- P1, P2a, P2c are fixed costs, determined by memory footprint and cluster coordination
  time. Neither strategy can avoid them.
- P2b is the primary differentiator: ~11 s for DEGRADED vs ~111-132 s for REPLACE.
- P3 is the secondary factor. DEGRADED's P3 penalty shrinks as cluster size grows
  (fewer workers lost as a fraction of total), which is the structural reason DEGRADED's
  advantage widens at larger scales, but only while remaining work stays below the
  crossover threshold.

### 10.4 Strategy Recommendations

**DEGRADED_RESUME is the better choice for the short-to-medium jobs evaluated here** (4-8
workers, 100-500 s MANA-noFT time). It wins in all tested cases at 4w and 8w, and in
most at 2w. The structural reason is that the remaining work in our tested jobs is always
below the crossover threshold at which REPLACE would become faster.

**For long-running jobs** (where remaining work exceeds ~763 s at 8w, ~327 s at 4w, or
~109 s at 2w), **REPLACE tends to be the safer and faster choice**, regardless of fault
timing. The longer the job, the more DEGRADED's capacity loss accumulates, eventually
exceeding the P2b savings.

**REPLACE is also better** when multiple successive failures are expected: capacity never
degrades and each failure costs the same fixed P2b.

**For very short jobs** (< 1-2 min noFT), neither strategy is economically justified:
recovery overhead exceeds the spot pricing discount regardless of strategy.

### 10.5 Limitations and Future Work

1. **CG-C anomaly at 2 workers** is unresolved. Per-rank timing instrumentation inside
   the NAS benchmark would confirm or refute the CPU spin-loop cascade hypothesis.
2. **MANA overhead variability** across real benchmarks is not explained. Profiling
   MANA's internal state (coordinator interactions, lock contention patterns) during
   actual benchmark runs would be needed to identify the root cause.
3. **Multiple successive failures** were not tested. DEGRADED's capacity degradation
   under repeated failures is a critical open question for the long-job regime where
   REPLACE may already be preferred.
4. **Only m5.xlarge instances** were evaluated. Different instance types would change
   Phase 1 times and MANA overhead factors.
5. **EFS throughput ceiling** limits checkpoint feasibility for large memory footprints.
   Enabling EFS Provisioned Throughput or using a parallel filesystem would extend the
   range of viable workloads.
6. **No very long workloads** were tested. Directly measuring the performance and economic
   crossover points for hour-long jobs would validate the tendency analysis in Sections
   7.3 and 9.2.

---

*Report generated from 282 valid runs collected 2026-05-30 to 2026-06-14.*
*All figures generated by `TCC/analyze.py`. Raw data: `results_raw.csv`. Tables: `tables/`. Aggregated stats: `summary.md`.*
