use crate::database::models::{Cluster, InterruptionEvent, Node};
use crate::integrations::providers::aws::{AwsClusterContext, AwsInterface};
use crate::integrations::CloudResourceManager;
use crate::utils::random::generate_id;

use anyhow::{bail, Result};
use indicatif::ProgressBar;
use sqlx::sqlite::SqlitePool;
use std::collections::{HashMap, HashSet, VecDeque};
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

#[derive(Debug, Clone)]
pub struct FaultToleranceConfig {
    pub strategy: String,
    pub replacement_allocation_mode: String,
    pub process_count: u32,
    pub checkpoint_dir: String,
    pub poll_interval_secs: u64,
    pub terminate_cluster_on_zero_workers: bool,
}

fn watcher_info(progress: &Option<ProgressBar>, message: &str) {
    info!("{}", message);
    if let Some(pb) = progress {
        pb.println(message.to_string());
        pb.set_message(message.to_string());
    } else {
        println!("{}", message);
    }
}

fn watcher_warn(progress: &Option<ProgressBar>, message: &str) {
    warn!("{}", message);
    if let Some(pb) = progress {
        pb.println(message.to_string());
        pb.set_message(message.to_string());
    } else {
        eprintln!("{}", message);
    }
}

pub async fn run_watcher(
    pool: SqlitePool,
    cluster: Cluster,
    all_nodes: Vec<Node>,
    worker_nodes: Vec<(usize, Node)>,
    mut worker_instance_ids: HashMap<usize, String>,
    head_instance_id: String,
    context: AwsClusterContext,
    cloud_interface: AwsInterface,
    ft: FaultToleranceConfig,
    progress: Option<ProgressBar>,
) -> Result<()> {
    let mut in_flight: HashSet<usize> = HashSet::new();
    let mut queued: HashSet<usize> = HashSet::new();
    let mut pending: VecDeque<usize> = VecDeque::new();
    let mut missing_running_instance_counts: HashMap<usize, u8> = HashMap::new();
    let mut consumed_warning_event_ids: HashSet<String> = HashSet::new();
    let watcher_started_at = chrono::Utc::now().to_rfc3339();

    watcher_info(
        &progress,
        &format!(
        "[watcher] Started (strategy={}, workers={}, poll={}s)",
        ft.strategy,
        worker_nodes.len(),
        ft.poll_interval_secs
    ),
    );

    enqueue_detected_interruptions(
        &pool,
        &cluster.id,
        &watcher_started_at,
        &worker_nodes,
        &mut worker_instance_ids,
        &cloud_interface,
        &context,
        &in_flight,
        &mut queued,
        &mut pending,
        &mut missing_running_instance_counts,
        &mut consumed_warning_event_ids,
    )
    .await;

    loop {
        enqueue_detected_interruptions(
            &pool,
            &cluster.id,
            &watcher_started_at,
            &worker_nodes,
            &mut worker_instance_ids,
            &cloud_interface,
            &context,
            &in_flight,
            &mut queued,
            &mut pending,
            &mut missing_running_instance_counts,
            &mut consumed_warning_event_ids,
        )
        .await;

        if let Some(node_index) = pending.pop_front() {
            queued.remove(&node_index);
            if in_flight.contains(&node_index) {
                sleep(Duration::from_secs(ft.poll_interval_secs)).await;
                continue;
            }

            if let Some((_, worker)) = worker_nodes.iter().find(|(idx, _)| *idx == node_index) {
                let private_ip = worker.private_ip.as_deref().unwrap_or("?");
                let recovery_cycle_id = generate_id();
                watcher_info(
                    &progress,
                    &format!(
                    "[watcher][cycle={}] Interruption detected - worker index={} private_ip={}",
                    recovery_cycle_id, node_index, private_ip
                ),
                );

                in_flight.insert(node_index);

                if let Err(e) = handle_interruption(
                    &pool,
                    &cluster,
                    worker,
                    node_index,
                    &all_nodes,
                    &cloud_interface,
                    &context,
                    &head_instance_id,
                    &ft,
                    &progress,
                    &recovery_cycle_id,
                )
                .await
                {
                    if let Err(insert_err) = InterruptionEvent::new(
                        &cluster.id,
                        &worker.id,
                        private_ip,
                        "recovery_failed",
                        Some(&format!("cycle_id={} | {}", recovery_cycle_id, e)),
                    )
                    .insert(&pool)
                    .await
                    {
                        warn!(
                            "[watcher] failed to persist recovery_failed event for worker index {}: {}",
                            node_index, insert_err
                        );
                    }

                    error!(
                        "[watcher][cycle={}] recovery failed for worker index {} (private_ip={}): {}",
                        recovery_cycle_id, node_index, private_ip, e
                    );
                    watcher_warn(
                        &progress,
                        &format!(
                            "[watcher][cycle={}] Recovery attempt failed; continuing to monitor for next interruption/recovery cycle.",
                            recovery_cycle_id
                        ),
                    );
                    in_flight.remove(&node_index);
                    // Keep watcher alive so subsequent failures/recoveries in the same run can still be handled.
                    continue;
                }

                in_flight.remove(&node_index);
            }
        }

        sleep(Duration::from_secs(ft.poll_interval_secs)).await;
    }
}

