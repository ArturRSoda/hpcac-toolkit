#include <mpi.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* Allocates N MB per process and runs through a checkpoint+restart cycle.
 * Compile with -DMEM_MB=N to set memory per process:
 *   -DMEM_MB=50   -DMEM_MB=100  -DMEM_MB=200  -DMEM_MB=400  -DMEM_MB=800
 *
 * No runtime argument — MANA crashes when argc>1 due to startup stack
 * relocation assuming argc=1. Compile-time constant avoids this.
 *
 * Designed for DEGRADED_RESUME with trigger_after_secs=30.
 * Phase 1 time (detection → checkpoint written) scales with MEM_MB,
 * producing a direct memory-footprint → checkpoint-write-time curve.
 *
 * memset forces the OS to map all allocated pages before the checkpoint fires,
 * so the checkpoint image truly reflects the requested memory footprint. */

#define TARGET_SECS 120.0

#ifndef MEM_MB
#define MEM_MB 100
#endif

int main(int argc, char **argv) {
    MPI_Init(&argc, &argv);
    int rank, nprocs;
    MPI_Comm_rank(MPI_COMM_WORLD, &rank);
    MPI_Comm_size(MPI_COMM_WORLD, &nprocs);

    long mem_mb = MEM_MB;
    long n = (mem_mb * 1024L * 1024L) / sizeof(double);

    double *buf = (double *)malloc(n * sizeof(double));
    if (!buf) {
        fprintf(stderr, "rank %d: malloc(%ld MB) failed\n", rank, mem_mb);
        MPI_Abort(MPI_COMM_WORLD, 1);
    }
    memset(buf, 0x42, n * sizeof(double));  /* force OS to map all pages */

    if (rank == 0)
        printf("synth_ckpt_size: %d MB/proc, %d procs, running %.0fs\n",
               MEM_MB, nprocs, TARGET_SECS);

    double t_start = MPI_Wtime();
    volatile long counter = 0;

    while (MPI_Wtime() - t_start < TARGET_SECS) {
        buf[counter % n] += 1.0;  /* keep buffer live so it appears in checkpoint */
        counter++;
        if (counter % 5000000L == 0)
            MPI_Barrier(MPI_COMM_WORLD);
    }

    if (rank == 0)
        printf("synth_ckpt_size: done counter=%ld\n", counter);

    free(buf);
    MPI_Finalize();
    return 0;
}
