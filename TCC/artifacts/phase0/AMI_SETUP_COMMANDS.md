# Amazon Linux 2023 Setup Commands for HPC@Cloud + MANA + Slurm + NPB

Target host profile:
- Instance type: t3.medium
- OS: Amazon Linux 2023
- Purpose: Build a reusable AMI with required libraries and applications

Notes:
- Run as ec2-user with sudo permissions.
- Commands are organized in execution order.
- If a command fails, stop and fix before continuing.

---

## 1) Base system update and core tools

```bash
sudo dnf clean all
sudo dnf makecache
sudo dnf -y update

sudo dnf -y --allowerasing install \
  git wget curl tar unzip which patch file rsync \
  gcc gcc-c++ gcc-gfortran make cmake \
  autoconf automake libtool flex bison \
  python3 python3-pip \
  perl \
  libatomic \
  pam-devel readline-devel ncurses-devel libcurl-devel json-c-devel \
  openssl-devel zlib-devel libevent-devel hwloc-devel numactl-devel \
  nfs-utils procps-ng hostname
```

---

## 2) Create working directories

```bash
mkdir -p "$HOME/src" "$HOME/build" "$HOME/tools" "$HOME/downloads"
cd "$HOME"
```

---

## 3) Install MPICH 3.3.2 (source build, pinned version)

```bash
cd "$HOME/downloads"
MPICH_VERSION=3.3.2
wget -O "mpich-${MPICH_VERSION}.tar.gz" \
  "https://www.mpich.org/static/downloads/${MPICH_VERSION}/mpich-${MPICH_VERSION}.tar.gz"

tar -xzf "mpich-${MPICH_VERSION}.tar.gz"
cd "mpich-${MPICH_VERSION}"

# MPICH 3.3.2 + newer GCC/gfortran requires this compatibility flag.
# Without it, configure may fail with:
# "will not compile files that call the same routine with arguments of different types"
export FFLAGS="-O2 -fallow-argument-mismatch"
export FCFLAGS="-O2 -fallow-argument-mismatch"

./configure --prefix=/opt/mpich-${MPICH_VERSION} \
  FFLAGS="$FFLAGS" FCFLAGS="$FCFLAGS"
make -j"$(nproc)"
sudo make install

sudo tee /etc/profile.d/mpich.sh >/dev/null <<'EOF'
export MPI_HOME=/opt/mpich-3.3.2
export PATH=$MPI_HOME/bin:$PATH
export LD_LIBRARY_PATH=$MPI_HOME/lib:$LD_LIBRARY_PATH
EOF

source /etc/profile.d/mpich.sh
mpirun --version
mpicc --version
```

---

## 4) Install MANA (build from source)

```bash
cd "$HOME/src"
if [ ! -d mana ]; then
  git clone https://github.com/mpickpt/mana.git
fi

cd mana
git submodule init
git submodule update

./configure
make -j"$(nproc)" mana

sudo mkdir -p /opt/mana
sudo rsync -a ./ /opt/mana/

sudo tee /etc/profile.d/mana.sh >/dev/null <<'EOF'
export MANA_ROOT=/opt/mana
export PATH=$MANA_ROOT/bin:$PATH
EOF

source /etc/profile.d/mana.sh
# Do not use `mana_launch --help` here: current script may parse args before help
# and throw IndexError when no executable is provided.
# Do not use `mana_status --help` before coordinator: it expects .mana.rc state file.
command -v mana_coordinator
command -v mana_launch
command -v mana_restart
command -v mana_status

# Check the underlying DMTCP binaries can run.
dmtcp_coordinator --help | head -n 3 || true
dmtcp_command --help | head -n 3 || true

# Safe runtime sanity checks for MANA wrappers
mana_restart --help | head -n 3 || true
mana_coordinator --help | head -n 3 || true

cd /opt/mana
git rev-parse HEAD || true
```

### 4.1 If `./configure` fails with `Page size not found; Manually change include/config.h`

This comes from the embedded DMTCP configure test in some environments.