async fn enqueue_detected_interruptions(
    pool: &SqlitePool,
    cluster_id: &str,
    watcher_started_at: &str,
    worker_nodes: &[(usize, Node)],
    worker_instance_ids: &mut HashMap<usize, String>,
    cloud_interface: &AwsInterface,
    context: &AwsClusterContext,
    in_flight: &HashSet<usize>,
    queued: &mut HashSet<usize>,
    pending: &mut VecDeque<usize>,
    missing_running_instance_counts: &mut HashMap<usize, u8>,
    consumed_warning_event_ids: &mut HashSet<String>,
) {
    enqueue_simulated_warning_events(
        pool,
        cluster_id,
        watcher_started_at,
        worker_nodes,
        in_flight,
        queued,
        pending,
        consumed_warning_event_ids,
    )
    .await;

    for (node_index, node) in worker_nodes {
        let private_ip = match node.private_ip.as_deref() {
            Some(ip) => ip,
            None => {
                warn!(
                    "[watcher] worker index {} has no private_ip; skipping interruption probe",
                    node_index
                );
                continue;
            }
        };

        if has_active_recovery_for_node(pool, cluster_id, watcher_started_at, private_ip).await {
            continue;
        }

        if in_flight.contains(node_index) || queued.contains(node_index) {
            continue;
        }

        let current_instance_id = match resolve_running_instance_id_for_worker(
            context,
            cluster_id,
            private_ip,
        )
        .await
        {
            Ok(Some(id)) => {
                missing_running_instance_counts.remove(node_index);
                worker_instance_ids.insert(*node_index, id.clone());
                id
            }
            Ok(None) => {
                worker_instance_ids.remove(node_index);
                let misses = missing_running_instance_counts.entry(*node_index).or_insert(0);
                *misses = misses.saturating_add(1);
                if *misses >= 3 {
                    warn!(
                        "[watcher] worker index {} has no running instance for {} consecutive polls; queuing recovery",
                        node_index, misses
                    );
                    queued.insert(*node_index);
                    pending.push_back(*node_index);
                    *misses = 0;
                }
                continue;
            }
            Err(e) => {
                warn!(
                    "[watcher] failed to resolve running instance id for worker index {} (private_ip={}): {}",
                    node_index, private_ip, e
                );
                continue;
            }
        };

        let status = match cloud_interface
            .fetch_spot_instance_status(context, &current_instance_id)
            .await
        {
            Ok(status) => status,
            Err(e) => {
                warn!(
                    "[watcher] failed to fetch spot status for instance {} (worker index {}): {}",
                    current_instance_id, node_index, e
                );
                "unknown".to_string()
            }
        };

        if is_interruption_status(&status) {
            queued.insert(*node_index);
            pending.push_back(*node_index);
        }
    }
}

async fn has_active_recovery_for_node(
    pool: &SqlitePool,
    cluster_id: &str,
    watcher_started_at: &str,
    node_private_ip: &str,
) -> bool {
    let started = match sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(1)
        FROM interruption_events
        WHERE cluster_id = ?
          AND occurred_at >= ?
          AND node_private_ip = ?
          AND event_type = 'recovery_started'
        "#,
    )
    .bind(cluster_id)
    .bind(watcher_started_at)
    .bind(node_private_ip)
    .fetch_one(pool)
    .await
    {
        Ok(v) => v,
        Err(e) => {
            warn!(
                "[watcher] failed to query active recovery starts for node {}: {}",
                node_private_ip, e
            );
            return false;
        }
    };

    let terminal = match sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(1)
        FROM interruption_events
        WHERE cluster_id = ?
          AND occurred_at >= ?
          AND node_private_ip = ?
          AND event_type IN ('recovery_completed', 'degraded_resume', 'no_workers_remaining', 'recovery_failed')
        "#,
    )
    .bind(cluster_id)
    .bind(watcher_started_at)
    .bind(node_private_ip)
    .fetch_one(pool)
    .await
    {
        Ok(v) => v,
        Err(e) => {
            warn!(
                "[watcher] failed to query active recovery terminal events for node {}: {}",
                node_private_ip, e
            );
            return false;
        }
    };

    started > terminal
}

async fn enqueue_simulated_warning_events(
    pool: &SqlitePool,
    cluster_id: &str,
    watcher_started_at: &str,
    worker_nodes: &[(usize, Node)],
    in_flight: &HashSet<usize>,
    queued: &mut HashSet<usize>,
    pending: &mut VecDeque<usize>,
    consumed_warning_event_ids: &mut HashSet<String>,
) {
    let rows = match sqlx::query!(
        r#"
        SELECT id as "id!", node_private_ip as "node_private_ip!"
        FROM interruption_events
        WHERE cluster_id = ?
          AND event_type = 'interruption_warning'
                    AND occurred_at >= ?
        ORDER BY occurred_at DESC
        LIMIT 64
        "#,
        cluster_id,
                watcher_started_at,
    )
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            warn!(
                "[watcher] failed to query interruption_warning events for cluster '{}': {}",
                cluster_id, e
            );
            return;
        }
    };

    for row in rows {
        if consumed_warning_event_ids.contains(&row.id) {
            continue;
        }

        if let Some((node_index, _)) = worker_nodes
            .iter()
            .find(|(_, node)| node.private_ip.as_deref() == Some(row.node_private_ip.as_str()))
        {
            if has_active_recovery_for_node(pool, cluster_id, watcher_started_at, row.node_private_ip.as_str()).await {
                consumed_warning_event_ids.insert(row.id);
                continue;
            }

            if in_flight.contains(node_index) || queued.contains(node_index) {
                continue;
            }

            queued.insert(*node_index);
            pending.push_back(*node_index);
            consumed_warning_event_ids.insert(row.id);
        }
    }
}

async fn resolve_running_instance_id_for_worker(
    context: &AwsClusterContext,
    cluster_id: &str,
    private_ip: &str,
) -> Result<Option<String>> {
    let ip_filter = aws_sdk_ec2::types::Filter::builder()
        .name("private-ip-address")
        .values(private_ip)
        .build();
    let running_filter = aws_sdk_ec2::types::Filter::builder()
        .name("instance-state-name")
        .values("running")
        .build();
    let cluster_filter = aws_sdk_ec2::types::Filter::builder()
        .name("tag:ClusterId")
        .values(cluster_id)
        .build();

    let resp = context
        .ec2_client
        .describe_instances()
        .filters(ip_filter)
        .filters(running_filter)
        .filters(cluster_filter)
        .send()
        .await?;

    Ok(resp
        .reservations()
        .iter()
        .flat_map(|r| r.instances())
        .find_map(|i| i.instance_id().map(|id| id.to_string())))
}

fn is_interruption_status(status: &str) -> bool {
    matches!(
        status,
        "marked-for-termination" | 
        "instance-terminated-by-user" |
        "instance-terminated-no-capacity" |
        "instance-terminated-by-price" |
        "instance-terminated-capacity-oversubscribed" |
        "closed"
    )
}

