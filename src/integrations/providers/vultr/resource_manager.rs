use crate::database::models::{Cluster, Node};
use crate::integrations::CloudResourceManager;

use anyhow::{Result, bail};
use sqlx::sqlite::SqlitePool;

use super::interface::VultrInterface;

impl CloudResourceManager for VultrInterface {
    async fn spawn_cluster(
        &self,
        _pool: &SqlitePool,
        _cluster: Cluster,
        _nodes: Vec<Node>,
    ) -> Result<()> {
        bail!("Not implemented")
    }

    async fn terminate_cluster(
        &self,
        _pool: &SqlitePool,
        _cluster: Cluster,
        _nodes: Vec<Node>,
    ) -> Result<()> {
        bail!("Not implemented")
    }

    async fn restore_cluster(
        &self,
        _pool: &SqlitePool,
        _cluster: Cluster,
        _nodes: Vec<Node>,
    ) -> Result<()> {
        bail!("Not implemented")
    }

    async fn simulate_cluster_failure(
        &self,
        _pool: &SqlitePool,
        _cluster: Cluster,
        _node_private_ip: &str,
        _warning_time_secs: u64,
    ) -> Result<()> {
        bail!("Not implemented")
    }

    async fn respawn_worker_node(
        &self,
        _pool: &SqlitePool,
        _cluster: Cluster,
        _node: Node,
        _node_index: usize,
        _all_nodes: Vec<Node>,
        _replacement_allocation_mode: Option<String>,
    ) -> Result<()> {
        bail!("Not implemented")
    }
}