```bash
cd "$HOME/src/mana/dmtcp"

# Quick sanity check: page size must print a number like 4096
getconf PAGESIZE

# If your shell/home mount is noexec, configure test binaries cannot run.
# This must NOT include noexec:
mount | grep ' on /home '

# Clean partial configure state
make distclean || true
rm -f config.cache

# Retry configure
cd "$HOME/src/mana"
./configure
```

If it still fails, apply a fallback that uses `getconf PAGESIZE` for the DMTCP test:

```bash
cd "$HOME/src/mana/dmtcp"

# Replace the failure branch with a fallback page size value
sed -i '/sysconf_pagesize=.unknown./,/fi/{s/sysconf_pagesize=.unknown./sysconf_pagesize=$(getconf PAGESIZE 2>\/dev\/null || echo 4096)/; s/AC_MSG_FAILURE.*/AC_MSG_NOTICE([Using fallback page size: $sysconf_pagesize])/}' configure

cd "$HOME/src/mana"
./configure
make -j"$(nproc)" mana
```

### 4.1.1 If MANA shows `libatomic.so.1: cannot open shared object file`

```bash
sudo dnf -y install libatomic

# Ensure dynamic linker cache is updated
sudo ldconfig

# Verify availability
ldconfig -p | grep libatomic

# Re-test
source /etc/profile.d/mana.sh
dmtcp_coordinator --help | head -n 3
```

---

### 4.2 Important launcher note for MPICH 3.3.2

For MPICH 3.3.2, MANA multi-rank restart is known to be unreliable with Hydra (`mpirun`) and can terminate with signal 9 after restore.

Recommended for this project:
- Use Slurm launch path (`srun`) for launch and restart during experiments.
- Avoid Hydra-based restart for multi-rank jobs in this stack.
- Compile MPI apps with regular `mpicc` (not `mpicc_mana`) when using MPICH 3.x.

---

## 5) Install Slurm + Munge (AL2023)

### 5.1 Install Munge from dnf

```bash
sudo dnf -y install munge munge-libs munge-devel
```

### 5.2 Build Slurm from source (primary path on AL2023)

```bash
cd "$HOME/downloads"
SLURM_VERSION=24.05.4
wget -O "slurm-${SLURM_VERSION}.tar.bz2" \
  "https://download.schedmd.com/slurm/slurm-${SLURM_VERSION}.tar.bz2"

tar -xjf "slurm-${SLURM_VERSION}.tar.bz2"
cd "slurm-${SLURM_VERSION}"

./configure \
  --prefix=/opt/slurm-${SLURM_VERSION} \
  --sysconfdir=/etc/slurm \
  --localstatedir=/var \
  --with-munge=/usr
make -j"$(nproc)"
sudo make install

sudo tee /etc/profile.d/slurm.sh >/dev/null <<'EOF'
export SLURM_HOME=/opt/slurm-24.05.4
export PATH=$SLURM_HOME/bin:$SLURM_HOME/sbin:$PATH
EOF

source /etc/profile.d/slurm.sh

slurmctld --version
slurmd --version
```

### 5.3 Create service user and runtime directories

```bash
sudo id -u slurm >/dev/null 2>&1 || sudo useradd -r -M -s /sbin/nologin slurm
sudo mkdir -p /etc/slurm /var/spool/slurmctld /var/spool/slurmd /var/log/slurm
sudo chown -R slurm:slurm /var/spool/slurmctld /var/spool/slurmd /var/log/slurm
```

### 5.4 Optional: systemd unit files for source-built Slurm

```bash
sudo tee /etc/systemd/system/slurmctld.service >/dev/null <<'EOF'
[Unit]
Description=Slurm controller daemon
After=network.target munge.service
Requires=munge.service

[Service]
Type=simple
User=slurm
Group=slurm
ExecStart=/opt/slurm-24.05.4/sbin/slurmctld -D -s
Restart=on-failure

[Install]
WantedBy=multi-user.target
EOF

sudo tee /etc/systemd/system/slurmd.service >/dev/null <<'EOF'
[Unit]
Description=Slurm node daemon
After=network.target munge.service
Requires=munge.service

[Service]
Type=simple
ExecStart=/opt/slurm-24.05.4/sbin/slurmd -D -s
Restart=on-failure

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
```