async fn handle_interruption(
    pool: &SqlitePool,
    cluster: &Cluster,
    node: &Node,
    node_index: usize,
    all_nodes: &[Node],
    cloud_interface: &AwsInterface,
    context: &AwsClusterContext,
    head_instance_id: &str,
    ft: &FaultToleranceConfig,
    progress: &Option<ProgressBar>,
    recovery_cycle_id: &str,
) -> Result<()> {
    let node_private_ip = node.private_ip.as_deref().unwrap_or("");
    let worker_host = format!("ip-{}", node_private_ip.replace('.', "-"));

    InterruptionEvent::new(
        &cluster.id,
        &node.id,
        node_private_ip,
        "interruption_detected",
        Some(&format!("cycle_id={}", recovery_cycle_id)),
    )
    .insert(pool)
    .await?;

    InterruptionEvent::new(
        &cluster.id,
        &node.id,
        node_private_ip,
        "recovery_started",
        Some(&format!("cycle_id={}", recovery_cycle_id)),
    )
    .insert(pool)
    .await?;

    watcher_info(
        progress,
        &format!(
            "[watcher][cycle={}]   -> Checkpoint: triggering and waiting for checkpoint files",
            recovery_cycle_id
        ),
    );
    // Clear the checkpoint directory, trigger a fresh checkpoint, then poll the filesystem
    // directly for completion.  Polling files (rather than coordinator status) means we detect
    // success even when the coordinator exits with --exit-on-last right after writing the files.
    // A valid checkpoint requires one ckpt_*.dmtcp AND one header.mana per rank with no
    // *.dmtcp.temp files remaining.
    let checkpoint_cmd = format!(r#"cat > /tmp/hpcac_ckpt_wait.sh <<'CKPTWAIT'
#!/bin/bash
set -uo pipefail
CKPT_DIR="{ckpt}"
EXPECTED={procs}

ckpt_ready() {{
    local dmtcp_count header_count tmp_count
    dmtcp_count=$(find "$CKPT_DIR" -maxdepth 2 -name 'ckpt_*.dmtcp' 2>/dev/null | wc -l)
    header_count=$(find "$CKPT_DIR" -maxdepth 2 -name 'header.mana'  2>/dev/null | wc -l)
    tmp_count=$(find   "$CKPT_DIR" -maxdepth 2 -name '*.dmtcp.temp'  2>/dev/null | wc -l)
    echo "ckpt-check: dmtcp=$dmtcp_count header=$header_count tmp=$tmp_count expected=$EXPECTED"
    [ "$dmtcp_count" -eq "$EXPECTED" ] && [ "$header_count" -eq "$EXPECTED" ] && [ "$tmp_count" -eq 0 ]
}}

# Clear any stale files from previous runs.
sudo mkdir -p "$CKPT_DIR" && sudo chown ec2-user:ec2-user "$CKPT_DIR" && sudo chmod 775 "$CKPT_DIR"
find "$CKPT_DIR" -mindepth 1 -maxdepth 1 -exec rm -rf {{}} + 2>/dev/null || true
echo "Checkpoint directory cleared"

# Trigger checkpoint; exit 2 = ERROR_NOT_RUNNING_STATE means one is already in progress.
/opt/mana/bin/dmtcp_command -h 10.0.0.10 -p 7779 -c 2>&1 || true

# Poll the filesystem until all checkpoint files are present and complete.
for i in $(seq 1 60); do
    sleep 5
    if ckpt_ready; then
        echo "Checkpoint complete (waited $((i*5)) s)"
        exit 0
    fi
done
echo "ERROR: checkpoint did not complete within 300s"
exit 1
CKPTWAIT
bash /tmp/hpcac_ckpt_wait.sh"#,
        ckpt = ft.checkpoint_dir,
        procs = ft.process_count,
    );
    let checkpoint_cmd_id = cloud_interface
        .create_ssm_command(context, head_instance_id, checkpoint_cmd)
        .await?;

    match cloud_interface
        .poll_ssm_command_until_completion(
            context,
            &checkpoint_cmd_id,
            head_instance_id,
            Duration::from_secs(360),
            Duration::from_secs(10),
        )
        .await
    {
        Ok(_) => {
            InterruptionEvent::new(
                &cluster.id,
                &node.id,
                node_private_ip,
                "checkpoint_completed",
                Some(&format!("cycle_id={}", recovery_cycle_id)),
            )
            .insert(pool)
            .await?;
        }
        Err(e) => {
            let checkpoint_err = e.to_string();
            if let Err(insert_err) = InterruptionEvent::new(
                &cluster.id,
                &node.id,
                node_private_ip,
                "checkpoint_failed",
                Some(&format!("cycle_id={} | {}", recovery_cycle_id, checkpoint_err)),
            )
            .insert(pool)
            .await
            {
                watcher_warn(
                    progress,
                    &format!(
                        "[watcher][cycle={}] failed to persist checkpoint_failed event: {}",
                        recovery_cycle_id, insert_err
                    ),
                );
            }

            bail!(
                "[watcher][cycle={}] checkpoint command failed: {}",
                recovery_cycle_id,
                checkpoint_err
            );
        }
    }

    watcher_info(
        progress,
        &format!(
            "[watcher][cycle={}]   -> Draining '{}' in Slurm",
            recovery_cycle_id, worker_host
        ),
    );
    let drain_cmd = format!(
        "sudo /opt/slurm-24.05.4/bin/scontrol update NodeName={} State=DRAIN Reason=spot-interruption",
        worker_host
    );
    let drain_cmd_id = cloud_interface
        .create_ssm_command(context, head_instance_id, drain_cmd)
        .await?;
    cloud_interface
        .poll_ssm_command_until_completion(
            context,
            &drain_cmd_id,
            head_instance_id,
            Duration::from_secs(30),
            Duration::from_secs(5),
        )
        .await?;

    InterruptionEvent::new(
        &cluster.id,
        &node.id,
        node_private_ip,
        "node_drained",
        Some(&format!("cycle_id={}", recovery_cycle_id)),
    )
        .insert(pool)
        .await?;

    watcher_info(
        progress,
        &format!(
            "[watcher][cycle={}]   -> Killing Slurm Job",
            recovery_cycle_id
        ),
    );
    let scancel_cmd = format!(
        "scancel --user=$USER --signal=KILL"
    );
    let scancel_cmd_id = cloud_interface
        .create_ssm_command(context, head_instance_id, scancel_cmd)
        .await?;
    cloud_interface
        .poll_ssm_command_until_completion(
            context,
            &scancel_cmd_id,
            head_instance_id,
            Duration::from_secs(30),
            Duration::from_secs(5),
        )
        .await?;

    match ft.strategy.as_str() {
        "REPLACE_RESUME" => {
            handle_policy_replace_resume(
                pool,
                cluster,
                node,
                node_index,
                all_nodes,
                cloud_interface,
                context,
                head_instance_id,
                &worker_host,
                ft,
                progress,
                recovery_cycle_id,
            )
            .await
        }
        "DEGRADED_RESUME" => {
            handle_policy_degraded_resume(
                pool,
                cluster,
                node,
                all_nodes,
                cloud_interface,
                context,
                head_instance_id,
                &worker_host,
                ft,
                progress,
                recovery_cycle_id,
            )
            .await
        }
        other => bail!("Unknown fault_tolerance.strategy: {}", other),
    }
}

