use super::watcher;
use crate::database::models::{Cluster, ClusterState, InstanceType, Node, ProviderConfig};
use crate::integrations::cloud_interface::CloudResourceManager;
use crate::integrations::providers::aws::AwsInterface;
use crate::utils;

use anyhow::{anyhow, bail, Result};
use aws_sdk_ec2::types::Filter;
use chrono::{Local, Utc};
use indicatif::ProgressBar;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::task::JoinHandle;
use tracing::{error, info};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AutoTestFailureConfig {
    /// Seconds after the run commands start before firing the simulated failure.
    trigger_after_secs: u64,
    /// 0-indexed position among worker nodes sorted by private IP.
    /// 0 = first worker (ip-10-0-0-11), 1 = second worker, etc.
    #[serde(default)]
    target_worker_index: usize,
    /// How long the simulated spot "warning" lasts before the instance is
    /// actually terminated. Mirrors the real AWS 2-minute spot warning.
    #[serde(default = "default_warning_time_secs")]
    warning_time_secs: u64,
}

fn default_warning_time_secs() -> u64 {
    120
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FaultToleranceConfig {
    strategy: String,
    #[serde(default = "default_replacement_allocation_mode")]
    replacement_allocation_mode: String,
    process_count: u32,
    checkpoint_dir: String,
    #[serde(default = "default_poll_interval")]
    poll_interval_secs: u64,
    #[serde(default)]
    terminate_cluster_on_zero_workers: bool,
    #[serde(default = "default_recovery_timeout_secs")]
    recovery_timeout_secs: u64,
    /// If set, automatically fires a simulated spot-interruption on the
    /// target worker after `trigger_after_secs` from when the run commands
    /// start. Omit this field entirely to disable auto failure injection.
    #[serde(default)]
    auto_test_failure: Option<AutoTestFailureConfig>,
}

fn default_replacement_allocation_mode() -> String {
    "spot".to_string()
}

fn default_poll_interval() -> u64 {
    10
}

fn default_recovery_timeout_secs() -> u64 {
    86400 // 24 hours — restarted job may need to run to completion
}

fn is_ft_active(ft: &Option<FaultToleranceConfig>) -> bool {
    matches!(ft, Some(f) if f.strategy != "NONE")
}

fn node_index_from_private_ip(ip: &str) -> Result<usize> {
    let last = ip
        .rsplit('.')
        .next()
        .ok_or_else(|| anyhow!("invalid private IP: {}", ip))?
        .parse::<usize>()?;
    if last < 10 {
        bail!("invalid HPCAC private IP: {}", ip);
    }
    Ok(last - 10)
}

fn extract_cycle_id(details: Option<&str>) -> Option<String> {
    let details = details?;
    let marker = "cycle_id=";
    let start = details.find(marker)? + marker.len();
    let tail = &details[start..];
    let end = tail
        .find(|c: char| c == '|' || c.is_whitespace())
        .unwrap_or(tail.len());
    let id = tail[..end].trim();
    if id.is_empty() {
        None
    } else {
        Some(id.to_string())
    }
}

fn extract_restart_output(details: Option<&str>) -> Option<String> {
    let details = details?;
    let begin_marker = "--restart_output_begin--";
    let end_marker = "--restart_output_end--";
    let begin = details.find(begin_marker)? + begin_marker.len();
    let tail = &details[begin..];
    let end = tail.find(end_marker)?;
    Some(tail[..end].trim().to_string())
}

/// Strip noisy lines from SSM / MANA output before writing to the result file.
/// Removes MANA low-level debug lines (dbg_* / argc_ptr), SSH known-hosts warnings, etc.
fn filter_ssm_output(output: &str) -> String {
    const NOISE_PREFIXES: &[&str] = &[
        "original argc_ptr:",
        "original argv_ptr:",
        "dbg_argc_addr:",
        "dbg_argv_ptr_addr:",
        "dbg_env_ptr_addr:",
        "dbg_auxv_ptr_addr:",
        "dbg_argv_strings_addr:",
        "dbg_env_strings_addr:",
        "dbg_end_marker_addr:",
        "dbg_bottom_of_stack:",
    ];
    const NOISE_SUBSTRINGS: &[&str] = &["Warning: Permanently added"];
    output
        .lines()
        .filter(|line| {
            let t = line.trim();
            !NOISE_PREFIXES.iter().any(|p| t.starts_with(p))
                && !NOISE_SUBSTRINGS.iter().any(|s| t.contains(s))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Compute duration in seconds between two RFC-3339 timestamp strings.
/// Returns `None` if either timestamp cannot be parsed.
fn phase_duration_secs(t_from: &str, t_to: &str) -> Option<f64> {
    let from = chrono::DateTime::parse_from_rfc3339(t_from)
        .ok()?
        .timestamp_millis();
    let to = chrono::DateTime::parse_from_rfc3339(t_to)
        .ok()?
        .timestamp_millis();
    Some((to - from) as f64 / 1000.0)
}

async fn get_latest_restart_output_for_cycles(
    pool: &SqlitePool,
    cluster_id: &str,
    run_started_at: &str,
    cycle_ids: &[String],
) -> Result<Option<(String, String)>> {
    if cycle_ids.is_empty() {
        return Ok(None);
    }

    let rows = sqlx::query!(
        r#"
        SELECT details
        FROM interruption_events
        WHERE cluster_id = ?
          AND occurred_at >= ?
          AND event_type = 'restart_completed'
        ORDER BY occurred_at DESC
        LIMIT 16
        "#,
        cluster_id,
        run_started_at,
    )
    .fetch_all(pool)
    .await?;

    for row in rows {
        let details = row.details.as_deref();
        if let Some(cycle_id) = extract_cycle_id(details) {
            if cycle_ids.iter().any(|id| id == &cycle_id) {
                if let Some(output) = extract_restart_output(details) {
                    return Ok(Some((cycle_id, output)));
                }
            }
        }
    }

    Ok(None)
}

async fn get_active_recovery_cycle_ids(
    pool: &SqlitePool,
    cluster_id: &str,
    run_started_at: &str,
) -> Result<Vec<String>> {
    let rows = sqlx::query!(
        r#"
        SELECT event_type as "event_type!", details
        FROM interruption_events
        WHERE cluster_id = ?
          AND occurred_at >= ?
          AND event_type IN ('recovery_started', 'recovery_completed', 'degraded_resume', 'no_workers_remaining', 'recovery_failed')
        ORDER BY occurred_at ASC
        "#,
        cluster_id,
        run_started_at,
    )
    .fetch_all(pool)
    .await?;

    let mut started: HashMap<String, bool> = HashMap::new();
    for row in rows {
        if let Some(cycle_id) = extract_cycle_id(row.details.as_deref()) {
            if row.event_type == "recovery_started" {
                started.insert(cycle_id, true);
            } else {
                started.remove(&cycle_id);
            }
        }
    }

    let mut active_ids: Vec<String> = started.into_keys().collect();
    active_ids.sort();
    Ok(active_ids)
}

async fn get_completed_recovery_cycle_ids(
    pool: &SqlitePool,
    cluster_id: &str,
    run_started_at: &str,
) -> Result<Vec<String>> {
    let rows = sqlx::query!(
        r#"
        SELECT details
        FROM interruption_events
        WHERE cluster_id = ?
          AND occurred_at >= ?
          AND event_type IN ('recovery_completed', 'degraded_resume', 'no_workers_remaining')
        ORDER BY occurred_at ASC
        "#,
        cluster_id,
        run_started_at,
    )
    .fetch_all(pool)
    .await?;

    let mut cycle_ids: Vec<String> = rows
        .iter()
        .filter_map(|row| extract_cycle_id(row.details.as_deref()))
        .collect();
    cycle_ids.sort();
    cycle_ids.dedup();
    Ok(cycle_ids)
}

async fn wait_for_watcher_recovery_if_needed(
    pool: &SqlitePool,
    cluster_id: &str,
    run_started_at: &str,
    max_wait: Duration,
    progress_bar: &utils::ProgressTracker,
) -> Result<bool> {
    let started_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(1)
        FROM interruption_events
        WHERE cluster_id = ?
          AND occurred_at >= ?
          AND event_type = 'recovery_started'
        "#,
    )
    .bind(cluster_id)
    .bind(run_started_at)
    .fetch_one(pool)
    .await?;

    let terminal_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(1)
        FROM interruption_events
        WHERE cluster_id = ?
          AND occurred_at >= ?
                    AND event_type IN ('recovery_completed', 'degraded_resume', 'no_workers_remaining', 'recovery_failed')
        "#,
    )
    .bind(cluster_id)
    .bind(run_started_at)
    .fetch_one(pool)
    .await?;

    if started_count <= terminal_count {
        return Ok(false);
    }

    let waiting_message = format!(
        "Waiting for watcher recovery and restarted workload completion (started={}, completed={})...",
        started_count, terminal_count
    );
    progress_bar.update_message(&waiting_message);
    info!("{}", waiting_message);

    let deadline = Instant::now() + max_wait;
    loop {
        let started_now = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(1)
            FROM interruption_events
            WHERE cluster_id = ?
              AND occurred_at >= ?
              AND event_type = 'recovery_started'
            "#,
        )
        .bind(cluster_id)
        .bind(run_started_at)
        .fetch_one(pool)
        .await?;

        let terminal_now = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(1)
            FROM interruption_events
            WHERE cluster_id = ?
              AND occurred_at >= ?
                            AND event_type IN ('recovery_completed', 'degraded_resume', 'no_workers_remaining', 'recovery_failed')
            "#,
        )
        .bind(cluster_id)
        .bind(run_started_at)
        .fetch_one(pool)
        .await?;

        let active_cycle_ids = get_active_recovery_cycle_ids(pool, cluster_id, run_started_at).await?;
        let active_cycle_label = if active_cycle_ids.is_empty() {
            "none".to_string()
        } else {
            active_cycle_ids.join(",")
        };

        if started_now <= terminal_now {
            let latest_terminal = sqlx::query!(
                r#"
                SELECT event_type as "event_type!", details
                FROM interruption_events
                WHERE cluster_id = ?
                  AND occurred_at >= ?
                  AND event_type IN ('recovery_completed', 'degraded_resume', 'no_workers_remaining', 'recovery_failed')
                ORDER BY occurred_at DESC
                LIMIT 1
                "#,
                cluster_id,
                run_started_at,
            )
            .fetch_optional(pool)
            .await?;

            if let Some(latest) = latest_terminal {
                if latest.event_type == "recovery_failed" {
                    let failed_cycle = extract_cycle_id(latest.details.as_deref())
                        .unwrap_or_else(|| "unknown".to_string());
                    let waiting_retry_message = format!(
                        "Latest watcher cycle failed (cycle={}); waiting for next recovery attempt...",
                        failed_cycle
                    );
                    progress_bar.update_message(&waiting_retry_message);
                    info!("{}", waiting_retry_message);

                    if Instant::now() >= deadline {
                        bail!(
                            "Watcher recovery did not complete successfully before timeout (last_failed_cycle={}).",
                            failed_cycle
                        );
                    }

                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            }

            let completion_message = format!(
                "Watcher recovery and restarted workload completed (started={}, completed={}, active_cycles={}).",
                started_now, terminal_now, active_cycle_label
            );
            progress_bar.update_message(&completion_message);
            info!("{}", completion_message);
            return Ok(true);
        }

        if Instant::now() >= deadline {
            let timeout_message = format!(
                "Timed out waiting for watcher recovery/restarted workload completion (started={}, completed={}, active_cycles={}).",
                started_now, terminal_now, active_cycle_label
            );
            progress_bar.update_message(&timeout_message);
            bail!("{}", timeout_message);
        }

        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct TasksYaml {
    #[serde(default)]
    fault_tolerance: Option<FaultToleranceConfig>,
    tasks: Vec<Task>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Task {
    task_tag: String,
    setup_commands: Vec<String>,
    run_commands: Vec<String>,
}

pub async fn run_task(
    pool: &SqlitePool,
    yaml_file_path: &str,
    cluster_id: &str,
    skip_confirmation: bool,
) -> Result<()> {
    info!("Invoked `run_tasks` command...");

    // Prepare report file
    let mut report_dir = PathBuf::from("results");
    report_dir.push(format!("cluster_{}", cluster_id));

    if let Err(e) = fs::create_dir_all(&report_dir) {
        error!("Failed to create directory for result report: {}", e);
        bail!("FileSystem error: {}", e);
    }

    let timestamp = Local::now().format("%Y-%m-%dT%H:%M:%S");
    let filename = format!("{}.txt", timestamp);
    let report_path = report_dir.join(&filename);

    // Open file in Append/Create mode
    let mut report_file = match OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&report_path)
    {
        Ok(f) => {
            info!("Result report will be streamed to '{:?}'", report_path);
            f
        }
        Err(e) => {
            error!("Failed to create report file: {}", e);
            bail!("FileSystem error: {}", e);
        }
    };

    // Helper closure to write to file and handle errors cleanly
    macro_rules! log_report {
        ($($arg:tt)*) => ({
            let text = format!($($arg)*);
            // print!("{}", text);
            if let Err(e) = report_file.write_all(text.as_bytes()) {
                error!("Failed to write to report file: {}", e);
            }
            if let Err(e) = report_file.flush() {
                error!("Failed to flush report file: {}", e);
            }
        })
    }

    info!("Parsing contents of `tasks_config.yaml` file...");
    let path = Path::new(yaml_file_path);
    let tasks_yaml_str: String = match fs::read_to_string(path) {
        Ok(result) => {
            info!("Successfully read file: '{}'", yaml_file_path);
            result
        }
        Err(e) => {
            error!("{}", e.to_string());
            bail!("Failed to read file '{}'", yaml_file_path)
        }
    };

    let tasks_yaml: TasksYaml = match serde_yaml::from_str(&tasks_yaml_str) {
        Ok(result) => {
            info!("Parsed tasks yaml file successfully");
            result
        }
        Err(e) => {
            error!("{}", e.to_string());
            bail!(
                "Failed to parse yaml file: '{}': {:?}",
                yaml_file_path,
                e.to_string()
            )
        }
    };

    if let Some(ft) = &tasks_yaml.fault_tolerance {
        match ft.strategy.as_str() {
            "NONE" | "REPLACE_RESUME" | "DEGRADED_RESUME" => {}
            other => {
                bail!(
                    "Invalid fault_tolerance.strategy '{}'. Expected NONE, REPLACE_RESUME, or DEGRADED_RESUME.",
                    other
                )
            }
        }
        if ft.strategy == "REPLACE_RESUME"
            && ft.replacement_allocation_mode != "spot"
            && ft.replacement_allocation_mode != "on-demand"
        {
            bail!(
                "Invalid fault_tolerance.replacement_allocation_mode '{}'. Expected 'spot' or 'on-demand'.",
                ft.replacement_allocation_mode
            );
        }
    }

    info!("fetching Clusters (id='{}')", cluster_id);
    let cluster = match Cluster::fetch_by_id(pool, cluster_id).await? {
        Some(cluster) => cluster,
        None => {
            println!("Cluster (id='{}') not found", cluster_id);
            return Ok(());
        }
    };
    let nodes = cluster.get_nodes(pool).await?;

    if cluster.state != ClusterState::Running {
        println!("Cluster with id '{}' was not spawned.", cluster_id);
        return Ok(());
    }
    info!("Found online Cluster (id='{}')!", cluster_id);

    let provider_config =
        match ProviderConfig::fetch_by_id(pool, cluster.provider_config_id).await? {
            Some(config) => config,
            None => {
                error!("Missing ProviderConfig '{}'", cluster.provider_config_id);
                bail!("Data Consistency Failure");
            }
        };
    let config_vars = provider_config.get_config_vars(pool).await?;
    let provider_id = provider_config.provider_id.clone();
    let cloud_interface = match provider_id.as_str() {
        "aws" => AwsInterface {
            config_vars: config_vars.clone(),
        },
        _ => {
            bail!("Provider '{}' is currently not supported.", &provider_id)
        }
    };

    // Confirm with user
    if let Some(ft) = &tasks_yaml.fault_tolerance {
        println!("Fault Tolerance:");
        println!(" - strategy: {}", ft.strategy);
        if ft.strategy == "REPLACE_RESUME" {
            println!(
                " - replacement_allocation_mode: {}",
                ft.replacement_allocation_mode
            );
        }
        println!(" - process_count: {}", ft.process_count);
        println!(" - checkpoint_dir: {}", ft.checkpoint_dir);
        println!(" - poll_interval_secs: {}", ft.poll_interval_secs);
        println!(
            " - terminate_cluster_on_zero_workers: {}",
            ft.terminate_cluster_on_zero_workers
        );
        if let Some(atf) = &ft.auto_test_failure {
            println!(" - auto_test_failure:");
            println!("     trigger_after_secs : {}", atf.trigger_after_secs);
            println!("     target_worker_index: {} (ip-10-0-0-{})", atf.target_worker_index, 11 + atf.target_worker_index);
            println!("     warning_time_secs  : {}", atf.warning_time_secs);
        }
    }
    println!("Tasks:");
    for task in tasks_yaml.tasks.iter() {
        println!(" - name: {}", task.task_tag);
        println!("   setup_commands:");
        for command in task.setup_commands.iter() {
            println!("     - {}", command);
        }
        println!("   run_commands:");
        for command in task.run_commands.iter() {
            println!("     - {}", command);
        }
        println!();
    }
    if !(utils::user_confirmation(skip_confirmation, "Run this tasks on the cluster?")?) {
        return Ok(());
    }
    println!();

    // Get context and task_runner_instance_id
    let context = cloud_interface.create_cluster_context(&cluster)?;
    let task_runner_instance_name = context.ec2_instance_name(0);
    
    // Filter by Name
    let name_filter = Filter::builder()
        .name("tag:Name")
        .values(&task_runner_instance_name)
        .build();

    // Filter by State
    let state_filter = Filter::builder()
        .name("instance-state-name")
        .values("running")
        .build();

    let resp = context
        .ec2_client
        .describe_instances()
        .filters(name_filter)
        .filters(state_filter)
        .send()
        .await?;

    let ec2_id = resp
        .reservations()
        .iter()
        .flat_map(|r| r.instances())
        .find_map(|i| i.instance_id().map(|id| id.to_string()));

    let task_runner_instance_ec2_id = match ec2_id {
        Some(id) => id,
        None => bail!("Unable to retrieve ec2 instance id."),
    };

    let mut worker_nodes: Vec<(usize, Node)> = nodes
        .iter()
        .filter(|n| n.role == "worker")
        .map(|n| {
            let ip = n
                .private_ip
                .as_deref()
                .ok_or_else(|| anyhow!("Worker node {} has no private IP", n.id))?;
            Ok((node_index_from_private_ip(ip)?, n.clone()))
        })
        .collect::<Result<Vec<_>>>()?;
    worker_nodes.sort_by_key(|(idx, _)| *idx);

    if is_ft_active(&tasks_yaml.fault_tolerance) {
        println!("Fault tolerance worker mapping (private_ip -> node_index):");
        for (idx, worker) in &worker_nodes {
            let ip = worker.private_ip.as_deref().unwrap_or("<missing>");
            println!(" - {} -> {}", ip, idx);
        }
    }

    let head_count = nodes.iter().filter(|n| n.role == "head").count();
    if head_count != 1 {
        bail!(
            "Invalid cluster topology: expected exactly one head node, found {}",
            head_count
        );
    }
    let head = nodes.iter().find(|n| n.role == "head").unwrap();
    let head_ip = head
        .private_ip
        .as_deref()
        .ok_or_else(|| anyhow!("Head has no private IP"))?;
    if node_index_from_private_ip(head_ip)? != 0 {
        bail!(
            "Invalid topology: head private IP {} does not map to node_index 0",
            head_ip
        );
    }

    let worker_instance_ids: HashMap<usize, String> = if is_ft_active(&tasks_yaml.fault_tolerance)
    {
        let mut map = HashMap::new();
        let running_filter = Filter::builder()
            .name("instance-state-name")
            .values("running")
            .build();
        let cluster_filter = Filter::builder()
            .name("tag:ClusterId")
            .values(&cluster.id)
            .build();

        for (node_index, worker) in &worker_nodes {
            let private_ip = worker
                .private_ip
                .as_deref()
                .ok_or_else(|| anyhow!("Worker at index {} has no private IP", node_index))?;
            let ip_filter = Filter::builder()
                .name("private-ip-address")
                .values(private_ip)
                .build();
            let resp = context
                .ec2_client
                .describe_instances()
                .filters(ip_filter)
                .filters(running_filter.clone())
                .filters(cluster_filter.clone())
                .send()
                .await?;
            let worker_ec2_id = resp
                .reservations()
                .iter()
                .flat_map(|r| r.instances())
                .find_map(|i| i.instance_id().map(|id| id.to_string()));

            if let Some(worker_ec2_id) = worker_ec2_id {
                map.insert(*node_index, worker_ec2_id);
            } else {
                println!(
                    "[watcher] warning: no running instance found for worker IP {} (node_index={}); preflight recovery will handle it",
                    private_ip, node_index
                );
            }
        }

        map
    } else {
        HashMap::new()
    };

    info!("Checking SSM Agent status for instance '{}'...", task_runner_instance_ec2_id);
    println!("Waiting for node to be ready for commands (SSM Agent)...\n");

    cloud_interface.wait_for_ssm_agent_ready(
        &context, 
        &task_runner_instance_ec2_id, 
        Duration::from_secs(300) // Wait up to 5 minutes
    ).await?;

    info!("SSM Agent is ready!");

    // Write cluster details to file
    log_report!("-=-=-=-=-=-=-=-= CLUSTER DETAILS =-=-=-=-=-=-=-=-\n");
    log_report!("{:<35}: {}\n", "Cluster Name", cluster.display_name);
    log_report!("{:<35}: {}\n", "Provider", cluster.provider_id);
    log_report!("{:<35}: {}\n", "Region", cluster.region);
    log_report!(
        "{:<35}: {}\n",
        "Availability Zone",
        cluster.availability_zone
    );
    log_report!("{:<35}: {}\n", "Use Node Affinity", cluster.use_node_affinity);
    log_report!(
        "{:<35}: {}\n",
        "Use Elastic Fabric Adapters (EFAs)",
        cluster.use_elastic_fabric_adapters
    );
    log_report!(
        "{:<35}: {}\n",
        "Use Elastic File System (EFS)",
        cluster.use_elastic_file_system
    );
    log_report!(
        "{:<35}: {}\n",
        "On Instance Creation Failure",
        cluster
            .on_instance_creation_failure
            .clone()
            .unwrap()
            .to_string()
    );
    log_report!(
        "{:<35}: {}\n",
        "Provider Config",
        provider_config.display_name
    );
    log_report!("{:<35}: {}\n\n", "Node Count", nodes.len());
    if let Some(ft) = &tasks_yaml.fault_tolerance {
        log_report!("{:<35}: {}\n", "Fault Tolerance Strategy", ft.strategy);
        if ft.strategy == "REPLACE_RESUME" {
            log_report!(
                "{:<35}: {}\n",
                "Replacement Allocation Mode",
                ft.replacement_allocation_mode
            );
        }
        log_report!("{:<35}: {}\n", "FT Process Count", ft.process_count);
        log_report!("{:<35}: {}\n", "FT Checkpoint Dir", ft.checkpoint_dir);
        log_report!(
            "{:<35}: {}\n",
            "FT Poll Interval (secs)",
            ft.poll_interval_secs
        );
        log_report!(
            "{:<35}: {}\n\n",
            "FT Terminate on Zero Workers",
            ft.terminate_cluster_on_zero_workers
        );
    }

    log_report!("Node Details:\n");
    for (i, node) in nodes.iter().enumerate() {
        let instance_type_name = &node.instance_type;
        let instance_details = match InstanceType::fetch_by_name_and_region(
            pool,
            instance_type_name,
            &cluster.region,
        )
        .await?
        {
            Some(it) => it,
            None => bail!("Missing InstanceType '{}' in region '{}'", instance_type_name, &cluster.region),
        };
        let processor_info = match &instance_details.core_count {
            Some(cores) => {
                format!(
                    "{}-Core {} {}",
                    cores, instance_details.cpu_architecture, instance_details.cpu_type
                )
            }
            None => {
                format!(
                    "{} {}",
                    instance_details.cpu_architecture, instance_details.cpu_type
                )
            }
        };

        let gpu_info = match instance_details.gpu_type {
            Some(gpu) => {
                format!("{}x {}", instance_details.gpu_count, gpu)
            }
            None => "N/A".to_string(),
        };

        log_report!("  Node {}:\n", i + 1);
        log_report!("    Instance Type   : {}\n", node.instance_type);
        log_report!("    Processor       : {}\n", processor_info);
        log_report!("    vCPUs:          : {}\n", instance_details.vcpus);
        log_report!("    GPUs:           : {}\n", gpu_info);
        log_report!("    Image ID        : {}\n", node.image_id);
        log_report!("    Allocation Mode : {}\n", node.allocation_mode);
        log_report!(
            "    Burstable Mode  : {}\n",
            node.burstable_mode.as_deref().unwrap_or("N/A")
        );
    }
    log_report!("-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-\n\n");

    // Running tasks
    let steps: usize = tasks_yaml.tasks.iter().fold(0, |acc, task| {
        acc + task.setup_commands.len() + task.run_commands.len()
    });

    let multi = utils::ProgressTracker::create_multi();
    let main_progress =
        utils::ProgressTracker::add_to_multi(&multi, steps as u64, Some("Initializing..."));

    let operation_spinner = utils::ProgressTracker::new_indeterminate(&multi, "Initializing...");
    let run_started_at = Utc::now().to_rfc3339();
    let watcher_progress_bar: Option<ProgressBar> = if is_ft_active(&tasks_yaml.fault_tolerance) { Some(
            utils::ProgressTracker::new_indeterminate(&multi, "[watcher] waiting for events")
                .progress_bar
                .clone(),
        )
    } else {
        None
    };
    let auto_failure_progress_bar: Option<ProgressBar> =
        if let Some(ft) = &tasks_yaml.fault_tolerance {
            ft.auto_test_failure.as_ref().map(|atf| {
                utils::ProgressTracker::new_indeterminate(
                    &multi,
                    &format!(
                        "[auto-failure] armed — fires in {}s on ip-10-0-0-{}",
                        atf.trigger_after_secs,
                        11 + atf.target_worker_index
                    ),
                )
                .progress_bar
                .clone()
            })
        } else {
            None
        };

    let watcher_handle: Option<JoinHandle<Result<()>>> =
        if let Some(ft) = &tasks_yaml.fault_tolerance {
            if ft.strategy == "NONE" {
                None
            } else {

                let watcher_interface = AwsInterface {
                    config_vars: config_vars.clone(),
                };
                let watcher_context = watcher_interface.create_cluster_context(&cluster)?;
                let watcher_ft = watcher::FaultToleranceConfig {
                    strategy: ft.strategy.clone(),
                    replacement_allocation_mode: ft.replacement_allocation_mode.clone(),
                    process_count: ft.process_count,
                    checkpoint_dir: ft.checkpoint_dir.clone(),
                    poll_interval_secs: ft.poll_interval_secs,
                    terminate_cluster_on_zero_workers: ft.terminate_cluster_on_zero_workers,
                };

                let pool_clone = pool.clone();
                let cluster_clone = cluster.clone();
                let all_nodes_clone = nodes.clone();
                let worker_nodes_clone = worker_nodes.clone();
                let worker_ids_clone = worker_instance_ids.clone();
                let head_id_clone = task_runner_instance_ec2_id.clone();
                let watcher_progress_clone = watcher_progress_bar.clone();

                Some(tokio::spawn(async move {
                    watcher::run_watcher(
                        pool_clone,
                        cluster_clone,
                        all_nodes_clone,
                        worker_nodes_clone,
                        worker_ids_clone,
                        head_id_clone,
                        watcher_context,
                        watcher_interface,
                        watcher_ft,
                        watcher_progress_clone,
                    )
                    .await
                }))
            }
        } else {
            None
        };


    info!("Starting Task loop...");
    for task in tasks_yaml.tasks.iter() {
        {
            let sep = "=".repeat(60);
            log_report!("\n{}\n TASK: {}\n{}\n\n", sep, task.task_tag, sep);
        }

        let running_task_message = format!("Running task '{}' setup commands...", task.task_tag);
        info!(running_task_message);
        main_progress.update_message(&running_task_message);

        let setup_commands_start = Instant::now();
        let mut setup_idx: usize = 0;
        let setup_total = task.setup_commands.len();
        log_report!(
            "-- SETUP ({} command{}) --\n\n",
            setup_total,
            if setup_total == 1 { "" } else { "s" }
        );
        while setup_idx < setup_total {
            let command = &task.setup_commands[setup_idx];
            operation_spinner.update_message(&format!(
                "Executing setup command {}/{} for task '{}': '{}'",
                setup_idx + 1,
                setup_total,
                task.task_tag,
                command
            ));
            log_report!("[setup {}/{}] $ {}\n", setup_idx + 1, setup_total, command);

            let result = async {
                let cmd_id = cloud_interface
                    .create_ssm_command(&context, &task_runner_instance_ec2_id, command.clone())
                    .await?;

                cloud_interface
                    .poll_ssm_command_until_completion(
                        &context,
                        &cmd_id,
                        &task_runner_instance_ec2_id,
                        Duration::from_secs(3600),
                        Duration::from_secs(2),
                    )
                    .await
            }
            .await;

            match &result {
                Ok(out) => {
                    log_report!("{}\n\n", filter_ssm_output(out));
                    setup_idx += 1;
                    main_progress.inc(1);
                }
                Err(e) => {
                    log_report!("error: {}\n\n", e);
                    if !is_ft_active(&tasks_yaml.fault_tolerance) {
                        if let Some(handle) = watcher_handle.as_ref() {
                            handle.abort();
                        }
                        bail!("Task setup command failed: {}", e);
                    }

                    let recovery_timeout = tasks_yaml
                        .fault_tolerance
                        .as_ref()
                        .map(|ft| ft.recovery_timeout_secs)
                        .unwrap_or_else(default_recovery_timeout_secs);
                    let recovered = match wait_for_watcher_recovery_if_needed(
                        pool,
                        &cluster.id,
                        &run_started_at,
                        Duration::from_secs(recovery_timeout),
                        &operation_spinner,
                    )
                    .await
                    {
                        Ok(recovered) => recovered,
                        Err(wait_err) => {
                            if let Some(handle) = watcher_handle.as_ref() {
                                handle.abort();
                            }
                            bail!("{}", wait_err);
                        }
                    };

                    if !recovered {
                        if let Some(handle) = watcher_handle.as_ref() {
                            handle.abort();
                        }
                        bail!(
                            "Task setup command failed without watcher recovery event: {}",
                            e
                        );
                    }

                    let completed_cycles =
                        get_completed_recovery_cycle_ids(pool, &cluster.id, &run_started_at)
                            .await?;
                    let completed_cycles_label = if completed_cycles.is_empty() {
                        "unknown".to_string()
                    } else {
                        completed_cycles.join(",")
                    };
                    let resume_message = format!(
                        "Recovery complete (cycle(s): {}). Treating failed setup command {}/{} of task '{}' as recovered and continuing to next command.",
                        completed_cycles_label,
                        setup_idx + 1,
                        setup_total,
                        task.task_tag
                    );
                    info!("{}", resume_message);
                    println!("{}", resume_message);
                    main_progress.update_message(&resume_message);
                    operation_spinner.update_message(&resume_message);
                    log_report!("\n[recovery] {}\n", resume_message);
                    if let Some((cycle_id, restart_output)) = get_latest_restart_output_for_cycles(
                        pool,
                        &cluster.id,
                        &run_started_at,
                        &completed_cycles,
                    )
                    .await?
                    {
                        log_report!("[recovery] watcher restart output (cycle={}):\n", cycle_id);
                        for line in filter_ssm_output(&restart_output).lines() {
                            log_report!("  {}\n", line);
                        }
                        log_report!("\n");
                    }
                    setup_idx += 1;
                    main_progress.inc(1);
                }
            }
        }
        let setup_commands_elapsed_sec = setup_commands_start.elapsed().as_secs_f64();

        let running_task_message = format!("Running task '{}' run_commands...", task.task_tag);
        info!(running_task_message);
        main_progress.update_message(&running_task_message);

        let run_commands_start = Instant::now();

        // Spawn the auto test-failure trigger if configured.
        // It fires `trigger_after_secs` after the run phase starts,
        // simulating a spot interruption on the target worker.
        // The handle is aborted after the run phase so it never fires
        // once the job has already completed normally.
        let auto_failure_handle: Option<tokio::task::JoinHandle<()>> =
            if let Some(atf) = tasks_yaml
                .fault_tolerance
                .as_ref()
                .filter(|_ft| is_ft_active(&tasks_yaml.fault_tolerance))
                .and_then(|ft| ft.auto_test_failure.as_ref())
            {
                // Resolve target worker private IP from 0-indexed position
                let target_ip = worker_nodes
                    .get(atf.target_worker_index)
                    .and_then(|(_, n)| n.private_ip.clone());

                match target_ip {
                    None => {
                        let msg = format!(
                            "[auto-failure] WARNING: target_worker_index={} out of range \
                             ({} workers) — auto failure will NOT fire.",
                            atf.target_worker_index,
                            worker_nodes.len()
                        );
                        info!("{}", msg);
                        log_report!("{}\n", msg);
                        if let Some(pb) = &auto_failure_progress_bar {
                            pb.finish_with_message(msg.clone());
                        }
                        None
                    }
                    Some(ip) => {
                        let trigger_secs  = atf.trigger_after_secs;
                        let warning_secs  = atf.warning_time_secs;
                        let pool_clone    = pool.clone();
                        let cluster_clone = cluster.clone();
                        let iface_clone   = AwsInterface { config_vars: config_vars.clone() };
                        let pb_clone      = auto_failure_progress_bar.clone();

                        let msg = format!(
                            "[auto-failure] Scheduled: will terminate worker {} in {}s \
                             (warning_time={}s).",
                            ip, trigger_secs, warning_secs
                        );
                        info!("{}", msg);
                        log_report!("{}\n\n", msg);

                        Some(tokio::spawn(async move {
                            // Live countdown on the progress bar
                            for remaining in (1..=trigger_secs).rev() {
                                if let Some(pb) = &pb_clone {
                                    pb.set_message(format!(
                                        "[auto-failure] fires in {}s on {}",
                                        remaining, ip
                                    ));
                                }
                                tokio::time::sleep(Duration::from_secs(1)).await;
                            }

                            info!("[auto-failure] Firing simulated spot interruption on {}", ip);
                            if let Some(pb) = &pb_clone {
                                pb.set_message(format!(
                                    "[auto-failure] FIRING on {} — waiting {}s warning...",
                                    ip, warning_secs
                                ));
                            }

                            match iface_clone
                                .simulate_cluster_failure(
                                    &pool_clone,
                                    cluster_clone,
                                    &ip,
                                    warning_secs,
                                )
                                .await
                            {
                                Ok(_) => {
                                    info!("[auto-failure] Instance {} terminated.", ip);
                                    if let Some(pb) = &pb_clone {
                                        pb.set_message(format!(
                                            "[auto-failure] {} terminated — watcher recovering",
                                            ip
                                        ));
                                    }
                                }
                                Err(e) => {
                                    error!("[auto-failure] simulate_cluster_failure failed: {}", e);
                                    if let Some(pb) = &pb_clone {
                                        pb.set_message(format!(
                                            "[auto-failure] ERROR terminating {}: {}",
                                            ip, e
                                        ));
                                    }
                                }
                            }
                        }))
                    }
                }
            } else {
                None
            };

        let mut run_idx: usize = 0;
        let run_total = task.run_commands.len();
        log_report!(
            "-- RUN ({} command{}) --\n\n",
            run_total,
            if run_total == 1 { "" } else { "s" }
        );
        while run_idx < run_total {
            let command = &task.run_commands[run_idx];
            operation_spinner.update_message(&format!(
                "Executing run command {}/{} for task '{}': '{}'",
                run_idx + 1,
                run_total,
                task.task_tag,
                command
            ));
            log_report!("[run {}/{}] $ {}\n", run_idx + 1, run_total, command);

            let result = async {
                let cmd_id = cloud_interface
                    .create_ssm_command(&context, &task_runner_instance_ec2_id, command.clone())
                    .await?;

                cloud_interface
                    .poll_ssm_command_until_completion(
                        &context,
                        &cmd_id,
                        &task_runner_instance_ec2_id,
                        Duration::from_secs(3600),
                        Duration::from_secs(2),
                    )
                    .await
            }
            .await;

            match &result {
                Ok(out) => {
                    log_report!("{}\n\n", filter_ssm_output(out));
                    run_idx += 1;
                    main_progress.inc(1);
                }
                Err(e) => {
                    log_report!("error: {}\n\n", e);
                    if !is_ft_active(&tasks_yaml.fault_tolerance) {
                        if let Some(handle) = watcher_handle.as_ref() {
                            handle.abort();
                        }
                        bail!("Task run command failed: {}", e);
                    }

                    let recovery_timeout = tasks_yaml
                        .fault_tolerance
                        .as_ref()
                        .map(|ft| ft.recovery_timeout_secs)
                        .unwrap_or_else(default_recovery_timeout_secs);
                    let recovered = match wait_for_watcher_recovery_if_needed(
                        pool,
                        &cluster.id,
                        &run_started_at,
                        Duration::from_secs(recovery_timeout),
                        &operation_spinner,
                    )
                    .await
                    {
                        Ok(recovered) => recovered,
                        Err(wait_err) => {
                            if let Some(handle) = watcher_handle.as_ref() {
                                handle.abort();
                            }
                            bail!("{}", wait_err);
                        }
                    };

                    if !recovered {
                        if let Some(handle) = watcher_handle.as_ref() {
                            handle.abort();
                        }
                        bail!(
                            "Task run command failed without watcher recovery event: {}",
                            e
                        );
                    }

                    let completed_cycles =
                        get_completed_recovery_cycle_ids(pool, &cluster.id, &run_started_at)
                            .await?;
                    let completed_cycles_label = if completed_cycles.is_empty() {
                        "unknown".to_string()
                    } else {
                        completed_cycles.join(",")
                    };
                    let resume_message = format!(
                        "Recovery complete (cycle(s): {}). Treating failed run command {}/{} of task '{}' as recovered and continuing to next command.",
                        completed_cycles_label,
                        run_idx + 1,
                        run_total,
                        task.task_tag
                    );
                    info!("{}", resume_message);
                    println!("{}", resume_message);
                    main_progress.update_message(&resume_message);
                    operation_spinner.update_message(&resume_message);
                    log_report!("\n[recovery] {}\n", resume_message);
                    if let Some((cycle_id, restart_output)) = get_latest_restart_output_for_cycles(
                        pool,
                        &cluster.id,
                        &run_started_at,
                        &completed_cycles,
                    )
                    .await?
                    {
                        log_report!("[recovery] watcher restart output (cycle={}):\n", cycle_id);
                        for line in filter_ssm_output(&restart_output).lines() {
                            log_report!("  {}\n", line);
                        }
                        log_report!("\n");
                    }
                    run_idx += 1;
                    main_progress.inc(1);
                }
            }
        }

        let run_commands_elapsed_sec = run_commands_start.elapsed().as_secs_f64();
        let exec_time = setup_commands_elapsed_sec + run_commands_elapsed_sec;

        // Cancel the auto-failure trigger if it hasn't fired yet (job finished before trigger)
        if let Some(handle) = auto_failure_handle {
            handle.abort();
        }

        log_report!("-- TIMING --\n");
        log_report!("  Setup time  : {:>10.3} s\n", setup_commands_elapsed_sec);
        log_report!("  Run time    : {:>10.3} s\n", run_commands_elapsed_sec);
        log_report!("  Total time  : {:>10.3} s\n", exec_time);
        log_report!("{}\n\n", "=".repeat(60));

        // [RUN_METRICS]: machine-readable per-task summary for analysis scripts.
        // Always written when a task completes (setup + run both finished).
        // Files without this block should be treated as failed/incomplete runs.
        {
            let strategy_str = tasks_yaml
                .fault_tolerance
                .as_ref()
                .map(|ft| ft.strategy.as_str())
                .unwrap_or("NONE");
            let worker_count = worker_nodes.len();
            let worker_instance_type = worker_nodes
                .first()
                .map(|(_, n)| n.instance_type.as_str())
                .unwrap_or("unknown");
            let head_instance_type = nodes
                .iter()
                .find(|n| n.role == "head")
                .map(|n| n.instance_type.as_str())
                .unwrap_or("unknown");

            let auto_trigger_secs = tasks_yaml
                .fault_tolerance
                .as_ref()
                .and_then(|ft| ft.auto_test_failure.as_ref())
                .map(|atf| atf.trigger_after_secs);

            log_report!("[RUN_METRICS]\n");
            log_report!("task_tag={}\n",              task.task_tag);
            log_report!("strategy={}\n",              strategy_str);
            log_report!("workers={}\n",               worker_count);
            log_report!("worker_instance_type={}\n",  worker_instance_type);
            log_report!("head_instance_type={}\n",    head_instance_type);
            log_report!("ft_wall_time_s={:.3}\n",     run_commands_elapsed_sec);
            log_report!("setup_time_s={:.3}\n",       setup_commands_elapsed_sec);
            log_report!("total_time_s={:.3}\n",       exec_time);
            if let Some(t) = auto_trigger_secs {
                log_report!("auto_failure_trigger_secs={}\n", t);
            }
            log_report!("status=SUCCESS\n");
            log_report!("[/RUN_METRICS]\n\n");
        }
    }

    operation_spinner.finish_with_message("All commands of all tasks completed!");
    main_progress.finish_with_message("All tasks completed!");

    if is_ft_active(&tasks_yaml.fault_tolerance) {
        let ft_strategy = tasks_yaml
            .fault_tolerance
            .as_ref()
            .map(|ft| ft.strategy.as_str())
            .unwrap_or("UNKNOWN");

        let cluster_id_ref = cluster.id.as_str();
        let run_started_at_ref = run_started_at.as_str();
        let recovery_rows = sqlx::query!(
            r#"
            SELECT event_type as "event_type!", details, occurred_at as "occurred_at!", node_private_ip as "node_private_ip!"
            FROM interruption_events
            WHERE cluster_id = ?
              AND occurred_at >= ?
            ORDER BY occurred_at ASC
            "#,
            cluster_id_ref,
            run_started_at_ref,
        )
        .fetch_all(pool)
        .await?;

        if !recovery_rows.is_empty() {
            // Group events by cycle_id, preserving first-seen order.
            // Each entry: Vec<(occurred_at, event_type, node_ip, details)>
            let mut cycle_events: HashMap<
                String,
                Vec<(String, String, String, Option<String>)>,
            > = HashMap::new();
            let mut ordered_cycle_ids: Vec<String> = Vec::new();

            for row in &recovery_rows {
                let cycle_id = extract_cycle_id(row.details.as_deref())
                    .unwrap_or_else(|| "no_cycle_id".to_string());
                if !cycle_events.contains_key(&cycle_id) {
                    ordered_cycle_ids.push(cycle_id.clone());
                }
                cycle_events.entry(cycle_id).or_default().push((
                    row.occurred_at.clone(),
                    row.event_type.clone(),
                    row.node_private_ip.clone(),
                    row.details.clone(),
                ));
            }

            let sep = "=".repeat(60);
            log_report!("\n{}\n FT RECOVERY SUMMARY\n{}\n", sep, sep);
            log_report!("  Strategy        : {}\n", ft_strategy);
            log_report!("  Recovery Cycles : {}\n", ordered_cycle_ids.len());

            for (cycle_num, cycle_id) in ordered_cycle_ids.iter().enumerate() {
                let events = match cycle_events.get(cycle_id) {
                    Some(e) => e,
                    None => continue,
                };

                let node_ip = events
                    .first()
                    .map(|(_, _, ip, _)| ip.as_str())
                    .unwrap_or("?");

                log_report!(
                    "\n-- Cycle #{} (id={}) -- Node IP: {}\n",
                    cycle_num + 1,
                    cycle_id,
                    node_ip
                );
                log_report!("\n  Event Timeline:\n");

                let mut prev_ts_ms: Option<i64> = None;
                for (occurred_at, event_type, _, _) in events {
                    // restart_completed is verbose; its output is shown in the restart section
                    if event_type == "restart_completed" {
                        continue;
                    }
                    let ts_ms = chrono::DateTime::parse_from_rfc3339(occurred_at)
                        .ok()
                        .map(|dt| dt.timestamp_millis());
                    let delta_str = match (ts_ms, prev_ts_ms) {
                        (Some(ts), Some(prev)) => {
                            format!("  (+{:.1}s)", (ts - prev) as f64 / 1000.0)
                        }
                        _ => String::new(),
                    };
                    log_report!(
                        "    {}  {:<35}{}\n",
                        occurred_at,
                        event_type,
                        delta_str
                    );
                    prev_ts_ms = ts_ms;
                }

                // Retrieve timestamps for each phase boundary
                let get_ts = |target: &str| -> Option<String> {
                    events
                        .iter()
                        .find(|(_, et, _, _)| et == target)
                        .map(|(ts, _, _, _)| ts.clone())
                };

                let t_detected   = get_ts("interruption_detected");
                let t_checkpoint = get_ts("checkpoint_completed");
                let t_dispatched = get_ts("restart_dispatched");
                let t_terminal   = get_ts("recovery_completed")
                    .or_else(|| get_ts("degraded_resume"))
                    .or_else(|| get_ts("no_workers_remaining"))
                    .or_else(|| get_ts("recovery_failed"));

                log_report!("\n  Phase Durations:\n");

                if let (Some(t1), Some(t2)) = (&t_detected, &t_checkpoint) {
                    if let Some(d) = phase_duration_secs(t1, t2) {
                        log_report!(
                            "    interruption_detected  ->  checkpoint_completed  : {:>8.1} s  (detection + checkpoint write)\n",
                            d
                        );
                    }
                }

                if let (Some(t1), Some(t2)) = (&t_checkpoint, &t_dispatched) {
                    if let Some(d) = phase_duration_secs(t1, t2) {
                        log_report!(
                            "    checkpoint_completed   ->  restart_dispatched    : {:>8.1} s  (node recovery + dispatch)\n",
                            d
                        );
                    }
                } else if t_checkpoint.is_some() && t_dispatched.is_none() {
                    log_report!(
                        "    checkpoint_completed   ->  restart_dispatched    :      N/A  (restart_dispatched not recorded — re-run needed)\n"
                    );
                }

                if let (Some(t1), Some(t2)) = (&t_dispatched, &t_terminal) {
                    if let Some(d) = phase_duration_secs(t1, t2) {
                        log_report!(
                            "    restart_dispatched     ->  terminal event        : {:>8.1} s  (restart + remaining compute)\n",
                            d
                        );
                    }
                }

                if let (Some(t1), Some(t2)) = (&t_detected, &t_terminal) {
                    if let Some(total) = phase_duration_secs(t1, t2) {
                        log_report!("    {}\n", "-".repeat(58));
                        log_report!(
                            "    Total recovery time                               : {:>8.1} s\n",
                            total
                        );
                    }
                }

                // Restart output (filtered)
                let restart_out = events
                    .iter()
                    .find(|(_, et, _, _)| et == "restart_completed")
                    .and_then(|(_, _, _, d)| extract_restart_output(d.as_deref()));
                if let Some(ref raw_out) = restart_out {
                    log_report!("\n  -- Restart Output --\n");
                    for line in filter_ssm_output(raw_out).lines() {
                        log_report!("  {}\n", line);
                    }
                    log_report!("  --------------------\n");
                }

                // Machine-readable metrics block (for scripts / table generation)
                log_report!("\n  [METRICS]\n");
                log_report!("  cycle_id={}\n", cycle_id);
                log_report!("  node_ip={}\n", node_ip);
                log_report!("  strategy={}\n", ft_strategy);
                if let (Some(t1), Some(t2)) = (&t_detected, &t_checkpoint) {
                    if let Some(d) = phase_duration_secs(t1, t2) {
                        log_report!("  phase1_detection_to_checkpoint_s={:.3}\n", d);
                    }
                }
                if let (Some(t1), Some(t2)) = (&t_checkpoint, &t_dispatched) {
                    if let Some(d) = phase_duration_secs(t1, t2) {
                        log_report!("  phase2_checkpoint_to_dispatch_s={:.3}\n", d);
                    }
                }
                if let (Some(t1), Some(t2)) = (&t_dispatched, &t_terminal) {
                    if let Some(d) = phase_duration_secs(t1, t2) {
                        log_report!("  phase3_dispatch_to_done_s={:.3}\n", d);
                    }
                }
                if let (Some(t1), Some(t2)) = (&t_detected, &t_terminal) {
                    if let Some(d) = phase_duration_secs(t1, t2) {
                        log_report!("  total_recovery_s={:.3}\n", d);
                    }
                }
                log_report!("  [/METRICS]\n");
            }

            log_report!("\n{}\n\n", sep);

            println!(
                "Fault-tolerance summary: {} recovery cycle(s) — ids: {}",
                ordered_cycle_ids.len(),
                ordered_cycle_ids.join(", ")
            );
        }
    }

    if let Some(pb) = &watcher_progress_bar {
        pb.finish_with_message("[watcher] stopped");
    }
    if let Some(pb) = &auto_failure_progress_bar {
        pb.finish_with_message("[auto-failure] done");
    }
    info!("All tasks completed!");

    if let Some(handle) = watcher_handle {
        println!("Tasks complete. Stopping watcher...");
        handle.abort();
    }

    info!("Result report saved at '{:?}'", report_path);
    println!(
        "All logs and results were saved at '{}'",
        report_path.display()
    );

    Ok(())
}
