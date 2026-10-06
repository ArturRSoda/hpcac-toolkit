#include <mpi.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* Fixed-memory, fixed-WORK benchmark (not fixed-time). Compile with
 * -DTOTAL_ITERS=N to set the iteration count.
 *
 * No runtime argument -- MANA crashes when argc>1 due to startup stack
 * relocation assuming argc=1. Compile-time constant avoids this.
 *
 * Designed for Phase 10 (TCC), Etapa 11.2: validates that the ADAPTIVE
 * decision threshold (remaining_secs > 389s) holds across cluster sizes
 * (N=2, 4, 8), which the NPB benchmarks alone cannot test because only
 * EP-D at N=2 reaches a large enough T_MANA_noFT to cross it.
 *
 * IMPORTANT: an earlier version of this file used a wall-clock loop
 * (`while (MPI_Wtime() - t_start < DURATION_SECS)`), which turned out to be
 * broken for this purpose: MPI_Wtime() reflects real elapsed time regardless
 * of CPU throughput, so an oversubscribed (DEGRADED) process "waits out the
 * clock" exactly as fast as a non-oversubscribed (REPLACE) one -- the loop
 * doesn't slow down under contention, so it can never reproduce the
 * DEGRADED-is-slower dynamic the ADAPTIVE formula depends on. A first smoke
 * test confirmed this: total wall time converged to ~DURATION_SECS regardless
 * of strategy, because each run's own recovery gap got "absorbed" into its
 * time budget instead of adding a genuine slowdown.
 *
 * Fixed iteration count fixes this: completion time now genuinely depends on
 * available CPU throughput, so an oversubscribed process takes longer in
 * wall-clock terms to finish the SAME amount of work, matching how real
 * compute-bound MPI applications (like EP-D) behave.
 *
 * TOTAL_ITERS must be calibrated per target hardware (see
 * TCC/artifacts/phase10/PHASE10_PLAN.md, Etapa 11.2, calibration note) to
 * approximate the intended T_synth (600s for this experiment) on ONE
 * non-oversubscribed core.
 *
 * memset forces the OS to map all allocated pages before the checkpoint
 * fires, so the checkpoint image is realistic even though small. The
 * sequential (not random) index pattern (i % n cycling 0..n-1 in order)
 * keeps memory access cache/prefetch-friendly, so cost per iteration is
 * dominated by the modulo + add, not by memory latency. */

#define MEM_MB 50

#ifndef TOTAL_ITERS
/* Calibrated on m5.xlarge (2026-09-16): 3e9 iters took 13.9s
 * (~215.8M iters/s per non-oversubscribed core) -> 129.5e9 iters ~= 600s. */
#define TOTAL_ITERS 129500000000L
#endif

int main(int argc, char **argv) {
    MPI_Init(&argc, &argv);
    int rank, nprocs;
    MPI_Comm_rank(MPI_COMM_WORLD, &rank);
    MPI_Comm_size(MPI_COMM_WORLD, &nprocs);

    long n = (MEM_MB * 1024L * 1024L) / sizeof(double);

    double *buf = (double *)malloc(n * sizeof(double));
    if (!buf) {
        fprintf(stderr, "rank %d: malloc(%d MB) failed\n", rank, MEM_MB);
        MPI_Abort(MPI_COMM_WORLD, 1);
    }
    memset(buf, 0x42, n * sizeof(double));  /* force OS to map all pages */

    if (rank == 0)
        printf("synth_duration: %d MB/proc, %d procs, %ld iters\n",
               MEM_MB, nprocs, (long)TOTAL_ITERS);

    double t_start = MPI_Wtime();

    for (long i = 0; i < TOTAL_ITERS; i++) {
        buf[i % n] += 1.0;
        if (i % 5000000L == 0)
            MPI_Barrier(MPI_COMM_WORLD);
    }

    double elapsed = MPI_Wtime() - t_start;

    if (rank == 0)
        printf("synth_duration: done elapsed=%.1fs\n", elapsed);

    free(buf);
    MPI_Finalize();
    return 0;
}
