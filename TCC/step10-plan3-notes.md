# 10.0

```bash
…/hpcac-toolkit on  tcc [!?] via 🦀 v1.95.0 on ☁️  (us-east-1) took 9s 
➜ cargo run cluster spawn --cluster-id ClusterMANA     
warning: associated function `fetch_all_by_cluster_id` is never used
  --> src/database/models/interruption_event.rs:63:18
   |
20 | impl InterruptionEvent {
   | ---------------------- associated function in this implementation
...
63 |     pub async fn fetch_all_by_cluster_id(
   |                  ^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: `fastcluster` (bin "fastcluster") generated 1 warning
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.13s
     Running `target/debug/fastcluster cluster spawn --cluster-id ClusterMANA`

Cluster Name                       : MyClusterMANA
Provider                           : aws
Region                             : us-east-1
Availability Zone                  : us-east-1a
Use Node Affinity                  : false
Use Elastic Fabric Adapters (EFAs) : false
Use Elastic File System (EFS)      : true
On Instance Creation Failure       : migrate
Provider Config                    : Artur
Node Count                         : 3

Node Details:
  Node 1:
    Instance Type   : t3.medium
    Processor       : 1-Core x86_64 Intel
    vCPUs:          : 2
    GPUs:           : N/A
    Image ID        : ami-08b9f0fb120be798a
    Allocation Mode : on-demand
    Burstable Mode  : Unlimited

  Node 2:
    Instance Type   : m5.large
    Processor       : 1-Core x86_64 Intel
    vCPUs:          : 2
    GPUs:           : N/A
    Image ID        : ami-08b9f0fb120be798a
    Allocation Mode : spot
    Burstable Mode  : N/A

  Node 3:
    Instance Type   : m5.large
    Processor       : 1-Core x86_64 Intel
    vCPUs:          : 2
    GPUs:           : N/A
    Image ID        : ami-08b9f0fb120be798a
    Allocation Mode : spot
    Burstable Mode  : N/A

> Do you want to proceed spawning this cluster? Yes
Confirmed! Proceeding...

 ATTEMPT: 1
[00:02:21] ########################################      40/40      Cluster 'MyClusterMANA' spawned successfully!
[00:02:21]   All Cloud operations completed                                                                                               
Cluster spawn completed successfully. You can access your nodes using:
Node '10.0.0.11': ssh ec2-user@54.145.68.129
Node '10.0.0.10': ssh ec2-user@3.219.39.255
Node '10.0.0.12': ssh ec2-user@100.51.86.2
```

# 10.1

```bash
…/hpcac-toolkit on  tcc [!?] via 🦀 v1.95.0 on ☁️  (us-east-1) 
➜ cargo run -- cluster run-task --cluster-id ClusterMANA --file tasks-bad.yaml -y

warning: associated function `fetch_all_by_cluster_id` is never used
  --> src/database/models/interruption_event.rs:63:18
   |
20 | impl InterruptionEvent {
   | ---------------------- associated function in this implementation
...
63 |     pub async fn fetch_all_by_cluster_id(
   |                  ^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: `fastcluster` (bin "fastcluster") generated 1 warning
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.17s
     Running `target/debug/fastcluster cluster run-task --cluster-id ClusterMANA --file tasks-bad.yaml -y`
Error: Invalid fault_tolerance.strategy 'invalid'. Expected NONE, REPLACE_RESUME, or DEGRADED_RESUME.

…/hpcac-toolkit on  tcc [!?] via 🦀 v1.95.0 on ☁️  (us-east-1) 
✗ cargo run -- cluster run-task --cluster-id ClusterMANA --file tasks-bad-replace-mode.yaml -y 

warning: associated function `fetch_all_by_cluster_id` is never used
  --> src/database/models/interruption_event.rs:63:18
   |
20 | impl InterruptionEvent {
   | ---------------------- associated function in this implementation
...
63 |     pub async fn fetch_all_by_cluster_id(
   |                  ^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: `fastcluster` (bin "fastcluster") generated 1 warning
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.13s
     Running `target/debug/fastcluster cluster run-task --cluster-id ClusterMANA --file tasks-bad-replace-mode.yaml -y`
Error: Invalid fault_tolerance.replacement_allocation_mode 'invalid_spot'. Expected 'spot' or 'on-demand'.
```

# 10.2

