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
        if in_flight.contains(node_index) || queued.contains(node_index) {
            continue;
        }

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

    // Always clear previous checkpoints before taking a new one so restart never reuses stale state.
    watcher_info(
        progress,
        &format!(
            "[watcher][cycle={}]   -> Clearing checkpoint directory '{}'",
            recovery_cycle_id, ft.checkpoint_dir
        ),
    );
    let clear_ckpt_cmd = format!(
        "sudo mkdir -p '{dir}' && sudo find '{dir}' -mindepth 1 -maxdepth 1 -exec rm -rf {{}} +",
        dir = ft.checkpoint_dir
    );
    let clear_ckpt_cmd_id = cloud_interface
        .create_ssm_command(context, head_instance_id, clear_ckpt_cmd)
        .await?;
    cloud_interface
        .poll_ssm_command_until_completion(
            context,
            &clear_ckpt_cmd_id,
            head_instance_id,
            Duration::from_secs(120),
            Duration::from_secs(5),
        )
        .await?;

    InterruptionEvent::new(
        &cluster.id,
        &node.id,
        node_private_ip,
        "checkpoint_dir_cleared",
        Some(&format!("cycle_id={} | dir={}", recovery_cycle_id, ft.checkpoint_dir)),
    )
    .insert(pool)
    .await?;

    watcher_info(
        progress,
        &format!(
            "[watcher][cycle={}]   -> Checkpoint: sending bcheckpoint to head",
            recovery_cycle_id
        ),
    );
    let checkpoint_cmd =
        "DMTCP_COORD_HOST=10.0.0.10 DMTCP_COORD_PORT=7779 /opt/mana/bin/dmtcp_command --bcheckpoint"
            .to_string();
    let checkpoint_cmd_id = cloud_interface
        .create_ssm_command(context, head_instance_id, checkpoint_cmd)
        .await?;

    match cloud_interface
        .poll_ssm_command_until_completion(
            context,
            &checkpoint_cmd_id,
            head_instance_id,
            Duration::from_secs(120),
            Duration::from_secs(5),
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
            watcher_warn(
                progress,
                &format!(
                "[watcher][cycle={}]   WARNING: checkpoint command failed ({}). Continuing.",
                recovery_cycle_id, e
            ),
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
            "[watcher][cycle={}]   -> Waiting for worker EC2 instance to terminate",
            recovery_cycle_id
        ),
    );
    let pre_recovery_instance_id = resolve_running_instance_id_for_worker(
        context,
        &cluster.id,
        node_private_ip,
    )
    .await?;
    wait_for_worker_termination(
        context,
        node_index,
        pre_recovery_instance_id.as_deref(),
        Duration::from_secs(180),
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
    cloud_interface
        .respawn_worker_node(
            pool,
            cluster.clone(),
            node.clone(),
            node_index,
            all_nodes.to_vec(),
            Some(ft.replacement_allocation_mode.clone()),
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

    InterruptionEvent::new(
        &cluster.id,
        &node.id,
        node_private_ip,
        "recovery_started",
        Some(&format!("cycle_id={}", recovery_cycle_id)),
    )
    .insert(pool)
    .await?;

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
    format!(
        r#"cat > /tmp/hpcac_restart.sh <<'INNER'
#!/bin/bash
sudo pkill -9 -f dmtcp_coordinator || true
sudo pkill -9 -f mana_coordinator || true
export DMTCP_COORD_HOST=10.0.0.10
export DMTCP_COORD_PORT=7779
srun --overlap --immediate=5 --nodelist={nodelist} -N{nodes} -n{nodes} --label /bin/sh -c 'pkill -9 -f lower-half || true; pkill -9 -f mana_launch || true; pkill -9 -f dmtcp_launch || true; pkill -9 -f dmtcp_worker || true'
rm -f "$HOME/.mana-slurm-${{SLURM_JOB_ID}}.rc"
sudo chown ec2-user:ec2-user {ckpt}
sudo chmod 775 {ckpt}
/opt/mana/bin/mana_coordinator --port 7779 --ckptdir {ckpt} --exit-on-last &
sleep 1
RC="/shared/slurm/.mana-slurm-${{SLURM_JOB_ID}}.rc"
DEST=".mana-slurm-${{SLURM_JOB_ID}}.rc"
sudo cp -f "$HOME/$DEST" "$RC"
sudo chmod 644 "$RC"
srun --overlap --immediate=5 --nodelist={nodelist} -N{nodes} -n{nodes} --label env RC="$RC" DEST="$DEST" /bin/sh -c 'cp -f "$RC" "$HOME/$DEST"'
srun --overlap --mpi=pmi2{oversubscribe}{overcommit} --nodelist={nodelist} -N{nodes} -n{procs} /opt/mana/bin/mana_restart --ckptdir {ckpt} --restartdir {ckpt} 
INNER
chmod +x /tmp/hpcac_restart.sh
salloc{oversubscribe}{overcommit} -w {nodelist} -N{nodes} -n{procs} -t 00:30:00 bash /tmp/hpcac_restart.sh 2>&1 | tee restart_output.txt"#,
        ckpt = ft.checkpoint_dir,
        nodelist = nodelist,
        nodes = worker_count,
    procs = process_count,
    oversubscribe = oversubscribe_flag,
    overcommit = overcommit_flag,
    )
}

async fn wait_for_worker_termination(
    context: &AwsClusterContext,
    node_index: usize,
    expected_instance_id: Option<&str>,
    timeout: Duration,
) -> Result<()> {
    let instance_name = context.ec2_instance_name(node_index);
    let start = tokio::time::Instant::now();

    while start.elapsed() < timeout {
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
        let resp = match request.send().await {
            Ok(resp) => resp,
            Err(e) => {
                if expected_instance_id.is_some() && e.to_string().contains("InvalidInstanceID.NotFound") {
                    return Ok(());
                }
                return Err(e.into());
            }
        };

        let mut found_any = false;
        let mut all_terminated = true;
        for reservation in resp.reservations() {
            for instance in reservation.instances() {
                found_any = true;
                if let Some(state_name) = instance.state().and_then(|s| s.name()) {
                    if *state_name != aws_sdk_ec2::types::InstanceStateName::Terminated {
                        all_terminated = false;
                    }
                } else {
                    all_terminated = false;
                }
            }
        }

        if found_any && all_terminated {
            return Ok(());
        }

        sleep(Duration::from_secs(5)).await;
    }

    bail!(
        "Timeout waiting for worker index {} (instance_id={:?}, name='{}') to reach Terminated before replacement",
        node_index,
        expected_instance_id,
        instance_name,
    )
}
