use super::interface::{AwsClusterContext, AwsInterface};
use crate::database::models::{
    Cluster, ClusterState, InstanceCreationFailurePolicy, InterruptionEvent, Node,
};
use crate::integrations::CloudResourceManager;
use crate::utils;

use std::collections::{HashMap, HashSet};

use anyhow::{Result, bail};
use sqlx::sqlite::SqlitePool;
use tokio::time::{Duration, sleep};
use tracing::info;
use tracing::error;
use tracing::warn;

const MAX_MIGRATION_ATTEMPTS: usize = 3;

fn node_index_from_private_ip(private_ip: &str) -> Option<usize> {
    let last_octet = private_ip.rsplit('.').next()?.parse::<usize>().ok()?;
    (last_octet >= 10).then_some(last_octet - 10)
}

fn is_active_instance_state(state: &str) -> bool {
    state == "running" || state == "pending"
}

fn is_terminated_like_instance_state(state: &str) -> bool {
    matches!(state, "terminated" | "shuttingdown" | "shutting_down" | "shutting-down")
}

impl AwsInterface {
    async fn find_cluster_instance_state_by_private_ip(
        &self,
        context: &AwsClusterContext,
        private_ip: &str,
    ) -> Result<Option<(String, String)>> {
        let describe_instances_response = context
            .ec2_client
            .describe_instances()
            .filters(
                aws_sdk_ec2::types::Filter::builder()
                    .name("private-ip-address")
                    .values(private_ip)
                    .build(),
            )
            .filters(context.cluster_id_filter.clone())
            .send()
            .await?;

        for reservation in describe_instances_response.reservations() {
            for instance in reservation.instances() {
                if let Some(instance_id) = instance.instance_id() {
                    let state_name = instance
                        .state()
                        .and_then(|state| state.name())
                        .map(|state| format!("{:?}", state).to_lowercase())
                        .unwrap_or_else(|| "unknown".to_string());
                    return Ok(Some((instance_id.to_string(), state_name)));
                }
            }
        }

        Ok(None)
    }
}