async fn handle_policy_replace_resume(
    pool: &SqlitePool,
    cluster: &Cluster,
    node: &Node,
    node_index: usize,
    all_nodes: &[Node],
    cloud_interface: &AwsInterface,
    context: &AwsClusterContext,
    head_instance_id: &str,
    worker_host: &str,
    ft: &FaultToleranceConfig,
    progress: &Option<ProgressBar>,
    recovery_cycle_id: &str,
) -> Result<()> {
    let node_private_ip = node.private_ip.as_deref().unwrap_or("");

    watcher_info(
        progress,
        &format!(
            "[watcher][cycle={}]   -> Waiting for worker EC2 instance to terminate (freeing network interface before respawn)",
            recovery_cycle_id
        ),
    );
    let pre_recovery_instance_id = resolve_running_instance_id_for_worker(
        context,
        &cluster.id,
        node_private_ip,
    )
    .await?;
    // Wait for the instance to fully terminate — not just leave running state.
    // Attempting to respawn while the old instance is still in "shutting-down" will fail because
    // AWS has not released the static private IP / network interface yet.  We nudge the
    // termination every 60 s in case the instance is stuck in the shutting-down phase.
    wait_for_instance_to_terminate(
        context,
        node_index,
        pre_recovery_instance_id.as_deref(),
        Duration::from_secs(600),
        progress,
        recovery_cycle_id,
    )
    .await?;

    node.set_efs_configuration_state(pool, false).await?;

    watcher_info(
        progress,
        &format!(
            "[watcher][cycle={}]   -> Respawning worker node",
            recovery_cycle_id
        ),
    );
    respawn_worker_with_retry(
        pool,
        cluster,
        node,
        node_index,
        all_nodes,
        cloud_interface,
        ft,
        progress,
        recovery_cycle_id,
        Duration::from_secs(60), // short retry window — IP is guaranteed free by now
    )
    .await?;

    watcher_info(
        progress,
        &format!(
            "[watcher][cycle={}]   -> Resuming '{}' in Slurm",
            recovery_cycle_id, worker_host
        ),
    );
    let resume_cmd = format!(
        "sudo /opt/slurm-24.05.4/bin/scontrol update NodeName={} State=RESUME",
        worker_host
    );
    let resume_cmd_id = cloud_interface
        .create_ssm_command(context, head_instance_id, resume_cmd)
        .await?;
    cloud_interface
        .poll_ssm_command_until_completion(
            context,
            &resume_cmd_id,
            head_instance_id,
            Duration::from_secs(30),
            Duration::from_secs(5),
        )
        .await?;

    watcher_info(
        progress,
        &format!(
            "[watcher][cycle={}]   -> Waiting for Slurm node '{}' to be IDLE",
            recovery_cycle_id, worker_host
        ),
    );
    let wait_cmd = format!(
                r#"for i in $(seq 1 48); do
    STATE=$(sudo /opt/slurm-24.05.4/bin/sinfo -N -n {host} --noheader | awk '{{print $4}}' | tr '[:upper:]' '[:lower:]')
    echo "Attempt $i: $STATE"
    case "$STATE" in
        idle*|mix*|alloc*)
            exit 0
            ;;
        down*|drain*|drng*|fail*|unk*|maint*)
            echo "Node entered non-schedulable state: $STATE"
            exit 2
            ;;
    esac
    sleep 10
done
exit 1"#,
        host = worker_host
    );
    let wait_cmd_id = cloud_interface
        .create_ssm_command(context, head_instance_id, wait_cmd)
        .await?;
    cloud_interface
        .poll_ssm_command_until_completion(
            context,
            &wait_cmd_id,
            head_instance_id,
            Duration::from_secs(600),
            Duration::from_secs(15),
        )
        .await?;

    if let Err(e) = InterruptionEvent::new(
        &cluster.id,
        &node.id,
        node_private_ip,
        "node_became_idle",
        Some(&format!("cycle_id={}", recovery_cycle_id)),
    )
    .insert(pool)
    .await
    {
        watcher_warn(
            progress,
            &format!(
                "[watcher][cycle={}] failed to persist node_became_idle event: {}",
                recovery_cycle_id, e
            ),
        );
    }

    let worker_hosts: Vec<String> = all_nodes
        .iter()
        .filter(|n| n.role == "worker")
        .filter_map(|n| n.private_ip.as_deref())
        .map(|ip| format!("ip-{}", ip.replace('.', "-")))
        .collect();
    let worker_count = worker_hosts.len();

    watcher_info(
        progress,
        &format!(
            "[watcher][cycle={}]   -> Running restart command",
            recovery_cycle_id
        ),
    );
    let restart_process_count = ft.process_count;
    let restart_cmd = build_restart_command(ft, &worker_hosts, restart_process_count, false);
    let restart_cmd_id = cloud_interface
        .create_ssm_command(context, head_instance_id, restart_cmd)
        .await?;

    if let Err(e) = InterruptionEvent::new(
        &cluster.id,
        &node.id,
        node_private_ip,
        "restart_dispatched",
        Some(&format!(
            "cycle_id={} | {} workers, {} processes",
            recovery_cycle_id, worker_count, restart_process_count
        )),
    )
    .insert(pool)
    .await
    {
        watcher_warn(
            progress,
            &format!(
                "[watcher][cycle={}] failed to persist restart_dispatched event: {}",
                recovery_cycle_id, e
            ),
        );
    }

    watcher_info(
        progress,
        &format!(
            "[watcher][cycle={}]   -> Restart dispatched; monitoring completion in background",
            recovery_cycle_id
        ),
    );
    spawn_restart_monitor(
        pool.clone(),
        cluster.clone(),
        node.clone(),
        node_private_ip.to_string(),
        context.clone(),
        cloud_interface.clone(),
        head_instance_id.to_string(),
        restart_cmd_id,
        recovery_cycle_id.to_string(),
        progress.clone(),
        "recovery_completed".to_string(),
        format!(
            "cycle_id={} | {} workers, {} processes",
            recovery_cycle_id, worker_count, restart_process_count
        ),
        format!(
            "[watcher][cycle={}]   -> [REPLACE_RESUME] Recovery complete. Restart finished.",
            recovery_cycle_id
        ),
    );
    Ok(())
}

