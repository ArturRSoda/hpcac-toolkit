CREATE TABLE interruption_events (
    id VARCHAR(32) PRIMARY KEY,
    cluster_id VARCHAR(32) NOT NULL,
    node_id VARCHAR(32) NOT NULL,
    node_private_ip TEXT NOT NULL,
    event_type TEXT NOT NULL,
        -- 'interruption_detected'
        -- 'checkpoint_completed'
        -- 'node_drained'
        -- 'recovery_started'     (Policy A only)
        -- 'recovery_completed'   (Policy A only)
        -- 'degraded_resume'      (Policy B only)
    occurred_at TEXT NOT NULL,
    details TEXT,
    FOREIGN KEY (cluster_id) REFERENCES clusters(id),
    FOREIGN KEY (node_id) REFERENCES nodes(id)
);
