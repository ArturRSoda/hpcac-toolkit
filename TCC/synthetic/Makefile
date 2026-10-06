CC     = mpicc
CFLAGS = -O2 -Wall

all: synth_mpi_calls synth_checkpoint_size synth_p2p synth_imbalanced synth_duration

synth_mpi_calls: synth_mpi_calls.c
	$(CC) $(CFLAGS) -o $@ $<

synth_checkpoint_size: synth_checkpoint_size.c
	$(CC) $(CFLAGS) -o $@ $<

synth_p2p: synth_p2p.c
	$(CC) $(CFLAGS) -o $@ $<

synth_imbalanced: synth_imbalanced.c
	$(CC) $(CFLAGS) -o $@ $<

# TOTAL_ITERS calibrated on m5.xlarge (2026-09-16): 3e9 iters took 13.9s
# (~215.8M iters/s per non-oversubscribed core) -> 129.5e9 iters ~= 600s.
# See PHASE10_PLAN.md, Etapa 11.2, calibration note.
synth_duration: synth_duration.c
	$(CC) $(CFLAGS) -DTOTAL_ITERS=129500000000L -o $@ $<

clean:
	rm -f synth_mpi_calls synth_checkpoint_size synth_p2p synth_imbalanced synth_duration