async fn handle_policy_degraded_resume(
    pool: &SqlitePool,
    cluster: &Cluster,
    node: &Node,
    all_nodes: &[Node],
    cloud_interface: &AwsInterface,
    context: &AwsClusterContext,
    head_instance_id: &str,
    worker_host: &str,
    ft: &FaultToleranceConfig,
    progress: &Option<ProgressBar>,
    recovery_cycle_id: &str,
) -> Result<()> {
    let node_private_ip = node.private_ip.as_deref().unwrap_or("");
    let total_workers = all_nodes.iter().filter(|n| n.role == "worker").count();
    let remaining_workers = total_workers.saturating_sub(1);

    if remaining_workers == 0 {
        watcher_info(
            progress,
            &format!(
                "[watcher][cycle={}]   -> [DEGRADED_RESUME] No workers left. Stopping jobs.",
                recovery_cycle_id
            ),
        );
        let stop_cmd = "DMTCP_COORD_HOST=10.0.0.10 DMTCP_COORD_PORT=7779 /opt/mana/bin/dmtcp_command --quit || true && sudo /opt/slurm-24.05.4/bin/scancel --state=RUNNING --name='*' || true".to_string();
        let stop_cmd_id = cloud_interface
            .create_ssm_command(context, head_instance_id, stop_cmd)
            .await?;
        cloud_interface
            .poll_ssm_command_until_completion(
                context,
                &stop_cmd_id,
                head_instance_id,
                Duration::from_secs(60),
                Duration::from_secs(5),
            )
            .await?;

        InterruptionEvent::new(
            &cluster.id,
            &node.id,
            node_private_ip,
            "no_workers_remaining",
            Some(&format!(
                "cycle_id={} | All spot workers interrupted; degraded resume not possible",
                recovery_cycle_id
            )),
        )
        .insert(pool)
        .await?;

        if ft.terminate_cluster_on_zero_workers {
            watcher_info(
                progress,
                &format!(
                    "[watcher][cycle={}]   -> Terminating cluster because no workers remain",
                    recovery_cycle_id
                ),
            );
            cloud_interface
                .terminate_cluster(pool, cluster.clone(), all_nodes.to_vec())
                .await?;
        }

        return Ok(());
    }

    let down_cmd = format!(
        "sudo /opt/slurm-24.05.4/bin/scontrol update NodeName={} State=DOWN Reason=spot-interrupted-no-replace",
        worker_host
    );
    let down_cmd_id = cloud_interface
        .create_ssm_command(context, head_instance_id, down_cmd)
        .await?;
    cloud_interface
        .poll_ssm_command_until_completion(
            context,
            &down_cmd_id,
            head_instance_id,
            Duration::from_secs(30),
            Duration::from_secs(5),
        )
        .await?;

    if let Err(e) = InterruptionEvent::new(
        &cluster.id,
        &node.id,
        node_private_ip,
        "node_became_idle",
        Some(&format!("cycle_id={} | node marked DOWN for degraded resume", recovery_cycle_id)),
    )
    .insert(pool)
    .await
    {
        watcher_warn(
            progress,
            &format!(
                "[watcher][cycle={}] failed to persist node_became_idle event: {}",
                recovery_cycle_id, e
            ),
        );
    }

    let worker_hosts: Vec<String> = all_nodes
        .iter()
        .filter(|n| n.role == "worker" && n.id != node.id)
        .filter_map(|n| n.private_ip.as_deref())
        .map(|ip| format!("ip-{}", ip.replace('.', "-")))
        .collect();
    let restart_process_count = ft.process_count;

    watcher_info(
        progress,
        &format!(
            "[watcher][cycle={}]   -> Running restart command",
            recovery_cycle_id
        ),
    );
    let restart_cmd = build_restart_command(ft, &worker_hosts, restart_process_count, true);
    let restart_cmd_id = cloud_interface
        .create_ssm_command(context, head_instance_id, restart_cmd)
        .await?;

    if let Err(e) = InterruptionEvent::new(
        &cluster.id,
        &node.id,
        node_private_ip,
        "restart_dispatched",
        Some(&format!(
            "cycle_id={} | {} remaining workers, {} processes (oversubscribe=true)",
            recovery_cycle_id, remaining_workers, restart_process_count
        )),
    )
    .insert(pool)
    .await
    {
        watcher_warn(
            progress,
            &format!(
                "[watcher][cycle={}] failed to persist restart_dispatched event: {}",
                recovery_cycle_id, e
            ),
        );
    }

    watcher_info(
        progress,
        &format!(
            "[watcher][cycle={}]   -> Restart dispatched; monitoring completion in background",
            recovery_cycle_id
        ),
    );
    spawn_restart_monitor(
        pool.clone(),
        cluster.clone(),
        node.clone(),
        node_private_ip.to_string(),
        context.clone(),
        cloud_interface.clone(),
        head_instance_id.to_string(),
        restart_cmd_id,
        recovery_cycle_id.to_string(),
        progress.clone(),
        "degraded_resume".to_string(),
        format!(
            "cycle_id={} | {} remaining workers, {} processes (oversubscribe=true)",
            recovery_cycle_id, remaining_workers, restart_process_count
        ),
        format!(
            "[watcher][cycle={}]   -> [DEGRADED_RESUME] Resume completed with {} worker(s)",
            recovery_cycle_id, remaining_workers
        ),
    );
    Ok(())
}