```bash
…/hpcac-toolkit on  tcc [!?] via 🦀 v1.95.0 on ☁️  (us-east-1) took 24s 
➜ cargo run -- cluster run-task --cluster-id ClusterMANA --file tasks-no-ft.yaml -y     

warning: associated function `fetch_all_by_cluster_id` is never used
  --> src/database/models/interruption_event.rs:63:18
   |
20 | impl InterruptionEvent {
   | ---------------------- associated function in this implementation
...
63 |     pub async fn fetch_all_by_cluster_id(
   |                  ^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: `fastcluster` (bin "fastcluster") generated 1 warning
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.15s
     Running `target/debug/fastcluster cluster run-task --cluster-id ClusterMANA --file tasks-no-ft.yaml -y`
Tasks:
 - name: slurm_smoke_test
   setup_commands:
     - sinfo -N
     - scontrol show nodes
   run_commands:

 - name: npb_ep_a
   setup_commands:
   run_commands:
     - srun -N2 -n4 /home/ec2-user/downloads/NPB3.4.4/NPB3.4-MPI/bin/ep.A.x

 - name: npb_cg_a
   setup_commands:
   run_commands:
     - srun -N2 -n4 /home/ec2-user/downloads/NPB3.4.4/NPB3.4-MPI/bin/cg.A.x

Automatic confirmation with -y flag. Proceeding...

Waiting for node to be ready for commands (SSM Agent)...

[00:00:21] ########################################       4/4       All tasks completed!                                                  All logs and results were saved at 'results/cluster_ClusterMANA/2026-05-09T20:19:58.txt'
```

# 10.3

```bash
…/hpcac-toolkit on  tcc [!?] via 🦀 v1.95.0 on ☁️  (us-east-1) 
✗ cargo run -- cluster run-task --cluster-id ClusterMANA --file tasks-ft-replace.yaml -y      

warning: associated function `fetch_all_by_cluster_id` is never used
  --> src/database/models/interruption_event.rs:63:18
   |
20 | impl InterruptionEvent {
   | ---------------------- associated function in this implementation
...
63 |     pub async fn fetch_all_by_cluster_id(
   |                  ^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: `fastcluster` (bin "fastcluster") generated 1 warning
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.13s
     Running `target/debug/fastcluster cluster run-task --cluster-id ClusterMANA --file tasks-ft-replace.yaml -y`
Fault Tolerance:
 - strategy: REPLACE_RESUME
 - replacement_allocation_mode: spot
 - process_count: 4
 - checkpoint_dir: /shared/checkpoints
 - poll_interval_secs: 10
 - terminate_cluster_on_zero_workers: false
Tasks:
 - name: slurm_smoke_test
   setup_commands:
     - sinfo -N
     - scontrol show nodes
   run_commands:

 - name: npb_ep_a
   setup_commands:
   run_commands:
     - srun -N2 -n4 /home/ec2-user/downloads/NPB3.4.4/NPB3.4-MPI/bin/ep.A.x

 - name: npb_cg_a
   setup_commands:
   run_commands:
     - srun -N2 -n4 /home/ec2-user/downloads/NPB3.4.4/NPB3.4-MPI/bin/cg.A.x

Automatic confirmation with -y flag. Proceeding...

Fault tolerance worker mapping (private_ip -> node_index):
 - 10.0.0.11 -> 1
 - 10.0.0.12 -> 2
Waiting for node to be ready for commands (SSM Agent)...

[00:00:00] ----------------------------------------       0/4       Executing command: 'sinfo -N'                                         [watcher] Started (strategy=REPLACE_RESUME, workers=2, poll=10s)
[00:00:21] ########################################       4/4       All tasks completed!                                                  Tasks complete. Stopping watcher...
All logs and results were saved at 'results/cluster_ClusterMANA/2026-05-09T20:17:08.txt'
```

# 10.4

```bash
…/hpcac-toolkit on  tcc [!?] via 🦀 v1.95.0 on ☁️  (us-east-1) 
➜ aws ec2 describe-spot-instance-requests \
    --filters "Name=instance-id,Values=i-0efcadf2bd0ebd0a7" \
    --query 'SpotInstanceRequests[0].Status.Code' \
    --output text \
   --no-cli-pager
fulfilled

…/hpcac-toolkit on  tcc [!?] via 🦀 v1.95.0 on ☁️  (us-east-1) 
➜ aws ec2 describe-spot-instance-requests \
    --filters "Name=instance-id,Values=                      
i-0fe9d691f55a38ccc7" \
    --query 'SpotInstanceRequests[0].Status.Code' \
    --output text \
   --no-cli-pager

…/hpcac-toolkit on  tcc [!?] via 🦀 v1.95.0 on ☁️  (us-east-1) 
➜ aws ec2 describe-spot-instance-requests \
    --filters "Name=instance-id,Values=i-0fe9d691f55a38ccc" \
    --query 'SpotInstanceRequests[0].Status.Code' \
    --output text \
   --no-cli-pager
None
```