### 5.5 Generate slurm.conf and cgroup.conf

> **AL2023 note:** The cgroup v2 plugin is not available in the Slurm 24.x build on AL2023.
> You must set `CgroupPlugin=disabled` and `TaskPlugin=task/none` or slurmd will crash on start.

```bash
# Capture hardware topology from this node
HOSTNAME=$(hostname -s)
NODE_PARAMS=$(slurmd -C 2>/dev/null | grep ^NodeName | head -n 1 | sed 's/NodeName=[^ ]*/NodeName=PLACEHOLDER/')

sudo tee /etc/slurm/slurm.conf >/dev/null <<EOF
ClusterName=local
SlurmctldHost=${HOSTNAME}
SlurmUser=slurm
StateSaveLocation=/var/spool/slurmctld
SlurmdSpoolDir=/var/spool/slurmd
SlurmctldPidFile=/var/run/slurmctld.pid
SlurmdPidFile=/var/run/slurmd.pid
ProctrackType=proctrack/linuxproc
SelectType=select/cons_tres
SelectTypeParameters=CR_Core_Memory
TaskPlugin=task/none
MpiDefault=pmi2
JobAcctGatherType=jobacct_gather/none
EOF

# Append the node line with actual hardware values
slurmd -C 2>/dev/null | grep ^NodeName | head -n 1 | sudo tee -a /etc/slurm/slurm.conf

sudo tee -a /etc/slurm/slurm.conf >/dev/null <<EOF

PartitionName=debug Nodes=${HOSTNAME} Default=YES MaxTime=INFINITE State=UP
EOF

# cgroup.conf: disable the cgroup plugin entirely (required on AL2023)
sudo tee /etc/slurm/cgroup.conf >/dev/null <<'EOF'
CgroupPlugin=disabled
ConstrainCores=no
ConstrainRAMSpace=no
ConstrainSwapSpace=no
ConstrainDevices=no
EOF

# Verify the files look right
echo '--- slurm.conf ---'
cat /etc/slurm/slurm.conf
echo '--- cgroup.conf ---'
cat /etc/slurm/cgroup.conf
```

### 5.6 Start and validate Slurm daemons

```bash
sudo systemctl enable slurmctld slurmd
sudo systemctl restart slurmctld slurmd

# Give daemons a moment to register
sleep 3

scontrol show node
# Node State should be IDLE or IDLE*; "unknown*" means slurmd did not register
# If state is "down" check: sudo journalctl -u slurmd -n 30
```

---

## 6) Configure Munge service (required by Slurm)

```bash
sudo mkdir -p /etc/munge /var/log/munge /var/lib/munge
sudo chown -R munge:munge /etc/munge /var/log/munge /var/lib/munge
sudo chmod 0700 /etc/munge

if [ ! -f /etc/munge/munge.key ]; then
  if command -v create-munge-key >/dev/null 2>&1; then
    sudo create-munge-key
  elif command -v mungekey >/dev/null 2>&1; then
    # Some distros provide mungekey instead of create-munge-key
    sudo mungekey --verbose
  else
    # Portable fallback: generate a strong random key directly
    sudo dd if=/dev/urandom bs=1 count=1024 of=/etc/munge/munge.key status=none
  fi
fi

sudo chown munge:munge /etc/munge/munge.key
sudo chmod 0400 /etc/munge/munge.key

sudo systemctl enable munge
sudo systemctl restart munge
sudo systemctl status munge --no-pager

# Validate munge end-to-end
munge -n | unmunge | head -n 5
```

---

## 7) Install and build NPB (MPI)