fn spawn_restart_monitor(
    pool: SqlitePool,
    cluster: Cluster,
    node: Node,
    node_private_ip: String,
    context: AwsClusterContext,
    cloud_interface: AwsInterface,
    head_instance_id: String,
    restart_cmd_id: String,
    recovery_cycle_id: String,
    progress: Option<ProgressBar>,
    terminal_event_type: String,
    terminal_details: String,
    completion_log_message: String,
) {
    tokio::spawn(async move {
        watcher_info(
            &progress,
            &format!(
                "[watcher][cycle={}]   -> Waiting for restart command completion",
                recovery_cycle_id
            ),
        );

        match cloud_interface
            .poll_ssm_command_until_completion(
                &context,
                &restart_cmd_id,
                &head_instance_id,
                Duration::from_secs(3600),
                Duration::from_secs(5),
            )
            .await
        {
            Ok(output) => {
                let restart_output = {
                    let trimmed = output.trim();
                    if trimmed.is_empty() {
                        "(empty)".to_string()
                    } else {
                        trimmed.to_string()
                    }
                };

                if let Err(e) = InterruptionEvent::new(
                    &cluster.id,
                    &node.id,
                    &node_private_ip,
                    "restart_completed",
                    Some(&format!(
                        "cycle_id={}\n--restart_output_begin--\n{}\n--restart_output_end--",
                        recovery_cycle_id, restart_output
                    )),
                )
                .insert(&pool)
                .await
                {
                    watcher_warn(
                        &progress,
                        &format!(
                            "[watcher][cycle={}] failed to persist restart_completed event: {}",
                            recovery_cycle_id, e
                        ),
                    );
                }

                if let Err(e) = InterruptionEvent::new(
                    &cluster.id,
                    &node.id,
                    &node_private_ip,
                    &terminal_event_type,
                    Some(&terminal_details),
                )
                .insert(&pool)
                .await
                {
                    watcher_warn(
                        &progress,
                        &format!(
                            "[watcher][cycle={}] failed to persist terminal recovery event '{}': {}",
                            recovery_cycle_id, terminal_event_type, e
                        ),
                    );
                    return;
                }

                watcher_info(&progress, &completion_log_message);
            }
            Err(e) => {
                let err_text = e.to_string();
                if let Err(insert_err) = InterruptionEvent::new(
                    &cluster.id,
                    &node.id,
                    &node_private_ip,
                    "recovery_failed",
                    Some(&format!("cycle_id={} | {}", recovery_cycle_id, err_text)),
                )
                .insert(&pool)
                .await
                {
                    watcher_warn(
                        &progress,
                        &format!(
                            "[watcher][cycle={}] failed to persist recovery_failed event: {}",
                            recovery_cycle_id, insert_err
                        ),
                    );
                }

                watcher_warn(
                    &progress,
                    &format!(
                        "[watcher][cycle={}] restart monitor observed failure: {}",
                        recovery_cycle_id, err_text
                    ),
                );
            }
        }
    });
}

