#include <mpi.h>
#include <stdio.h>
#include <stdlib.h>

/* Fixed-work MPI call frequency benchmark.
 * Compile with -DCALL_PERIOD=N to set calls:
 *   -DCALL_PERIOD=0  -> no calls    (L0)
 *   -DCALL_PERIOD=64 -> 50 calls    (L1)
 *   -DCALL_PERIOD=16 -> 200 calls   (L2)
 *   -DCALL_PERIOD=4  -> 800 calls   (L3)
 *   -DCALL_PERIOD=1  -> 3200 calls  (L4)
 * Levels match the MB scale in synth_checkpoint_size (×4 per step).
 *
 * No runtime argument — MANA crashes when argc>1 due to startup stack
 * relocation assuming argc=1. Compile-time constant avoids this.
 *
 * All levels do the same total compute (TOTAL_OUTER * INNER_ITERS ops).
 * noFT: all levels finish in ~the same wall time.
 * MANA-noFT: wall time grows with call frequency (proxy intercept overhead). */

#define TOTAL_OUTER  51200L
#define INNER_ITERS  281250L

#ifndef CALL_PERIOD
#define CALL_PERIOD 0
#endif

int main(int argc, char **argv) {
    MPI_Init(&argc, &argv);
    int rank, nprocs;
    MPI_Comm_rank(MPI_COMM_WORLD, &rank);
    MPI_Comm_size(MPI_COMM_WORLD, &nprocs);

    long call_period = CALL_PERIOD;

    volatile double x = (double)(rank + 1);
    double result = 0.0;
    long call_count = 0;

    double t_start = MPI_Wtime();

    for (long outer = 0; outer < TOTAL_OUTER; outer++) {
        for (long i = 0; i < INNER_ITERS; i++)
            x = x * 1.0000001 + 1e-10;
        if (call_period > 0 && outer % call_period == 0) {
            MPI_Allreduce(&x, &result, 1, MPI_DOUBLE, MPI_SUM, MPI_COMM_WORLD);
            x += result * 1e-20;  /* consume result to prevent dead-code elimination */
            call_count++;
        }
    }

    double elapsed = MPI_Wtime() - t_start;

    double total = 0.0;
    MPI_Reduce(&x, &total, 1, MPI_DOUBLE, MPI_SUM, 0, MPI_COMM_WORLD);
    if (rank == 0)
        printf("synth_mpi_calls: elapsed=%.2fs calls=%ld period=%d nprocs=%d result=%.6e\n",
               elapsed, call_count, CALL_PERIOD, nprocs, total);

    MPI_Finalize();
    return 0;
}
