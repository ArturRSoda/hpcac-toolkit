CC     = mpicc
CFLAGS = -O2 -Wall

all: synth_mpi_calls synth_checkpoint_size synth_p2p synth_imbalanced

synth_mpi_calls: synth_mpi_calls.c
	$(CC) $(CFLAGS) -o $@ $<

synth_checkpoint_size: synth_checkpoint_size.c
	$(CC) $(CFLAGS) -o $@ $<

synth_p2p: synth_p2p.c
	$(CC) $(CFLAGS) -o $@ $<

synth_imbalanced: synth_imbalanced.c
	$(CC) $(CFLAGS) -o $@ $<

clean:
	rm -f synth_mpi_calls synth_checkpoint_size synth_p2p synth_imbalanced