fn build_restart_command(
    ft: &FaultToleranceConfig,
    worker_hosts: &[String],
    process_count: u32,
    oversubscribe: bool,
) -> String {
    let nodelist = worker_hosts.join(",");
    let worker_count = worker_hosts.len();
    let oversubscribe_flag = if oversubscribe { " --oversubscribe" } else { "" };
    let overcommit_flag = if oversubscribe { " --overcommit" } else { "" };
    if worker_count == 0 {
        return "echo 'No worker hosts available for restart'; exit 1".to_string();
    }
    let ntasks_per_node = (process_count + worker_count as u32 - 1) / worker_count as u32;
    format!(
        r#"cat > /tmp/hpcac_restart.sh <<'INNER'
#!/bin/bash
set -euo pipefail
# No sudo needed: coordinator runs as ec2-user. Best-effort safety net only.
pkill -9 -f dmtcp_coordinator 2>/dev/null || true
pkill -9 -f mana_coordinator 2>/dev/null || true
# Use SSH instead of srun to clean up worker MPI processes so that Slurm's
# step-cleanup on the worker node cannot kill the cleanup commands.
for NODE in $(echo "{nodelist}" | tr ',' ' '); do
    ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o ConnectTimeout=5 "$NODE" \
        "pkill -9 -f lower-half 2>/dev/null; pkill -9 -f mana_launch 2>/dev/null; pkill -9 -f dmtcp_launch 2>/dev/null; pkill -9 -f dmtcp_worker 2>/dev/null" || true
done
rm -f "$HOME/.mana-slurm-${{SLURM_JOB_ID}}.rc"
export DMTCP_COORD_HOST=10.0.0.10
export DMTCP_COORD_PORT=7779
echo ${{SLURM_JOB_ID}} > ~/.tmp_job_id
/opt/mana/bin/mana_coordinator --port 7779 --ckptdir {ckpt} --exit-on-last --coord-logfile /tmp/mana_coord.log &
RC="/shared/slurm/.mana-slurm-${{SLURM_JOB_ID}}.rc"
DEST=".mana-slurm-${{SLURM_JOB_ID}}.rc"

# Wait for coordinator RC file generation so restart does not race on a missing file.
for i in $(seq 1 30); do
    if [ -s "$HOME/$DEST" ]; then
        break
    fi
    sleep 1
done
if [ ! -s "$HOME/$DEST" ]; then
    echo "ERROR: coordinator RC file '$HOME/$DEST' was not generated"
    echo "--- /tmp/mana_coord.log ---"
    cat /tmp/mana_coord.log 2>/dev/null || echo "(no coordinator log)"
    exit 41
fi
echo "Coordinator RC file ready: $HOME/$DEST"

# Ensure checkpoint directory has checkpoint payload before trying to restart.
if ! ls {ckpt}/ckpt_rank_* >/dev/null 2>&1; then
    echo "ERROR: checkpoint payload missing under {ckpt}"
    ls -la {ckpt} || true
    exit 42
fi
echo "Checkpoint payload verified under {ckpt}"

for NODE in $(echo "{nodelist}" | tr ',' ' '); do
    scp -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o ConnectTimeout=5 "$HOME/$DEST" "$NODE:$HOME/$DEST" || \
        {{ echo "ERROR: could not distribute RC file to $NODE"; exit 45; }}
done
RESTART_LOG_DIR="{ckpt}/restart_logs"
mkdir -p "$RESTART_LOG_DIR"
echo "--- Starting mana_restart (mpi=pmi2) ---"
srun --overlap --mpi=pmi2{oversubscribe}{overcommit} --nodelist={nodelist} -N{nodes} -n{procs} --ntasks-per-node={ntasks_per_node} \
    --output="$RESTART_LOG_DIR/task_%t.log" --error="$RESTART_LOG_DIR/task_%t.err" \
    bash -c 'cd {ckpt} && /opt/mana/bin/mana_restart --verbose --ckptdir {ckpt} --restartdir {ckpt}'
RESTART_RC=$?
echo "--- mana_restart exit code: $RESTART_RC ---"
echo "--- Restart output (rank 0) ---"
cat "$RESTART_LOG_DIR/task_0.log" 2>/dev/null || echo "(no rank 0 output)"
echo "--- Restart stderr (rank 0) ---"
cat "$RESTART_LOG_DIR/task_0.err" 2>/dev/null || echo "(no rank 0 stderr)"
if [ $RESTART_RC -ne 0 ]; then
    echo "--- Coordinator log ---"
    cat /tmp/mana_coord.log 2>/dev/null || echo "(no coordinator log)"
    echo "--- Task logs (tasks 1-3) ---"
    for t in 1 2 3; do
        if [ -f "$RESTART_LOG_DIR/task_$t.log" ]; then
            echo "=== task $t stdout ==="
            cat "$RESTART_LOG_DIR/task_$t.log"
        fi
        if [ -f "$RESTART_LOG_DIR/task_$t.err" ]; then
            echo "=== task $t stderr ==="
            cat "$RESTART_LOG_DIR/task_$t.err"
        fi
    done
fi
exit $RESTART_RC
INNER
chmod +x /tmp/hpcac_restart.sh
set -o pipefail
# Cancel previous workload allocation first (if known).
PREV_JOB_ID=$(cat "$HOME/.tmp_job_id" 2>/dev/null || true)
case "$PREV_JOB_ID" in
    ''|*[!0-9]*)
        ;;
    *)
        echo "Cancelling previous workload Slurm allocation: $PREV_JOB_ID"
        scancel "$PREV_JOB_ID" || true
        ;;
esac

# Fallback: cancel any remaining user jobs that could block or revoke restart allocation.
EXISTING_JOBS=$(squeue -h -u "$USER" -t PD,R,CF,CG,ST,S -o '%A' | tr '\n' ' ' | xargs)
if [ -n "$EXISTING_JOBS" ]; then
    echo "Cancelling remaining Slurm jobs before restart allocation: $EXISTING_JOBS"
    scancel $EXISTING_JOBS || true
fi

# Slurm cancellation is asynchronous; wait until user queue is fully drained.
for i in $(seq 1 180); do
    ACTIVE=$(squeue -h -u "$USER" -t PD,R,CF,CG,ST,S -o '%A' | wc -l)
    if [ "$ACTIVE" -eq 0 ]; then
        break
    fi
    echo "Waiting for Slurm queue drain after scancel (attempt $i, active=$ACTIVE)"
    if [ "$i" -eq 180 ]; then
        echo "ERROR: Slurm queue did not drain after cancellation; aborting restart allocation"
        squeue -u "$USER" || true
        exit 44
    fi
    sleep 1
done
# Allow Slurm to complete epilog/cgroup teardown of the previous job before submitting a new allocation.
sleep 5

# Ensure target nodes are schedulable before requesting a new allocation.
# If target nodes are temporarily drained/down, try to actively RESUME them first.
for i in $(seq 1 120); do
    BLOCKING=0
    for NODE in $(echo "{nodelist}" | tr ',' ' '); do
        STATE=$(sudo /opt/slurm-24.05.4/bin/sinfo -N -n "$NODE" --noheader | awk '{{print tolower($4)}}')
        case "$STATE" in
            idle*|mix*|alloc*)
                ;;
            down*|drain*|drng*|fail*|unk*|maint*)
                echo "Node '$NODE' is non-schedulable for restart (state=$STATE); trying RESUME"
                sudo /opt/slurm-24.05.4/bin/scontrol update NodeName="$NODE" State=RESUME || true
                BLOCKING=1
                ;;
            *)
                BLOCKING=1
                ;;
        esac
    done

    if [ "$BLOCKING" -eq 0 ]; then
        break
    fi

    echo "Waiting for target nodes to become schedulable before restart allocation (attempt $i)"
    if [ "$i" -eq 120 ]; then
        echo "ERROR: timed out waiting for target nodes to become schedulable"
        sudo /opt/slurm-24.05.4/bin/sinfo -N -n {nodelist} -la || true
        exit 46
    fi
    sleep 2
done
# Kill the previous MANA/DMTCP coordinator here, outside any Slurm cgroup, so that
# Slurm's cgroup cleanup of the prior allocation cannot kill the pkill commands mid-flight.
# The coordinator inside the salloc below will start fresh on a free port 7779.
pkill -9 -f dmtcp_coordinator 2>/dev/null || true
pkill -9 -f mana_coordinator 2>/dev/null || true
for i in $(seq 1 30); do
    ALIVE=0
    pgrep -f dmtcp_coordinator >/dev/null 2>&1 && ALIVE=1 || true
    pgrep -f mana_coordinator >/dev/null 2>&1 && ALIVE=1 || true
    if [ "$ALIVE" -eq 0 ]; then break; fi
    pkill -9 -f dmtcp_coordinator 2>/dev/null || true
    pkill -9 -f mana_coordinator 2>/dev/null || true
    sleep 1
