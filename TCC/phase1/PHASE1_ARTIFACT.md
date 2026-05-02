# Phase 1 Artifact — Node Roles and Hybrid Topology

Date: 2026-05-02
Project: Integrating MANA Fault Tolerance into HPC@Cloud for AWS Spot Clusters
Repository: hpcac-toolkit

## 1. Purpose
This artifact records the completed Phase 1 implementation: role-aware node model with
head-first provisioning in HPC@Cloud.

Phase 1 objective: support exactly one on-demand head node and N spot worker nodes in the
cluster configuration, with DB persistence, input validation, and head-first spawn ordering.

## 2. Changes Implemented

### 2.1 DB Migration
- File: `migrations/8__add_node_role.sql`
- Added `role TEXT NOT NULL DEFAULT 'worker'` column to the `nodes` table.
- Default `worker` preserves compatibility with any pre-existing records.
- Migration applied and verified via `sqlx migrate info` (status: 8/installed).

### 2.2 Node DB Model
- File: `src/database/models/node.rs`
- Added `role: String` field to the `Node` struct.
- `Node::insert()` now persists `role` in the INSERT query.
- `Node::fetch_by_private_ip()` SELECT updated to include `role`.
- `Cluster::get_nodes()` SELECT in `src/database/models/cluster.rs` updated to include `role`.

### 2.3 YAML Config Parser and Validation
- File: `src/commands/cluster/create.rs`
- Added `role: String` field to `NodeYaml` (required — serde rejects YAML missing it).
- Validation block runs immediately after YAML parse, before any AWS API calls:
  1. Role must be `head` or `worker` (rejects unknown values).
  2. Exactly one node must declare `role: head` (rejects 0 or 2+).
  3. The head node must use `allocation_mode: on-demand` (rejects spot head).
- Node display in confirmation prompt now shows `Role` per node.
- `Node` constructor passes `role` from `node_definition.role`.

### 2.4 Head-First Spawn Ordering
- File: `src/integrations/providers/aws/resource_manager.rs`
- Added sort before step 13 (ENI/EIP/EC2 provisioning loops):
  ```rust
  nodes.sort_by_key(|n| if n.role == "head" { 0usize } else { 1usize });
  ```
- Guarantees head EC2 instance is always requested and booted first, regardless of YAML
  declaration order.

### 2.5 Example Config
- File: `cluster.example.yaml`
- Updated with `role` field on all node entries.
- Active low-credit profile: t3.medium head (on-demand) + 2x m5.large workers (spot).
- Commented full-scale profile: t3.2xlarge head + up to 4x m5.8xlarge workers.
- Inline comments document role rules and `init_commands` usage.

## 3. Validation Summary (Passed)

### 3.1 Build
- `cargo build` completes with zero errors and zero warnings.

### 3.2 Validation rejection cases
All tested with `cargo run -- cluster create -f <yaml> -y`:

| Test case | Expected error | Result |
|---|---|---|
| Zero head nodes | `found 0` | PASS |
| Two head nodes | `found 2` | PASS |
| Head with `allocation_mode: spot` | `got 'spot'` | PASS |
| Unknown role value (`master`) | `Valid values: head, worker` | PASS |

### 3.3 DB schema
- Column `role TEXT NOT NULL DEFAULT 'worker'` confirmed present via `PRAGMA table_info(nodes)`.

### 3.4 Example YAML structural validity
- `cluster.example.yaml` parsed and confirmed: 1 head, 2 workers.

## 4. Constraints and Notes
- The head-first ordering acceptance criterion (head provisioned first in EC2 logs) will be
  confirmed live during the first real cluster spawn in Phase 2.
- `role` is stored as a plain string in the DB. No enum constraint at DB level — validation
  is enforced at parse time in `create.rs`.
- Workers have no `allocation_mode` default override; the existing default of `on-demand`
  applies if not specified in YAML. Spot must be declared explicitly.

## 5. Files Changed
| File | Type of change |
|---|---|
| `migrations/8__add_node_role.sql` | New file |
| `src/database/models/node.rs` | Added `role` field, updated INSERT and SELECT |
| `src/database/models/cluster.rs` | Updated `get_nodes` SELECT |
| `src/commands/cluster/create.rs` | Added `role` to `NodeYaml`, validation block, constructor wiring, display |
| `src/integrations/providers/aws/resource_manager.rs` | Head-first sort before provisioning loops |
| `cluster.example.yaml` | Updated with `role` field and MANA cluster profile |

## 6. Acceptance Criteria (Checklist)
- [x] `cargo build` passes with zero errors.
- [x] `cluster create` rejects YAML with zero head nodes.
- [x] `cluster create` rejects YAML with two head nodes.
- [x] `cluster create` rejects head node with `allocation_mode: spot`.
- [x] `cluster create` rejects unknown role value.
- [x] `cluster create` accepts the updated `cluster.example.yaml`.
- [x] DB `nodes` table has `role` column after migrations run.
- [ ] Spawned cluster has head node provisioned first in EC2 logs — to be confirmed in Phase 2.

## 7. Immediate Next Actions (Phase 2 Entry)
1. Write role-specific `init_commands` templates:
   - Head: generate `slurm.conf`, start `slurmctld`, start `mana_coordinator`.
   - Workers: join the Slurm cluster via `slurmd`, connect to head `slurmctld`.
2. Implement munge key generation on head and distribution to workers via SSM.
3. Verify EFS checkpoint directory is accessible on all nodes.
4. Spawn the first real 1-head + 2-worker cluster and confirm end-to-end Slurm job launch.
5. Validate MANA checkpoint/restart on multi-node cluster.

## 8. File References
- Phase 1 plan: `TCC/phase1/PHASE1_PLAN.md`
- Phase 0 artifact: `TCC/artifacts/phase0/PHASE0_BASELINE_ARTIFACT.md`
- Project plan: `TCC/TCC_MANA_INTEGRATION_PLAN.md`