impl CloudResourceManager for AwsInterface {
    async fn spawn_cluster(
        &self,
        pool: &SqlitePool,
        cluster: Cluster,
        nodes: Vec<Node>,
    ) -> Result<()> {

        let attempts: usize = cluster.migration_attempts as usize;
        if attempts >= MAX_MIGRATION_ATTEMPTS {
            bail!("\nMaximum migration of {} attempts reached for cluster '{}'.", attempts, cluster.display_name);
        }

        eprintln!("\n ATTEMPT: {}", attempts+1);

        let mut context = self.create_cluster_context(&cluster)?;
        let mut steps = 9 + (6 * nodes.len());
        if cluster.use_node_affinity {
            steps += 1;
        }
        if cluster.use_elastic_file_system {
            steps += 4 + (2 * nodes.len()) + nodes.len();
        } else {
            steps += nodes.len();
        }

        let spawning_message = format!("Spawning Cluster '{}'...", cluster.display_name);
        info!(spawning_message);

        let new_state = match cluster.state {
            ClusterState::Running => ClusterState::Restoring,
            _ => ClusterState::Spawning,
        };
        cluster.update_state(pool, new_state).await?;

        let multi = utils::ProgressTracker::create_multi();
        let main_progress =
            utils::ProgressTracker::add_to_multi(&multi, steps as u64, Some(&spawning_message));
        let operation_spinner =
            utils::ProgressTracker::new_indeterminate(&multi, "Initializing...");

        /*
         * AWS CLUSTER CLOUD RESOURCE CREATION CYCLE
         *
         * 1. (conditional) Request EFS device
         * 2. Create VPC
         * 3. Create Subnet
         * 4. Create Internet Gateway
         * 5. Create Route Table (and Routing Rules and Internet Gateway and Subnet attachments)
         * 6. Create Security Groups (and attach all of them to the VPC)
         * 7. Create IAM Role and attach Trust Policies
         * 8. Create IAM Profile and assume IAM Role
         * 9. (conditional) Wait for EFS device to be ready
         * 10. (conditional) Request EFS mount target
         * 11. Create SSH Key Pair
         * 12. (conditional) Create Placement Group
         * 13. for each node {
         *     13.1. Create ENI device
         *     13.2. Create Elastic IP
         *     13.3. Associate Elastic IP with ENI device
         * }
         * 14. for each node {
         *     14.1. Request EC2 instance creation
         * }
         * 15. Wait for all EC2 instances to be ready
         * 16. (conditional) Wait for EFS mount target to be ready
         * 17. Wait for SSM agents to be ready on all instances
         * 18. (conditional) Attach EC2 Instances to EFS mount target using SSM
         * 19. (conditional) Dispatch EC2 Instances initialization commands
         */

        // 1. Request EFS device creation...
        if cluster.use_elastic_file_system {
            operation_spinner
                .update_message("Requesting Elastic File System (EFS) device creation...");
            context.efs_device_id = Some(
                self.request_elastic_file_system_device_creation(&context)
                    .await?,
            );
            main_progress.inc(1);
        }

        // 2. Create VPC
        operation_spinner.update_message("Creating Virtual Private Cloud (VPC)...");
        context.vpc_id = Some(self.ensure_vpc(&context).await?);
        main_progress.inc(1);

        // 3. Create Subnet
        operation_spinner.update_message("Creating Subnet...");
        context.subnet_id = Some(self.ensure_subnet(&context).await?);
        main_progress.inc(1);

        // 4. Create Internet Gateway
        operation_spinner.update_message("Creating Internet Gateway...");
        context.gateway_id = Some(self.ensure_internet_gateway(&context).await?);
        main_progress.inc(1);

        // 5. Create Route Table
        operation_spinner.update_message("Creating Route Table and Routing Rules...");
        context.route_table_id = Some(self.ensure_route_table(&context).await?);
        main_progress.inc(1);

        // 6. Create Security Groups
        operation_spinner.update_message("Creating Security Group and Security Rules...");
        context.security_group_ids = self.ensure_security_group(&context).await?;
        main_progress.inc(1);

        // 7. Create IAM Role and attach Trust Policies
        operation_spinner.update_message("Creating IAM Role and Trust Policies...");
        self.ensure_iam_role_and_trust_policies(&context).await?;
        main_progress.inc(1);

        // 8. Create IAM Profile and assume IAM Role
        operation_spinner.update_message("Creating IAM Profile and assuming IAM Roles...");
        self.ensure_iam_profile(&context).await?;
        main_progress.inc(1);

        if cluster.use_elastic_file_system {
            // 9. Wait for EFS device to be ready...
            operation_spinner
                .update_message("Waiting for Elastic File System (EFS) device to be ready...");
            self.wait_for_elastic_file_system_device_to_be_ready(&context)
                .await?;
            main_progress.inc(1);

            // 10. Request EFS mount target creation...
            operation_spinner
                .update_message("Requesting Elastic File System (EFS) mount target creation...");
            context.efs_mount_target_id = Some(
                self.request_elastic_file_system_mount_target_creation(&context)
                    .await?,
            );
            main_progress.inc(1);
        }

        // 11. Create SSH Key Pair
        operation_spinner.update_message("Importing the SSH key pair...");
        context.ssh_key_id = Some(self.ensure_ssh_key(&context).await?);
        main_progress.inc(1);

        // 12. Create Placement Group
        if context.use_node_affinity {
            operation_spinner.update_message("Creating a Placement Group...");
            context.placement_group_name_actual =
                Some(self.ensure_placement_group(&context).await?);
            main_progress.inc(1);
        }

        // Sort nodes so the head is always provisioned first
        let mut nodes = nodes;
        nodes.sort_by_key(|n| if n.role == "head" { 0usize } else { 1usize });

        // 13. Create ENI devices, Elastic IPs, and associate them
        for (node_index, node) in nodes.iter().enumerate() {
            // 13.1. Create ENI device
            operation_spinner.update_message(&format!(
                "Creating {} of {} Elastic Network Interface (ENI) devices",
                node_index + 1,
                nodes.len()
            ));
            let eni_id = self
                .ensure_elastic_network_interface(&context, node_index)
                .await?;
            context
                .elastic_network_interface_ids
                .insert(node_index, eni_id.clone());
            main_progress.inc(1);

            // 13.2. Create Elastic IP
            operation_spinner.update_message(&format!(
                "Allocating {} of {} Elastic IPs",
                node_index + 1,
                nodes.len()
            ));
            let eip_id = self.ensure_elastic_ip(&context, node_index).await?;
            context.elastic_ip_ids.insert(node_index, eip_id.clone());
            main_progress.inc(1);

            // 13.3. Attach Elastic IP to ENI device
            operation_spinner.update_message(&format!(
                "Associating allocated Elastic IP {} with Elastic Network Interface (ENI) device {}...",
                eip_id, eni_id
            ));
            let node_public_ip = self
                .associate_elastic_ip_with_network_interface(&context, &eip_id, &eni_id)
                .await?;
            context
                .elastic_ips
                .insert(node_index, node_public_ip.clone());
            let node_private_ip = context.network_interface_private_ip(node_index);
            node.set_ips(pool, &node_private_ip, &node_public_ip)
                .await?;
            main_progress.inc(1);
        }

        // 14. Request EC2 Instances
        match cluster.state {
            // Sleep for 20s to give time for the IAM Profile to be propagated the first time the
            // Cluster is created
            ClusterState::Pending | ClusterState::Spawning => {
                operation_spinner
                    .update_message("Giving time for the IAM Profile to be propagated...");
                sleep(Duration::from_secs(20)).await;
            }
            _ => {}
        }

        for (node_index, node) in nodes.iter().enumerate() {
            // 14.1. Request EC2 instance creation...
            operation_spinner.update_message(&format!(
                "Requesting {} of {} EC2 Instances (type='{}')",
                node_index + 1,
                nodes.len(),
                node.instance_type
            ));
            let instance_id = match self
                .request_elastic_compute_instance_creation(&context, node, node_index)
                .await {
                    Ok(id) => id,
                    Err(e) => {
                        match cluster.on_instance_creation_failure.as_ref().unwrap() {
                            InstanceCreationFailurePolicy::Cancel => {
                                // Stop the progress bars before printing errors
                                operation_spinner.finish_with_message("Error occurred, cleaning up...");
                                main_progress.finish_with_message("Cluster creation failed.");
                                
                                // Print the error
                                eprintln!("Failed to create instance for node {}: {:#}", node_index+1, e);
                                eprintln!("\nDestroying and canceling the cluster");

                                // Attempt to terminate/cleanup the cluster
                                let cleanup_result = self.terminate_cluster(pool, cluster.clone(), nodes.clone()).await;
                                if let Err(cleanup_err) = cleanup_result {
                                    error!("Failed to cleanup cluster after instance creation failure: {:?}", cleanup_err);
                                }
                            },
                            InstanceCreationFailurePolicy::OnDemand => {
                                // Stop progress bars to report status
                                operation_spinner.finish_with_message("Spot allocation failed, switching strategy...");
                                main_progress.finish_with_message("Cluster creation interrupted.");

                                warn!(
                                    "Failed to create Spot instance for node {} (Allocation: '{}'). Error: {:#}", 
                                    node_index + 1, 
                                    nodes[node_index].allocation_mode, 
                                    e
                                );

                                eprintln!(
                                    "\nCreation failed. Switching Node {} to 'on-demand' and retrying (keeping Networking/IAM intact)...", 
                                    node_index + 1
                                );

                                // Terminate only the instances, not the whole cluster
                                if let Err(term_err) = self.request_termination_of_all_elastic_compute_instances(&context).await {
                                    error!("Failed to terminate instances during fallback: {:?}", term_err);
                                    // If we can't clean up instances, we must fail hard
                                    return Err(term_err); 
                                }
                                
                                // Wait for them to actually shut down so we can reuse the ENIs/Volumes cleanly
                                if let Err(wait_err) = self.wait_for_all_elastic_compute_instances_to_be_terminated(&context).await {
                                    error!("Timeout waiting for instances to terminate: {:?}", wait_err);
                                    return Err(wait_err);
                                }

                                // We must update the database so if the process crashes, we don't retry Spot next time.
                                let update_query = sqlx::query!(
                                    "UPDATE nodes SET allocation_mode = 'on-demand' WHERE id = ?",
                                    nodes[node_index].id
                                )
                                .execute(pool)
                                .await;

                                if let Err(db_err) = update_query {
                                    error!("Failed to persist allocation mode change to DB: {:?}", db_err);
                                    bail!("Database error during fallback");
                                }

                                // Update local state for the recursion
                                let mut new_cluster = cluster.clone();
                                new_cluster.migration_attempts += 1;
                                
                                let mut new_nodes = nodes.clone();
                                new_nodes[node_index].allocation_mode = "on-demand".to_string();

                                return Box::pin(self.spawn_cluster(
                                    pool,
                                    new_cluster,
                                    new_nodes,
                                )).await;
                            },
                            InstanceCreationFailurePolicy::Migrate => {
                                // Stop the progress bars before printing errors
                                operation_spinner.finish_with_message("Error occurred, cleaning up...");
                                main_progress.finish_with_message("Cluster creation failed.");
                                
                                // Print the error
                                eprintln!("Failed to create instance for node {} in availability zone {}: {:#}", node_index+1, cluster.availability_zone, e);
                                eprintln!("\nDestroying cluster and retrying in a different availability zone.");

                                // Attempt to terminate/cleanup the cluster
                                let cleanup_result = self.terminate_cluster(pool, cluster.clone(), nodes.clone()).await;
                                if let Err(cleanup_err) = cleanup_result {
                                    bail!("Failed to cleanup cluster after instance creation failure: {:?}", cleanup_err);
                                }

                                // Split the tried_zones string into a Vec<&str>, removing empty entries
                                let mut tried_zones: Vec<_> = cluster.tried_zones
                                    .as_deref()
                                    .unwrap_or("")
                                    .split(',')
                                    .filter(|s| !s.is_empty())
                                    .collect();
                                tried_zones.push(cluster.availability_zone.as_str());

                                // Get all available zones in the region
                                let all_zones = self.get_all_availability_zones(&context.ec2_client, &cluster.region).await?;

                                // Filter out the current zone that failed
                                let alternative_zones: Vec<_> = all_zones.into_iter()
                                    .filter(|z| !tried_zones.contains(&z.as_str()))
                                    .collect();
                                if alternative_zones.is_empty() {
                                    bail!("No alternative availability zones available in region {}", cluster.region);
                                }

                                // Try next alternative zone
                                if let Some(zone) = alternative_zones.first() {
                                    eprintln!("Attempting to create cluster in zone {}", zone);

                                    // Update the cluster with the new zone
                                    let mut new_cluster = cluster.clone();
                                    new_cluster.tried_zones = Some(tried_zones.join(","));
                                    new_cluster.availability_zone = zone.clone();
                                    new_cluster.migration_attempts += 1;

                                    // Try creating the cluster in the new zone
                                    return Box::pin(self.spawn_cluster(
                                        pool,
                                        new_cluster,
                                        nodes.clone(),
                                    )).await;
                                }

                                // If get here, all zones failed
                                bail!("Failed to find an availability zone with sufficient capacity");
                            }
                        }
                        return Err(e);
                    }
                };
            context.ec2_instance_ids.insert(node_index, instance_id);
            main_progress.inc(1);
        }

        // 15. Wait for all EC2 Instances to be available
        operation_spinner.update_message("Waiting for all EC2 Instances to be available...");
        sleep(Duration::from_secs(5)).await;
        self.wait_for_all_elastic_compute_instances_to_be_available(&context)
            .await?;
        main_progress.inc(1);

        if cluster.use_elastic_file_system {
            // 16. Wait for EFS mount target to be ready
            operation_spinner.update_message("Waiting for the EFS mount target to be ready...");
            self.wait_for_elastic_file_system_mount_target_to_be_ready(&context)
                .await?;
            main_progress.inc(1);

            // 17. Wait for SSM agents to be ready on all instances (for EFS mounting)
            for (node_index, _) in nodes.iter().enumerate() {
                let node_instance_id = &context.ec2_instance_ids[&node_index];
                operation_spinner.update_message(&format!(
                    "Waiting for SSM agent readiness on Node {} of {} (for EFS mounting)...",
                    node_index + 1,
                    nodes.len()
                ));
                self.wait_for_ssm_agent_ready(&context, node_instance_id, Duration::from_secs(300))
                    .await?;
                main_progress.inc(1);
            }

            // 18. Attach EC2 Instances to EFS mount target using SSM
            let efs_mount_target_ip = self
                .fetch_elastic_file_system_mount_target_ip(&context)
                .await?;
            let mut ssm_command_ids: HashMap<usize, String> = HashMap::new();
            for (node_index, _) in nodes.iter().enumerate() {
                let op_msg = format!(
                    "Requesting EFS Mount Target attachment for Node {} of {}...",
                    node_index + 1,
                    nodes.len()
                );
                operation_spinner.update_message(&op_msg);
                match nodes[node_index].was_efs_configured {
                    true => {
                        info!(
                            "Skipping Node {} of {} (already configured for EFS)...",
                            node_index + 1,
                            nodes.len()
                        );
                    }
                    false => {
                        let node_instance_id = &context.ec2_instance_ids[&node_index];
                        let efs_attach_script = format!(
                            r#"
if command -v dnf >/dev/null 2>&1; then
    sudo dnf install -y nfs-utils
else
    sudo yum install -y nfs-utils
fi
sudo mkdir -p /shared
i=1
max_attempts=30
while [ "$i" -le "$max_attempts" ]; do
   echo "EFS mount attempt $i..."
   if sudo mount -t nfs4 -o nfsvers=4.1 {}:/ /shared; then
       echo "EFS mount successful!"
       break
   else
       echo "EFS mount failed, waiting 10 seconds for DNS propagation..."
       sleep 10
       i=$((i + 1))
   fi
done
if [ "$i" -gt "$max_attempts" ]; then
    echo "ERROR: EFS mount failed after ${{max_attempts}} attempts"
    exit 1
fi
sudo chown ec2-user:ec2-user /shared
echo "EFS mount and setup complete!"
"#,
                            efs_mount_target_ip
                        );
                        let ssm_command_id = self
                            .create_ssm_command(&context, node_instance_id, efs_attach_script)
                            .await?;
                        ssm_command_ids.insert(node_index, ssm_command_id);
                    }
                }
                main_progress.inc(1);
            }
            for (node_index, _) in nodes.iter().enumerate() {
                let max_wait_time = Duration::from_secs(5 * 60);
                let poll_interval = Duration::from_secs(15);
                let op_msg = format!(
                    "Waiting for Node {} of {} to attach to EFS Mount Target...",
                    node_index + 1,
                    nodes.len()
                );
                operation_spinner.update_message(&op_msg);
                match nodes[node_index].was_efs_configured {
                    true => {}
                    false => {
                        let node_instance_id = &context.ec2_instance_ids[&node_index];
                        self.poll_ssm_command_until_completion(
                            &context,
                            &ssm_command_ids[&node_index],
                            node_instance_id,
                            max_wait_time,
                            poll_interval,
                        )
                        .await?;
                        nodes[node_index]
                            .set_efs_configuration_state(pool, true)
                            .await?;
                    }
                }
                main_progress.inc(1);
            }
        } else {
            // Wait for SSM agents to be ready for init commands (when not using EFS)
            for (node_index, _) in nodes.iter().enumerate() {
                let node_instance_id = &context.ec2_instance_ids[&node_index];
                operation_spinner.update_message(&format!(
                    "Waiting for SSM agent readiness on Node {} of {} (for init commands)...",
                    node_index + 1,
                    nodes.len()
                ));
                self.wait_for_ssm_agent_ready(&context, node_instance_id, Duration::from_secs(300))
                    .await?;
                main_progress.inc(1);
            }
        }

        // 19. Dispatch EC2 Instance initialization commands
        // TODO: Add logic to track/skip individual commands
        let mut ssm_init_command_ids: HashMap<usize, String> = HashMap::new();
        let private_key_content = match std::fs::read_to_string(&cluster.private_ssh_key_path) {
            Ok(content) => content,
            Err(e) => {
                bail!(
                    "Failed to read private SSH key file '{}': {}",
                    cluster.private_ssh_key_path,
                    e
                );
            }
        };

        let local_key_path = std::path::Path::new(&cluster.private_ssh_key_path);
        let key_filename = local_key_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("id_rsa"); // Fallback to id_rsa if path is weird
        let worker_count = nodes.len().saturating_sub(1);

        for (node_index, node) in nodes.iter().enumerate() {
            let mut node_init_commands = node.get_init_commands(pool).await?;
            let env_block = format!(
                "export HPCAC_NODE_ROLE={role} \
HPCAC_NODE_INDEX={idx} \
HPCAC_HEAD_PRIVATE_IP=10.0.0.10 \
HPCAC_NODE_COUNT={count} \
HPCAC_WORKER_COUNT={worker_count} \
HPCAC_USE_EFS={use_efs} \
HPCAC_EFS_MOUNT=/shared",
                role = node.role,
                idx = node_index,
                count = nodes.len(),
                worker_count = worker_count,
                use_efs = cluster.use_elastic_file_system,
            );

            let ssh_key_setup_script = format!(
                r#"echo "Setting up private SSH key..." && \
mkdir -p ~/.ssh && \
rm -f ~/.ssh/*.pub && \
cat > ~/.ssh/{0} << 'PRIVATE_KEY_EOF'
{1}
PRIVATE_KEY_EOF
chmod 600 ~/.ssh/{0} && \
chown ec2-user:ec2-user ~/.ssh/{0} && \
echo "Private SSH key successfully installed at ~/.ssh/{0}""#,
                key_filename,
                private_key_content
            );

            node_init_commands.insert(0, env_block);
            node_init_commands.insert(1, ssh_key_setup_script);
            let op_msg = format!(
                "Dispatching init script for Node {} of {}...",
                node_index + 1,
                nodes.len()
            );
            operation_spinner.update_message(&op_msg);
            if node_init_commands.is_empty() {
                continue;
            }
            let node_instance_id = &context.ec2_instance_ids[&node_index];
            let node_init_script = node_init_commands.join(" && ");
            let ssm_command_id = self
                .create_ssm_command(&context, node_instance_id, node_init_script)
                .await?;
            ssm_init_command_ids.insert(node_index, ssm_command_id);
            main_progress.inc(1);
        }
        for (node_index, ssm_init_command_id) in ssm_init_command_ids.iter() {
            let node_instance_id = &context.ec2_instance_ids[node_index];
            let max_wait_time = Duration::from_secs(15 * 60);
            let poll_interval = Duration::from_secs(15);
            self.poll_ssm_command_until_completion(
                &context,
                ssm_init_command_id,
                node_instance_id,
                max_wait_time,
                poll_interval,
            )
            .await?;
        }

        cluster.update_state(pool, ClusterState::Running).await?;

        operation_spinner.finish_with_message("All Cloud operations completed");
        main_progress.finish_with_message(&format!(
            "Cluster '{}' spawned successfully!",
            cluster.display_name
        ));
        println!("\nCluster spawn completed successfully. You can access your nodes using:");
        for (node_index, ip_address) in context.elastic_ips.iter() {
            println!(
                "Node '10.0.0.{}': ssh ec2-user@{}",
                node_index + 10,
                ip_address
            );
        }
        Ok(())
    }

    async fn terminate_cluster(
        &self,
        pool: &SqlitePool,
        cluster: Cluster,
        nodes: Vec<Node>,
    ) -> Result<()> {
        let context = self.create_cluster_context(&cluster)?;
        let mut steps = 10;
        steps += 2 * nodes.len();
        if cluster.use_node_affinity {
            steps += 1;
        }
        if cluster.use_elastic_file_system {
            steps += 4;
        }

        let terminating_message = format!("Terminating Cluster '{}'...", cluster.display_name);
        info!(terminating_message);

        cluster
            .update_state(pool, ClusterState::Terminating)
            .await?;
        for node in nodes.iter() {
            node.set_efs_configuration_state(pool, false).await?;
        }

        let multi = utils::ProgressTracker::create_multi();
        let main_progress =
            utils::ProgressTracker::add_to_multi(&multi, steps as u64, Some(&terminating_message));
        let operation_spinner =
            utils::ProgressTracker::new_indeterminate(&multi, "Initializing...");

        /*
         * AWS CLUSTER CLOUD RESOURCE DESTRUCTION CYCLE
         *
         * 1. (optional) Request EFS mount target deletion
         * 2. Request termination of all EC2 Instances
         * 3. (optional) Wait for EFS mount target to be deleted
         * 4. (optional) Request EFS device deletion
         * 5. Wait for all EC2 instances to be terminated
         * 6. for each node {
         *    6.1. Dissociate from ENI device and deallocate Elastic IP
         *    6.2. Destroy ENI device
         * }
         * 7. (optional) Destroy Placement Group
         * 8. Destroy SSH Key Pair
         * 9. Destroy Security Groups
         * 10. Destroy IAM Profile
         * 11. Destroy IAM Role and Trust Policies
         * 12. Destroy Route Table
         * 13. Destroy Internet Gateway
         * 14. Destroy Subnet
         * 15. Destroy VPC
         * 16. (optional) Wait for EFS device to be deleted
         */

        // 1. Request EFS mount target deletion
        if cluster.use_elastic_file_system {
            operation_spinner
                .update_message("Requesting Elastic File System (EFS) mount target deletion...");
            self.request_elastic_file_system_mount_target_deletion(&context)
                .await?;
            main_progress.inc(1);
        }

        // 2. Request termination of all EC2 Instances
        operation_spinner.update_message(&format!(
            "Requesting termination of {} Elastic Compute Instances...",
            nodes.len()
        ));
        self.request_termination_of_all_elastic_compute_instances(&context)
            .await?;
        main_progress.inc(nodes.len() as u64);

        if cluster.use_elastic_file_system {
            // 3. Wait for EFS mount target to be deleted
            operation_spinner
                .update_message("Waiting for Elastic File System (EFS) mount target deletion...");
            self.wait_for_elastic_file_system_mount_target_to_be_deleted(&context)
                .await?;
            main_progress.inc(1);

            // 4. Request EFS device deletion
            operation_spinner
                .update_message("Requesting Elastic File System (EFS) device deletion...");
            self.request_elastic_file_system_device_deletion(&context)
                .await?;
            main_progress.inc(1);
        }

        // 5. Wait for all instances to be terminated
        operation_spinner.update_message(&format!(
            "Waiting for {} Elastic Compute Instances to be terminated...",
            nodes.len()
        ));
        self.wait_for_all_elastic_compute_instances_to_be_terminated(&context)
            .await?;
        main_progress.inc(nodes.len() as u64);

        for (node_index, _node) in nodes.iter().enumerate() {
            // 6.1. Dissociate from ENI device and deallocate Elastic IP
            operation_spinner.update_message(&format!(
                "Destroying Elastic IP {}/{}",
                node_index + 1,
                nodes.len()
            ));
            self.cleanup_elastic_ip(&context, node_index).await?;
            main_progress.inc(1);

            // 6.2. Destroy ENI device
            operation_spinner.update_message(&format!(
                "Destroying Elastic Network Interface {}/{}",
                node_index + 1,
                nodes.len()
            ));
            self.cleanup_elastic_network_interface(&context, node_index)
                .await?;
            main_progress.inc(1);
        }

        // 7. Destroy Placement Group
        if context.use_node_affinity {
            operation_spinner.update_message("Destroying Placement Group...");
            self.cleanup_placement_group(&context).await?;
            main_progress.inc(1);
        }

        // 8. Destroy SSH Key Pair
        operation_spinner.update_message("Deregistering the SSH key pair...");
        self.cleanup_ssh_key(&context).await?;
        main_progress.inc(1);

        // 9. Destroy Security Groups
        operation_spinner.update_message("Destroying Security Rules and the Security Group...");
        self.cleanup_security_group(&context).await?;
        main_progress.inc(1);

        // 10. Destroy IAM Profile
        operation_spinner.update_message("Destroying IAM Profile...");
        self.cleanup_iam_profile(&context).await?;
        main_progress.inc(1);

        // 11. Destroy IAM Role
        operation_spinner.update_message("Destroying IAM Role and Trust Policies...");
        self.cleanup_trust_policies_and_iam_role(&context).await?;
        main_progress.inc(1);

        // 12. Destroy Route Table
        operation_spinner.update_message("Destroying Routing Rules and Route Table...");
        self.cleanup_route_table(&context).await?;
        main_progress.inc(1);

        // 13. Destroy Internet Gateway
        operation_spinner.update_message("Destroying Internet Gateway...");
        self.cleanup_internet_gateway(&context).await?;
        main_progress.inc(1);

        // 14. Destroy Subnet
        operation_spinner.update_message("Destroying Subnet...");
        self.cleanup_subnet(&context).await?;
        main_progress.inc(1);

        // 15. Destroy VPC
        operation_spinner.update_message("Destroying VPC...");
        self.cleanup_vpc(&context).await?;
        main_progress.inc(1);

        // 16. Wait for EFS device to be deleted
        if cluster.use_elastic_file_system {
            operation_spinner.update_message("Waiting for EFS device to be deleted...");
            self.wait_for_elastic_file_system_device_to_be_deleted(&context)
                .await?;
            main_progress.inc(1);
        }

        cluster.update_state(pool, ClusterState::Terminated).await?;

        operation_spinner.finish_with_message("All Cloud operations completed");
        main_progress.finish_with_message(&format!(
            "Cluster '{}' terminated successfully!",
            cluster.display_name
        ));
        println!("Cluster termination completed successfully!");
        Ok(())
    }

    async fn restore_cluster(
        &self,
        pool: &SqlitePool,
        cluster: Cluster,
        nodes: Vec<Node>,
    ) -> Result<()> {
        let context = self.create_cluster_context(&cluster)?;

        let mut indexed_nodes: Vec<(usize, Node, String)> = Vec::new();
        let mut nodes_without_index: Vec<Node> = Vec::new();
        let mut assigned_indexes: HashSet<usize> = HashSet::new();

        for node in nodes.iter().cloned() {
            if let Some(private_ip) = node.private_ip.clone() {
                if let Some(node_index) = node_index_from_private_ip(&private_ip) {
                    assigned_indexes.insert(node_index);
                    indexed_nodes.push((node_index, node, private_ip));
                    continue;
                }
            }
            nodes_without_index.push(node);
        }

        nodes_without_index.sort_by_key(|node| {
            (
                if node.role == "head" { 0usize } else { 1usize },
                node.id.clone(),
            )
        });

        let mut next_index = 0usize;
        for node in nodes_without_index {
            while assigned_indexes.contains(&next_index) {
                next_index += 1;
            }
            let expected_private_ip = context.network_interface_private_ip(next_index);
            assigned_indexes.insert(next_index);
            indexed_nodes.push((next_index, node, expected_private_ip));
            next_index += 1;
        }

        indexed_nodes.sort_by_key(|(node_index, node, _)| {
            (
                if node.role == "head" { 0usize } else { 1usize },
                *node_index,
            )
        });

        let all_nodes: Vec<Node> = indexed_nodes
            .iter()
            .map(|(_, node, _)| node.clone())
            .collect();

        println!(
            "Restoring Cluster '{}' (checking {} node slot(s))...",
            cluster.display_name,
            indexed_nodes.len()
        );
        cluster.update_state(pool, ClusterState::Restoring).await?;

        let mut restored_nodes = 0usize;
        let mut healthy_nodes = 0usize;
        let mut head_instance_id: Option<String> = None;
        let mut restored_worker_hosts: Vec<String> = Vec::new();

        for (node_index, node, expected_private_ip) in indexed_nodes {
            let role = node.role.clone();
            let slot_label = format!("{}:{}", role, expected_private_ip);
            let status = self
                .find_cluster_instance_state_by_private_ip(&context, &expected_private_ip)
                .await?;

            let mut needs_restore = true;
            if let Some((instance_id, state)) = status.clone() {
                if is_active_instance_state(&state) {
                    info!(
                        "[restore] Slot '{}' already has active instance '{}' (state={})",
                        slot_label, instance_id, state
                    );
                    println!(
                        "[restore] OK  node={} private_ip={} instance={} state={}",
                        role, expected_private_ip, instance_id, state
                    );
                    needs_restore = false;
                    healthy_nodes += 1;
                    if role == "head" {
                        head_instance_id = Some(instance_id);
                    }
                } else if !is_terminated_like_instance_state(&state) {
                    warn!(
                        "[restore] Slot '{}' has instance '{}' in state '{}'; terminating before restore",
                        slot_label, instance_id, state
                    );
                    self.terminate_elastic_compute_instance(&context, &instance_id)
                        .await?;

                    for _ in 0..24 {
                        sleep(Duration::from_secs(5)).await;
                        match self
                            .find_cluster_instance_state_by_private_ip(
                                &context,
                                &expected_private_ip,
                            )
                            .await?
                        {
                            None => break,
                            Some((_, s)) if is_terminated_like_instance_state(&s) => break,
                            _ => {}
                        }
                    }
                }
            }

            if needs_restore {
                info!(
                    "[restore] Recreating node slot '{}' at index {}",
                    slot_label, node_index
                );
                println!(
                    "[restore] FIX node={} private_ip={} action=respawn",
                    role, expected_private_ip
                );
                self.respawn_worker_node(
                    pool,
                    cluster.clone(),
                    node.clone(),
                    node_index,
                    all_nodes.clone(),
                    None,
                )
                .await?;
                if role == "worker" {
                    let worker_host = format!("ip-{}", expected_private_ip.replace('.', "-"));
                    restored_worker_hosts.push(worker_host);
                }
                restored_nodes += 1;
            }
        }

        if let Some(head_id) = &head_instance_id {
            for worker_host in &restored_worker_hosts {
                println!("[restore] Resuming '{}' in Slurm", worker_host);
                let resume_cmd = format!(
                    "sudo /opt/slurm-24.05.4/bin/scontrol update NodeName={} State=RESUME",
                    worker_host
                );
                let cmd_id = self
                    .create_ssm_command(&context, head_id, resume_cmd)
                    .await?;
                self.poll_ssm_command_until_completion(
                    &context,
                    &cmd_id,
                    head_id,
                    Duration::from_secs(30),
                    Duration::from_secs(5),
                )
                .await?;
            }
        }

        cluster.update_state(pool, ClusterState::Running).await?;
        let updated_nodes = cluster.get_nodes(pool).await?;
        println!(
            "Restore complete for cluster '{}': {} healthy, {} restored.",
            cluster.display_name, healthy_nodes, restored_nodes
        );
        println!("\nYou can access your nodes using:");
        for node in &updated_nodes {
            if let (Some(private_ip), Some(public_ip)) = (&node.private_ip, &node.public_ip) {
                println!("Node '{}': ssh ec2-user@{}", private_ip, public_ip);
            }
        }

        Ok(())
    }

    async fn simulate_cluster_failure(
        &self,
        pool: &SqlitePool,
        cluster: Cluster,
        node_private_ip: &str,
        warning_time_secs: u64,
    ) -> Result<()> {
        let context = self.create_cluster_context(&cluster)?;
        let db_node = Node::fetch_by_private_ip(pool, node_private_ip).await?;

        if warning_time_secs > 0 {
            if let Some(node) = &db_node {
                InterruptionEvent::new(
                    &cluster.id,
                    &node.id,
                    node_private_ip,
                    "interruption_warning",
                    Some(&format!(
                        "simulated_warning_time_secs={}",
                        warning_time_secs
                    )),
                )
                .insert(pool)
                .await?;
            }

            println!(
                "Issued simulated interruption warning for node '{}' ({}s before termination)",
                node_private_ip, warning_time_secs
            );
            sleep(Duration::from_secs(warning_time_secs)).await;
        }

        match self
            .find_elastic_compute_instance_by_private_ip(&context, node_private_ip)
            .await?
        {
            Some(id) => {
                println!("Terminating instance with IP: '{}'", node_private_ip);
                self.terminate_elastic_compute_instance(&context, &id)
                    .await?;
            }
            None => {
                println!(
                    "Private IP: '{}' not found in Cluster '{}'",
                    node_private_ip, cluster.display_name
                );
                return Ok(());
            }
        }

        match db_node {
            Some(failed_node) => {
                failed_node.set_efs_configuration_state(pool, false).await?;
                InterruptionEvent::new(
                    &cluster.id,
                    &failed_node.id,
                    node_private_ip,
                    "interruption_termination_requested",
                    Some("simulated_test_failure"),
                )
                .insert(pool)
                .await?;
                println!(
                    "Requested termination for Instance '{}' (failure simulation)",
                    failed_node.id
                );
            }
            None => {
                println!(
                    "Couldn't find Node record (private_ip='{}') in the database",
                    node_private_ip
                );
            }
        }

        Ok(())
    }

    async fn respawn_worker_node(
        &self,
        pool: &SqlitePool,
        cluster: Cluster,
        node: Node,
        node_index: usize,
        all_nodes: Vec<Node>,
        replacement_allocation_mode: Option<String>,
    ) -> Result<()> {
        let mut context = self.create_cluster_context(&cluster)?;

        context.vpc_id = Some(self.ensure_vpc(&context).await?);
        context.subnet_id = Some(self.ensure_subnet(&context).await?);
        context.gateway_id = Some(self.ensure_internet_gateway(&context).await?);
        context.route_table_id = Some(self.ensure_route_table(&context).await?);
        context.security_group_ids = self.ensure_security_group(&context).await?;
        self.ensure_iam_role_and_trust_policies(&context).await?;
        self.ensure_iam_profile(&context).await?;

        if cluster.use_elastic_file_system {
            context.efs_device_id = Some(self.request_elastic_file_system_device_creation(&context).await?);
            self.wait_for_elastic_file_system_device_to_be_ready(&context)
                .await?;
            context.efs_mount_target_id =
                Some(self.request_elastic_file_system_mount_target_creation(&context).await?);
            self.wait_for_elastic_file_system_mount_target_to_be_ready(&context)
                .await?;
        }

        context.ssh_key_id = Some(self.ensure_ssh_key(&context).await?);
        if context.use_node_affinity {
            context.placement_group_name_actual = Some(self.ensure_placement_group(&context).await?);
        }

        let eni_id = self
            .ensure_elastic_network_interface(&context, node_index)
            .await?;
        context
            .elastic_network_interface_ids
            .insert(node_index, eni_id.clone());

        let eip_id = self.ensure_elastic_ip(&context, node_index).await?;
        context.elastic_ip_ids.insert(node_index, eip_id.clone());

        let node_public_ip = self
            .associate_elastic_ip_with_network_interface(&context, &eip_id, &eni_id)
            .await?;
        context.elastic_ips.insert(node_index, node_public_ip.clone());

        let node_private_ip = context.network_interface_private_ip(node_index);
        node.set_ips(pool, &node_private_ip, &node_public_ip).await?;

        let mut replacement_node = node.clone();
        if let Some(mode) = replacement_allocation_mode {
            replacement_node.allocation_mode = mode;
            sqlx::query!(
                "UPDATE nodes SET allocation_mode = ? WHERE id = ?",
                replacement_node.allocation_mode,
                replacement_node.id
            )
            .execute(pool)
            .await?;
        }

        let instance_id = self
            .request_elastic_compute_instance_creation(&context, &replacement_node, node_index)
            .await?;
        context.ec2_instance_ids.insert(node_index, instance_id);

        self.wait_for_all_elastic_compute_instances_to_be_available(&context)
            .await?;

        let node_instance_id = &context.ec2_instance_ids[&node_index];
        self.wait_for_ssm_agent_ready(&context, node_instance_id, Duration::from_secs(300))
            .await?;

        if cluster.use_elastic_file_system {
            let efs_mount_target_ip = self
                .fetch_elastic_file_system_mount_target_ip(&context)
                .await?;
            let efs_attach_script = format!(
                r#"
if command -v dnf >/dev/null 2>&1; then
    sudo dnf install -y nfs-utils
else
    sudo yum install -y nfs-utils
fi
sudo mkdir -p /shared
i=1
max_attempts=30
while [ "$i" -le "$max_attempts" ]; do
   echo "EFS mount attempt $i..."
   if sudo mount -t nfs4 -o nfsvers=4.1 {}:/ /shared; then
       echo "EFS mount successful!"
       break
   else
       echo "EFS mount failed, waiting 10 seconds for DNS propagation..."
       sleep 10
       i=$((i + 1))
   fi
done
if [ "$i" -gt "$max_attempts" ]; then
    echo "ERROR: EFS mount failed after ${{max_attempts}} attempts"
    exit 1
fi
sudo chown ec2-user:ec2-user /shared
echo "EFS mount and setup complete!"
"#,
                efs_mount_target_ip
            );
            let ssm_command_id = self
                .create_ssm_command(&context, node_instance_id, efs_attach_script)
                .await?;
            self.poll_ssm_command_until_completion(
                &context,
                &ssm_command_id,
                node_instance_id,
                Duration::from_secs(5 * 60),
                Duration::from_secs(15),
            )
            .await?;
            node.set_efs_configuration_state(pool, true).await?;
        }

        let private_key_content = std::fs::read_to_string(&cluster.private_ssh_key_path)?;
        let local_key_path = std::path::Path::new(&cluster.private_ssh_key_path);
        let key_filename = local_key_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("id_rsa");

        let mut node_init_commands = node.get_init_commands(pool).await?;
        let worker_count = all_nodes.iter().filter(|n| n.role == "worker").count();
        let env_block = format!(
            "export HPCAC_NODE_ROLE={role} \
HPCAC_NODE_INDEX={idx} \
HPCAC_HEAD_PRIVATE_IP=10.0.0.10 \
HPCAC_NODE_COUNT={count} \
HPCAC_WORKER_COUNT={worker_count} \
HPCAC_USE_EFS={use_efs} \
HPCAC_EFS_MOUNT=/shared",
            role = node.role,
            idx = node_index,
            count = all_nodes.len(),
            worker_count = worker_count,
            use_efs = cluster.use_elastic_file_system,
        );
        let ssh_key_setup_script = format!(
            r#"echo "Setting up private SSH key..." && \
mkdir -p ~/.ssh && \
rm -f ~/.ssh/*.pub && \
cat > ~/.ssh/{0} << 'PRIVATE_KEY_EOF'
{1}
PRIVATE_KEY_EOF
chmod 600 ~/.ssh/{0} && \
chown ec2-user:ec2-user ~/.ssh/{0} && \
echo "Private SSH key successfully installed at ~/.ssh/{0}""#,
            key_filename,
            private_key_content
        );
        node_init_commands.insert(0, env_block);
        node_init_commands.insert(1, ssh_key_setup_script);

        if !node_init_commands.is_empty() {
            let node_init_script = node_init_commands.join(" && ");
            let ssm_command_id = self
                .create_ssm_command(&context, node_instance_id, node_init_script)
                .await?;
            self.poll_ssm_command_until_completion(
                &context,
                &ssm_command_id,
                node_instance_id,
                Duration::from_secs(15 * 60),
                Duration::from_secs(15),
            )
            .await?;
        }

        Ok(())
    }
}