done
# Wait for DMTCP coordinator port 7779 to be free so the new coordinator can bind it.
for i in $(seq 1 15); do
    if ! ss -lntp 2>/dev/null | grep -q ':7779 '; then break; fi
    echo "Waiting for DMTCP coordinator port 7779 to be released (attempt $i)"
    sleep 1
done
salloc{oversubscribe}{overcommit} -w {nodelist} -N{nodes} -n{procs} --ntasks-per-node={ntasks_per_node} -t 00:30:00 bash /tmp/hpcac_restart.sh 2>&1 | tee restart_output.txt
exit ${{PIPESTATUS[0]}}"#,
        ckpt = ft.checkpoint_dir,
        nodelist = nodelist,
        nodes = worker_count,
        procs = process_count,
        ntasks_per_node = ntasks_per_node,
        oversubscribe = oversubscribe_flag,
        overcommit = overcommit_flag,
    )
}

fn is_respawn_retryable_error(err_text: &str) -> bool {
    let e = err_text.to_lowercase();
    e.contains("invalidnetworkinterface.inuse")
        || e.contains("network interface")
            && (e.contains("in use") || e.contains("currently attached"))
        || e.contains("incorrectinstance-state")
        || e.contains("pending-instance-creation")
        || e.contains("failure creating ec2 instance resource")
}

async fn respawn_worker_with_retry(
    pool: &SqlitePool,
    cluster: &Cluster,
    node: &Node,
    node_index: usize,
    all_nodes: &[Node],
    cloud_interface: &AwsInterface,
    ft: &FaultToleranceConfig,
    progress: &Option<ProgressBar>,
    recovery_cycle_id: &str,
    max_wait: Duration,
) -> Result<()> {
    let start = tokio::time::Instant::now();
    let mut attempt: u32 = 0;

    loop {
        attempt += 1;
        match cloud_interface
            .respawn_worker_node(
                pool,
                cluster.clone(),
                node.clone(),
                node_index,
                all_nodes.to_vec(),
                Some(ft.replacement_allocation_mode.clone()),
            )
            .await
        {
            Ok(_) => return Ok(()),
            Err(e) => {
                let err_text = e.to_string();
                let elapsed = start.elapsed();
                if is_respawn_retryable_error(&err_text) && elapsed < max_wait {
                    watcher_warn(
                        progress,
                        &format!(
                            "[watcher][cycle={}]   -> Respawn attempt {} waiting for AWS resource release (elapsed={}s): {}",
                            recovery_cycle_id,
                            attempt,
                            elapsed.as_secs(),
                            err_text
                        ),
                    );
                    sleep(Duration::from_secs(10)).await;
                    continue;
                }
                return Err(e);
            }
        }
    }
}

/// Wait until the EC2 instance reaches the `Terminated` state (or is not found).
/// Unlike `wait_for_worker_to_exit_running_state`, this does NOT accept `ShuttingDown` as
/// success — we need the instance fully gone so AWS releases the static private IP / NIC
/// before we attempt to respawn a replacement at the same address.
///
/// Every 15 s we resend a `terminate-instances` call as a nudge, in case the instance
/// gets stuck in the `shutting-down` phase (e.g. due to slow EFS unmount).
async fn wait_for_instance_to_terminate(
    context: &AwsClusterContext,
    node_index: usize,
    expected_instance_id: Option<&str>,
    timeout: Duration,
    progress: &Option<ProgressBar>,
    recovery_cycle_id: &str,
) -> Result<()> {
    let instance_name = context.ec2_instance_name(node_index);
    let start = tokio::time::Instant::now();
    let mut last_nudge_elapsed = Duration::from_secs(0);

    loop {
        let elapsed = start.elapsed();
        if elapsed >= timeout {
            bail!(
                "[watcher][cycle={}] Timeout waiting for instance (index={}, id={:?}, name='{}') to reach terminated state after {}s",
                recovery_cycle_id, node_index, expected_instance_id, instance_name, timeout.as_secs()
            );
        }

        // Send a nudge terminate call every 15 s to unstick instances that linger in shutting-down.
        if elapsed >= last_nudge_elapsed + Duration::from_secs(15) {
            last_nudge_elapsed = elapsed;
            if let Some(instance_id) = expected_instance_id {
                let _ = context
                    .ec2_client
                    .terminate_instances()
                    .instance_ids(instance_id)
                    .send()
                    .await;
                watcher_info(
                    progress,
                    &format!(
                        "[watcher][cycle={}]   -> Instance still shutting-down after {}s; sent nudge terminate ({})",
                        recovery_cycle_id,
                        elapsed.as_secs(),
                        instance_id,
                    ),
                );
            }
        }

        let mut request = context.ec2_client.describe_instances();
        if let Some(instance_id) = expected_instance_id {
            request = request.instance_ids(instance_id);
        } else {
            request = request.filters(
                aws_sdk_ec2::types::Filter::builder()
                    .name("tag:Name")
                    .values(&instance_name)
                    .build(),
            );
        }

        match request.send().await {
            Ok(resp) => {
                let instances: Vec<_> = resp
                    .reservations()
                    .iter()
                    .flat_map(|r| r.instances())
                    .collect();

                if instances.is_empty() {
                    return Ok(());
                }

                let all_terminated = instances.iter().all(|i| {
                    matches!(
                        i.state().and_then(|s| s.name()),
                        Some(&aws_sdk_ec2::types::InstanceStateName::Terminated)
                    )
                });

                if all_terminated {
                    return Ok(());
                }
            }
            Err(e) => {
                if expected_instance_id.is_some()
                    && e.to_string().contains("InvalidInstanceID.NotFound")
                {
                    return Ok(());
                }
                warn!(
                    "[watcher][cycle={}] transient error polling instance termination state: {}",
                    recovery_cycle_id, e
                );
            }
        }

        sleep(Duration::from_secs(5)).await;
    }
}

