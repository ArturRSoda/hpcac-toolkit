#include <mpi.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

/* Communication-imbalance benchmark using MPI_Irecv + MPI_Send + MPI_Wait.
 * Compile with -DDELAY_US=N to control how long the sender sleeps before
 * sending, forcing the receiver to wait in MPI_Wait:
 *   -DDELAY_US=0      -> no delay (L0) — perfectly synchronized
 *   -DDELAY_US=100    -> 100 µs per exchange (L1) — ~80ms total
 *   -DDELAY_US=1000   -> 1 ms per exchange (L2) — ~800ms total
 *   -DDELAY_US=5000   -> 5 ms per exchange (L3) — ~4s total
 *   -DDELAY_US=20000  -> 20 ms per exchange (L4) — ~16s total
 *
 * MOTIVATION:
 * Previous tests (synth_calls, synth_p2p) showed MANA overhead is flat
 * regardless of call count. Both used synchronized patterns where the
 * message arrives before (or immediately when) the receiver polls.
 *
 * Real benchmarks (CG, LU) use MPI_Irecv + MPI_Send + MPI_Wait, not
 * MPI_Send + MPI_Recv. MANA's MPI_Wait implementation is a TIGHT SPIN LOOP:
 *
 *   while (!flag) {
 *     DMTCP_PLUGIN_DISABLE_CKPT();  // acquire RW lock
 *     MPI_Test_internal(...);
 *     DMTCP_PLUGIN_ENABLE_CKPT();   // release RW lock
 *   }
 *
 * Unlike MPI_Recv (which sleeps 1ms after 1000 failed Iprobes), MPI_Wait
 * has no sleep — it burns lock/unlock cycles proportional to wait time.
 * A sender delay of Δt causes ~Δt / T_iter lock/unlock pairs per exchange,
 * where T_iter is a few microseconds. MANA overhead should grow linearly
 * with DELAY_US.
 *
 * This test directly isolates communication imbalance as the overhead driver,
 * explaining why CG at 2w (large per-process partition → more computation
 * between exchanges → more partner drift) has 251% overhead while 4w/8w
 * show only ~55%.
 *
 * Communication pattern (4 processes, 2 per node):
 *   rank 0 (node1) <-> rank 2 (node2) : rank 0 is sender (delays), rank 2 waits
 *   rank 1 (node1) <-> rank 3 (node2) : rank 1 is sender (delays), rank 3 waits
 * Lower-half ranks (0,1) sleep DELAY_US then send; upper-half ranks (2,3) post
 * Irecv immediately then call MPI_Wait. */

#define TOTAL_OUTER  51200L
#define INNER_ITERS  281250L
#define CALL_PERIOD  64      /* 800 exchanges total — same as synth_p2p L1 */

#ifndef DELAY_US
#define DELAY_US 0
#endif

int main(int argc, char **argv) {
    MPI_Init(&argc, &argv);
    int rank, nprocs;
    MPI_Comm_rank(MPI_COMM_WORLD, &rank);
    MPI_Comm_size(MPI_COMM_WORLD, &nprocs);

    int partner = (rank + nprocs / 2) % nprocs;
    int is_sender = (rank < nprocs / 2);  /* lower-half ranks are senders */

    volatile double x = (double)(rank + 1);
    double recv_buf = 0.0;
    long call_count = 0;
    MPI_Request request;
    MPI_Status status;

    double t_start = MPI_Wtime();

    for (long outer = 0; outer < TOTAL_OUTER; outer++) {
        for (long i = 0; i < INNER_ITERS; i++)
            x = x * 1.0000001 + 1e-10;

        if (outer % CALL_PERIOD == 0) {
            if (is_sender) {
                /* sender: delay to create imbalance, then send */
                if (DELAY_US > 0)
                    usleep(DELAY_US);
                MPI_Send(&x, 1, MPI_DOUBLE, partner, 0, MPI_COMM_WORLD);
                /* sender also receives reply (symmetric exchange) */
                MPI_Irecv(&recv_buf, 1, MPI_DOUBLE, partner, 1, MPI_COMM_WORLD, &request);
                MPI_Wait(&request, &status);
            } else {
                /* receiver: post Irecv immediately, wait in MPI_Wait */
                MPI_Irecv(&recv_buf, 1, MPI_DOUBLE, partner, 0, MPI_COMM_WORLD, &request);
                MPI_Wait(&request, &status);
                /* send reply */
                MPI_Send(&x, 1, MPI_DOUBLE, partner, 1, MPI_COMM_WORLD);
            }
            x += recv_buf * 1e-20;
            call_count++;
        }
    }

    double elapsed = MPI_Wtime() - t_start;

    double total = 0.0;
    MPI_Reduce(&x, &total, 1, MPI_DOUBLE, MPI_SUM, 0, MPI_COMM_WORLD);
    if (rank == 0)
        printf("synth_imbalanced: elapsed=%.2fs calls=%ld delay_us=%d nprocs=%d result=%.6e\n",
               elapsed, call_count, DELAY_US, nprocs, total);

    MPI_Finalize();
    return 0;
}
