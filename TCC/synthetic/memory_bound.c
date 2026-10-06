#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

/* Memory-bound kernel: same loop as synth_duration.c, minus MPI.
 * One add per access into a 50MB buffer, larger than per-core cache.
 *
 * Standalone, non-MPI. Used in Phase 10 (TCC), Etapa 13, as part of the
 * exploratory per-core-sharing calibration test (see PHASE10_PLAN.md,
 * Etapa 13). The real code runs embedded in
 * my_clusters/m5-xlarge/2workers/synth/task-calib-core-sharing.yaml,
 * not from this file directly -- kept here as a reference copy, same
 * convention as synth_duration.c. */

#define MEM_MB 50

int main(int argc, char **argv) {
    long iters = (argc > 1) ? atol(argv[1]) : 3200000000L;
    long n = (MEM_MB * 1024L * 1024L) / sizeof(double);
    double *buf = (double *)malloc(n * sizeof(double));
    memset(buf, 0x42, n * sizeof(double));

    struct timespec t0, t1;
    clock_gettime(CLOCK_MONOTONIC, &t0);
    for (long i = 0; i < iters; i++) {
        buf[i % n] += 1.0;
    }
    clock_gettime(CLOCK_MONOTONIC, &t1);
    double elapsed = (t1.tv_sec - t0.tv_sec) + (t1.tv_nsec - t0.tv_nsec) / 1e9;
    printf("memory_bound: elapsed=%.3fs\n", elapsed);
    free(buf);
    return 0;
}
