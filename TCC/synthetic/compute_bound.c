#include <stdio.h>
#include <stdlib.h>
#include <time.h>

/* Compute-bound kernel: simple floating-point recurrence, stays in
 * registers, touches essentially no memory. Proxy for EP-D's
 * register-heavy profile.
 *
 * Standalone, non-MPI. Used in Phase 10 (TCC), Etapa 13, as part of the
 * exploratory per-core-sharing calibration test (see PHASE10_PLAN.md,
 * Etapa 13). The real code runs embedded in
 * my_clusters/m5-xlarge/2workers/synth/task-calib-core-sharing.yaml,
 * not from this file directly -- kept here as a reference copy, same
 * convention as synth_duration.c. */

int main(int argc, char **argv) {
    long iters = (argc > 1) ? atol(argv[1]) : 20000000000L;
    double x = 1.0000001;
    double acc = 1.0;
    struct timespec t0, t1;
    clock_gettime(CLOCK_MONOTONIC, &t0);
    for (long i = 0; i < iters; i++) {
        acc = acc * x + 1e-10;
        if (acc > 1e8) acc = 1.0;  /* keep bounded, rarely taken */
    }
    clock_gettime(CLOCK_MONOTONIC, &t1);
    double elapsed = (t1.tv_sec - t0.tv_sec) + (t1.tv_nsec - t0.tv_nsec) / 1e9;
    printf("compute_bound: acc=%f elapsed=%.3fs\n", acc, elapsed);
    return 0;
}
