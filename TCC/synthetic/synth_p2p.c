#include <mpi.h>
#include <stdio.h>
#include <stdlib.h>

/* Fixed-work point-to-point communication benchmark.
 * Compile with -DCALL_PERIOD=N to set p2p call frequency:
 *   -DCALL_PERIOD=0  -> no calls         (L0)
 *   -DCALL_PERIOD=64 -> 50 Send/Recv pairs  (L1)
 *   -DCALL_PERIOD=16 -> 200 pairs           (L2)
 *   -DCALL_PERIOD=4  -> 800 pairs           (L3)
 *   -DCALL_PERIOD=1  -> 3200 pairs          (L4)
 * Levels match the MB scale in synth_checkpoint_size (×4 per step).
 *
 * No runtime argument — MANA crashes when argc>1 due to startup stack
 * relocation assuming argc=1.
 *
 * Unlike synth_mpi_calls (which uses MPI_Allreduce), this test uses
 * MPI_Send + MPI_Recv pairs between cross-node process pairs.
 * MANA wraps MPI_Recv with a busy-poll loop: if no message is available,
 * it releases the checkpoint lock, sleeps 1ms, and retries. Cross-node
 * messages take ~50-100 us to arrive, so each MPI_Recv incurs ~1ms of
 * MANA overhead from the sleep cycle. This is the mechanism responsible
 * for MANA's higher overhead on communication-intensive benchmarks like CG.
 *
 * Partner assignment (4 processes, 2 per node):
 *   rank 0 (node1) <-> rank 2 (node2)
 *   rank 1 (node1) <-> rank 3 (node2)
 * Lower-half ranks send first (deadlock-free ping-pong). */

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
    int partner = (rank + nprocs / 2) % nprocs;  /* cross-node: 0↔2, 1↔3 */

    volatile double x = (double)(rank + 1);
    double result = 0.0;
    long call_count = 0;
    MPI_Status status;

    double t_start = MPI_Wtime();

    for (long outer = 0; outer < TOTAL_OUTER; outer++) {
        for (long i = 0; i < INNER_ITERS; i++)
            x = x * 1.0000001 + 1e-10;

        if (call_period > 0 && outer % call_period == 0) {
            if (rank < nprocs / 2) {
                MPI_Send(&x, 1, MPI_DOUBLE, partner, 0, MPI_COMM_WORLD);
                MPI_Recv(&result, 1, MPI_DOUBLE, partner, 0, MPI_COMM_WORLD, &status);
            } else {
                MPI_Recv(&result, 1, MPI_DOUBLE, partner, 0, MPI_COMM_WORLD, &status);
                MPI_Send(&x, 1, MPI_DOUBLE, partner, 0, MPI_COMM_WORLD);
            }
            x += result * 1e-20;
            call_count++;
        }
    }

    double elapsed = MPI_Wtime() - t_start;

    double total = 0.0;
    MPI_Reduce(&x, &total, 1, MPI_DOUBLE, MPI_SUM, 0, MPI_COMM_WORLD);
    if (rank == 0)
        printf("synth_p2p: elapsed=%.2fs calls=%ld period=%d nprocs=%d result=%.6e\n",
               elapsed, call_count, CALL_PERIOD, nprocs, total);

    MPI_Finalize();
    return 0;
}
