use crate::database::models::{Cluster, ClusterState, ProviderConfig};
use crate::integrations::{cloud_interface::CloudResourceManager, providers::aws::AwsInterface};
use crate::utils;

use anyhow::{Result, bail};
use sqlx::sqlite::SqlitePool;

pub async fn restore(pool: &SqlitePool, cluster_id: &str, skip_confirmation: bool) -> Result<()> {
    let cluster = match Cluster::fetch_by_id(pool, cluster_id).await? {
        Some(cluster) => cluster,
        None => {
            bail!("Cluster (id='{}') not found", cluster_id);
        }
    };

    match cluster.state {
        ClusterState::Terminated => {
            bail!("Cluster '{}' is terminated. Spawn it again before restore.", cluster.display_name);
        }
        ClusterState::Pending => {
            bail!(
                "Cluster '{}' is pending and has not been provisioned yet.",
                cluster.display_name
            );
        }
        _ => {}
    }

    let provider_config =
        match ProviderConfig::fetch_by_id(pool, cluster.provider_config_id).await? {
            Some(config) => config,
            None => {
                bail!(
                    "ProviderConfig (id='{}') not found",
                    cluster.provider_config_id
                );
            }
        };

    let config_vars = provider_config.get_config_vars(pool).await?;
    let provider_id = provider_config.provider_id.clone();
    let cloud_interface = match provider_id.as_str() {
        "aws" => AwsInterface { config_vars },
        _ => {
            bail!(
                "Provider (id='{}') is currently not supported.",
                &provider_id
            )
        }
    };

    let nodes = cluster.get_nodes(pool).await?;
    println!(
        "Preparing restore for cluster '{}' ({} node definitions)",
        cluster.display_name,
        nodes.len()
    );

    if !utils::user_confirmation(
        skip_confirmation,
        "Do you want to restore failed/down instances for this cluster?",
    )? {
        return Ok(());
    }

    cloud_interface.restore_cluster(pool, cluster, nodes).await?;
    Ok(())
}