```bash
cd "$HOME/downloads"
wget -O NPB3.4.4.tar.gz \
  "https://www.nas.nasa.gov/assets/npb/NPB3.4.4.tar.gz"

tar -xzf NPB3.4.4.tar.gz
cd NPB3.4.4/NPB3.4-MPI

cp config/make.def.template config/make.def

# Update compiler settings to MPICH binaries
sed -i 's|^MPIF77 *=.*|MPIF77 = mpif77|g' config/make.def
sed -i 's|^MPICC *=.*|MPICC = mpicc|g' config/make.def

# Low-cost baseline builds
make EP CLASS=A
make CG CLASS=A

ls -lh bin/ep.A.x bin/cg.A.x
```

---

## 8) Quick validation commands

```bash
source /etc/profile.d/mpich.sh
source /etc/profile.d/mana.sh

mpirun --version
python3 --version
dmtcp_coordinator --help >/dev/null
slurmctld --version
slurmd --version

# NPB binaries exist
ls -lh "$HOME/downloads/NPB3.4.4/NPB3.4-MPI/bin/ep.A.x"
ls -lh "$HOME/downloads/NPB3.4.4/NPB3.4-MPI/bin/cg.A.x"
```

---

## 9) Prepare instance for AMI snapshot

```bash
# Remove shell history if desired
history -c || true

# Remove temporary build artifacts if you want a smaller AMI
rm -rf "$HOME/build" || true

# Do not bake AWS credentials
rm -f "$HOME/.aws/credentials" "$HOME/.aws/config" || true

# Remove SSH host keys so they regenerate on first boot
sudo rm -f /etc/ssh/ssh_host_*

# Remove slurm.conf — it contains the current hostname and cannot be reused on other instances.
# cgroup.conf is safe to keep (it has no host-specific values).
# slurm.conf must be regenerated at first boot (see section 5.5).
sudo rm -f /etc/slurm/slurm.conf

sudo sync
sudo shutdown -h now
```

After stop:
- Create AMI from this instance in EC2 console.
- Record AMI ID and use it in cluster config.

---

## 10) Post-AMI first-boot checklist (per node)

```bash
source /etc/profile.d/mpich.sh
source /etc/profile.d/mana.sh
source /etc/profile.d/slurm.sh

command -v mpirun
command -v mana_launch
command -v slurmctld
command -v slurmd
systemctl is-enabled munge

# Regenerate slurm.conf for this instance's hostname and hardware.
# slurm.conf is NOT baked into the AMI because it contains the hostname.
# cgroup.conf is already present from the AMI (it has no host-specific values).
HOSTNAME=$(hostname -s)

sudo tee /etc/slurm/slurm.conf >/dev/null <<EOF
ClusterName=local
SlurmctldHost=${HOSTNAME}
SlurmUser=slurm
StateSaveLocation=/var/spool/slurmctld
SlurmdSpoolDir=/var/spool/slurmd
SlurmctldPidFile=/var/run/slurmctld.pid
SlurmdPidFile=/var/run/slurmd.pid
ProctrackType=proctrack/linuxproc
SelectType=select/cons_tres
SelectTypeParameters=CR_Core_Memory
TaskPlugin=task/none
MpiDefault=pmi2
JobAcctGatherType=jobacct_gather/none
EOF

slurmd -C 2>/dev/null | grep ^NodeName | head -n 1 | sudo tee -a /etc/slurm/slurm.conf

sudo tee -a /etc/slurm/slurm.conf >/dev/null <<EOF

PartitionName=debug Nodes=${HOSTNAME} Default=YES MaxTime=INFINITE State=UP
EOF

# Slurm smoke test (single node)
sudo systemctl restart munge slurmctld slurmd
sleep 3
scontrol show node   # should show State=IDLE

# NPB sanity run under Slurm
srun -N1 -n2 "$HOME/downloads/NPB3.4.4/NPB3.4-MPI/bin/ep.A.x"
```

### 10.1 MANA checkpoint/restart smoke test (single node)

This validates that MANA can checkpoint a running MPI job and restart it from the saved state.
Use MANA's built-in mpi_hello_world test first because it is a known-good target for this stack.

