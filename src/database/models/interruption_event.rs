use anyhow::{bail, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use tracing::error;

use crate::utils::random::generate_id;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct InterruptionEvent {
    pub id: String,
    pub cluster_id: String,
    pub node_id: String,
    pub node_private_ip: String,
    pub event_type: String,
    pub occurred_at: String,
    pub details: Option<String>,
}

impl InterruptionEvent {
    pub fn new(
        cluster_id: &str,
        node_id: &str,
        node_private_ip: &str,
        event_type: &str,
        details: Option<&str>,
    ) -> Self {
        InterruptionEvent {
            id: generate_id(),
            cluster_id: cluster_id.to_string(),
            node_id: node_id.to_string(),
            node_private_ip: node_private_ip.to_string(),
            event_type: event_type.to_string(),
            occurred_at: Utc::now().to_rfc3339(),
            details: details.map(|s| s.to_string()),
        }
    }

    pub async fn insert(&self, pool: &SqlitePool) -> Result<()> {
        match sqlx::query!(
            r#"INSERT INTO interruption_events
               (id, cluster_id, node_id, node_private_ip, event_type, occurred_at, details)
               VALUES (?, ?, ?, ?, ?, ?, ?)"#,
            self.id,
            self.cluster_id,
            self.node_id,
            self.node_private_ip,
            self.event_type,
            self.occurred_at,
            self.details,
        )
        .execute(pool)
        .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("SQLx Error: {}", e);
                bail!("DB Operation Failure");
            }
        }
    }

    #[allow(dead_code)]
    pub async fn fetch_all_by_cluster_id(
        pool: &SqlitePool,
        cluster_id: &str,
    ) -> Result<Vec<InterruptionEvent>> {
        Ok(sqlx::query_as!(
            InterruptionEvent,
            r#"SELECT id as "id!", cluster_id as "cluster_id!", node_id as "node_id!",
                      node_private_ip as "node_private_ip!", event_type as "event_type!",
                      occurred_at as "occurred_at!", details
               FROM interruption_events
               WHERE cluster_id = ?
               ORDER BY occurred_at DESC"#,
            cluster_id,
        )
        .fetch_all(pool)
        .await?)
    }
}
