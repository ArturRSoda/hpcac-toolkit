# Phase 1 Plan — Node Roles and Hybrid Topology

**Goal:** Support exactly one on-demand head node and N spot worker nodes in the cluster
configuration, with DB persistence and head-first spawn ordering.

**Depends on:** Phase 0 AMI `ami-06af33e2399c52709` (all nodes use same image)

---

## Overview of Changes

| Layer | File | What changes |
|---|---|---|
| YAML schema | `cluster.example.yaml` | Add `role` field to node examples |
| Config parsing | `src/commands/cluster/create.rs` | Parse `role` in `NodeYaml`, validate constraints |
| DB model | `src/database/models/node.rs` | Add `role` field to `Node` struct and INSERT query |
| DB migration | `migrations/8__add_node_role.sql` | `ALTER TABLE nodes ADD COLUMN role TEXT` |
| Spawn ordering | `src/integrations/providers/aws/resource_manager.rs` | Sort nodes head-first before provisioning loop |

---

## Step 1 — DB Migration

**File:** `migrations/8__add_node_role.sql`

Add a `role` column to the `nodes` table. Two valid values: `head` and `worker`.
Default to `worker` so existing records without a role remain consistent.

```sql
ALTER TABLE nodes ADD COLUMN role TEXT NOT NULL DEFAULT 'worker';
```

---

## Step 2 — Update the Node DB Model

**File:** `src/database/models/node.rs`

Add `role: String` to the `Node` struct and include it in the `INSERT` query.

The struct currently is:
```rust
pub struct Node {
    pub id: String,
    pub cluster_id: String,
    pub instance_type: String,
    pub allocation_mode: String,
    pub burstable_mode: Option<String>,
    pub image_id: String,
    pub private_ip: Option<String>,
    pub public_ip: Option<String>,
    pub was_efs_configured: bool,
    pub was_ssh_configured: bool,
}
```

After change:
```rust
pub struct Node {
    pub id: String,
    pub cluster_id: String,
    pub role: String,           // "head" | "worker"
    pub instance_type: String,
    pub allocation_mode: String,
    pub burstable_mode: Option<String>,
    pub image_id: String,
    pub private_ip: Option<String>,
    pub public_ip: Option<String>,
    pub was_efs_configured: bool,
    pub was_ssh_configured: bool,
}
```

The `INSERT` in `Node::insert()` must include `role` in the column list and bind `self.role`.

---

## Step 3 — Update the YAML Config Parser

**File:** `src/commands/cluster/create.rs`

### 3a — Add `role` to `NodeYaml`

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct NodeYaml {
    role: String,                       // required: "head" | "worker"
    instance_type: String,
    allocation_mode: Option<String>,
    burstable_mode: Option<String>,
    image_id: String,
    init_commands: Option<Vec<String>>,
}
```

### 3b — Validate role constraints (after YAML is parsed, before any DB writes)

Add a validation block after `cluster_yaml` is deserialized:

1. **Every node must declare a role.**
   Serde will enforce this at parse time since `role` is not `Option<String>`.
   If missing from YAML, serde will return an error with a clear field name.

2. **Exactly one node must have `role: head`.**
   ```
   let head_count = cluster_yaml.nodes.iter().filter(|n| n.role == "head").count();
   if head_count != 1 {
       bail!("Cluster must have exactly one node with role 'head', found {}", head_count);
   }
   ```

3. **The head node must use `allocation_mode: on-demand`.**
   ```
   for node in &cluster_yaml.nodes {
       if node.role == "head" {
           let mode = node.allocation_mode.as_deref().unwrap_or("on-demand");
           if mode != "on-demand" {
               bail!("Head node must use allocation_mode 'on-demand', got '{}'", mode);
           }
       }
   }
   ```

4. **`role` must be one of the two known values.**
   ```
   for node in &cluster_yaml.nodes {
       if node.role != "head" && node.role != "worker" {
           bail!("Unknown node role '{}'. Valid values: head, worker", node.role);
       }
   }
   ```

### 3c — Pass `role` through to the `Node` struct

When constructing `Node` objects from `NodeYaml` entries, set `role: node_yaml.role.clone()`.

---

## Step 4 — Head-First Spawn Ordering

**File:** `src/integrations/providers/aws/resource_manager.rs`

The `spawn_cluster` function receives `nodes: Vec<Node>`. Currently it iterates over them
in YAML declaration order.

After this change, sort the slice so the head node always comes first. This ensures:
- Head EC2 instance is requested first.
- Head is reachable (and can run slurmctld) before workers try to join.

Add a sort before the ENI/EIP/EC2 provisioning loops (step 13 onward):

```rust
let mut nodes = nodes; // make mutable
nodes.sort_by_key(|n| if n.role == "head" { 0usize } else { 1usize });
```

---

## Step 5 — Update `cluster.example.yaml`

Add `role` to every node entry and add a realistic MANA cluster example:

```yaml
nodes:
  - role: head
    instance_type: t3.medium
    allocation_mode: on-demand
    burstable_mode: Unlimited
    image_id: ami-06af33e2399c52709

  - role: worker
    instance_type: m5.large
    allocation_mode: spot
    image_id: ami-06af33e2399c52709

  - role: worker
    instance_type: m5.large
    allocation_mode: spot
    image_id: ami-06af33e2399c52709
```

---

## Acceptance Criteria

- [ ] `cargo build` passes with zero errors after all changes.
- [ ] `cluster create` rejects a YAML with zero head nodes (error message references "head").
- [ ] `cluster create` rejects a YAML with two head nodes.
- [ ] `cluster create` rejects a YAML where `role: head` + `allocation_mode: spot`.
- [ ] `cluster create` rejects a YAML with an unknown role value.
- [ ] `cluster create` accepts the updated `cluster.example.yaml`.
- [ ] DB `nodes` table has `role` column after migrations run.
- [ ] A spawned cluster has its head node provisioned as the first EC2 request in logs.

---

## Implementation Order

1. Migration file (no compile dependency).
2. `Node` struct + INSERT query (compile-time check catches all callers immediately).
3. `NodeYaml` + validation + Node construction in `create.rs`.
4. Sort in `resource_manager.rs`.
5. Update `cluster.example.yaml`.
6. Run `cargo build` and fix any remaining compile errors.
7. Walk through acceptance criteria manually.

---

## What is NOT in Phase 1

- Init command templates for head vs worker (that is Phase 2).
- Slurm config generation on head node (Phase 2).
- Munge key distribution (Phase 2).
- Any MANA coordinator placement logic (Phase 2).
- Multi-node smoke test (Phase 2).