```bash
source /etc/profile.d/mpich.sh
source /etc/profile.d/mana.sh
source /etc/profile.d/slurm.sh

mkdir -p ~/mana_test/ckpt
cd ~/mana_test

cd /opt/mana/mpi-proxy-split/test
make mpi_hello_world.mana.exe
HELLO_BIN="/opt/mana/mpi-proxy-split/test/mpi_hello_world.mana.exe"

# Preflight: avoid passing an empty executable argument to mana_launch.
if [ -z "${HELLO_BIN:-}" ] || [ ! -x "$HELLO_BIN" ]; then
  echo "ERROR: MANA hello-world binary not found or not executable: $HELLO_BIN"
  ls -lh /opt/mana/mpi-proxy-split/test/ || true
  exit 1
fi
echo "Using test binary: $HELLO_BIN"
printf 'HELLO_BIN shell-escaped: %q\n' "$HELLO_BIN"

# Reserve one Slurm allocation and keep all MANA commands inside it.
# This guarantees a stable SLURM_JOB_ID and matching ~/.mana-slurm-$SLURM_JOB_ID.rc.
salloc -N1 -n2 -t 00:30:00
```

**Step 1 — Launch under MANA:**

```bash
# Start coordinator explicitly (required by mana_launch/mana_restart)
mana_coordinator --exit-on-last --daemon

# Launch with Slurm from the same allocation (NOT mpirun/Hydra for MPICH 3.3.2)
srun -N1 -n2 /opt/mana/bin/mana_launch --ckptdir "$HOME/mana_test/ckpt" /opt/mana/mpi-proxy-split/test/mpi_hello_world.mana.exe &
MANA_PID=$!

# Give processes time to start and register with the coordinator
sleep 8
mana_status --list   # should show 2 processes in "Running" state
```

**Step 2 — Checkpoint:**

```bash
# Request a checkpoint; MANA will pause all ranks, write images, then resume
mana_status --checkpoint
sleep 6

# Confirm checkpoint images were written
ls -lh ~/mana_test/ckpt/
# Expect files like: ckpt_rank-0.dmtcp  ckpt_rank-1.dmtcp  and a ckpt_*.dmtcp header

# Stop the running job (simulates spot interruption)
mana_status --quit
echo "Job terminated."
```

**Step 3 — Restart from checkpoint:**

```bash
# Restart resumes execution from where the checkpoint was taken.
# Use srun + mana_restart (NOT mpirun) — Hydra-based restart is broken for MPICH 3.3.2.
mana_coordinator --exit-on-last --daemon
srun -N1 -n2 mana_restart --restartdir ~/mana_test/ckpt
```

Expected outcome: both ranks print hello-world, sleep, checkpoint successfully, and then continue after restart.

**If restart hangs or segfaults:**
```bash
# Check coordinator log for errors
cat /tmp/dmtcp_coordinator.log 2>/dev/null | tail -20 || true

# Confirm checkpoint files are not zero-size
ls -lh ~/mana_test/ckpt/

# Try with explicit coordinator port to avoid conflicts
mana_coordinator --daemon --exit-on-last --port 7779
srun -N1 -n2 mana_restart --coord-port 7779 --restartdir ~/mana_test/ckpt
```

When done, leave the allocation:
```bash
exit
```

Optional follow-up after hello-world passes:
```bash
# Longer real workload checkpoint/restart validation
EP_BIN="$HOME/downloads/NPB3.4.4/NPB3.4-MPI/bin/ep.A.x"
mana_coordinator --exit-on-last --daemon
srun -N1 -n2 mana_launch --ckptdir ~/mana_test/ckpt "$EP_BIN" &
sleep 2
mana_status --checkpoint
sleep 5
mana_status --quit
mana_coordinator --exit-on-last --daemon
srun -N1 -n2 mana_restart --restartdir ~/mana_test/ckpt
```

If all commands are available, the Slurm smoke test passes, and MANA checkpoint/restart completes successfully, the AMI is ready for cluster provisioning.
