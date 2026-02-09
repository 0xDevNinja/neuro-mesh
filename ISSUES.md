# NeuroMesh Issue Tracker

This document provides a comprehensive overview of all planned features, enhancements, and tasks for the NeuroMesh project. Issues are organized by component area and priority level for better visibility and tracking.

---

## Quick Reference

| Priority | Description | SLA Target |
|----------|-------------|------------|
| **P0** | Critical - Blocks release | Immediate |
| **P1** | High - Core functionality | Current sprint |
| **P2** | Medium - Important features | Next sprint |
| **P3** | Low - Nice to have | Backlog |

| Status | Description |
|--------|-------------|
| `Open` | Not yet started |
| `In Progress` | Currently being worked on |
| `Review` | Awaiting code review |
| `Done` | Completed and merged |

---

## Core Protocol - Chain & Runtime

### CORE-001: NeuroChain Substrate Runtime Skeleton
| Field | Value |
|-------|-------|
| **Priority** | P0 |
| **Status** | Open |
| **Area** | `core` |
| **Type** | Feature |

**Description:**
Scaffold a new Substrate runtime with basic pallets for balances, staking, extrinsics, and a custom `neurochain` pallet. This forms the foundation of the entire protocol.

**Acceptance Criteria:**
- [ ] Compilable Rust crate that runs with `cargo run`
- [ ] Includes placeholder pallets for balances, staking, and neurochain
- [ ] Unit tests for basic functionality
- [ ] Documentation for pallet structure

**Technical Notes:**
- Use Substrate FRAME framework
- Follow Substrate naming conventions
- Ensure compatibility with Polkadot ecosystem

---

### CORE-002: Subnet Registry Pallet
| Field | Value |
|-------|-------|
| **Priority** | P0 |
| **Status** | Open |
| **Area** | `core` |
| **Type** | Feature |

**Description:**
Implement a pallet to store subnet definitions including ID, task type, schemas, evaluation spec, emission weights, staking thresholds, and owner.

**Acceptance Criteria:**
- [ ] Extrinsics to create, update, and retire subnets
- [ ] On-chain state accessible via RPC
- [ ] Storage optimization for subnet metadata
- [ ] Events emitted for state changes

**Schema:**
```rust
struct Subnet {
    id: SubnetId,
    task_type: TaskType,      // CODE_GEN, IMAGE_GEN, PROTEIN_FOLDING, CUSTOM
    input_schema: Schema,
    output_schema: Schema,
    evaluation_spec: URI,
    emission_weight: Percent,
    min_stake_miner: Balance,
    min_stake_validator: Balance,
    owner: AccountId,
    status: SubnetStatus,
}
```

---

### CORE-003: Miner & Validator Registry Pallets
| Field | Value |
|-------|-------|
| **Priority** | P0 |
| **Status** | Open |
| **Area** | `core` |
| **Type** | Feature |

**Description:**
Implement registration logic for miners and validators, including UID allocation and stake deposits.

**Acceptance Criteria:**
- [ ] Miners can register with stake deposit
- [ ] Validators can register with stake deposit
- [ ] UID allocation is deterministic and collision-free
- [ ] Endpoint metadata storage (gRPC/HTTP addresses)
- [ ] Deregistration with stake unlock period
- [ ] Tests for registration, update, and deregistration flows

**Storage Structure:**
```rust
// Miner Registry
MinerRegistry: map (SubnetId, UID) => MinerInfo
MinerCount: map SubnetId => u32

// Validator Registry
ValidatorRegistry: map (SubnetId, UID) => ValidatorInfo
ValidatorCount: map SubnetId => u32
```

---

### CORE-004: Emissions & Reward Distribution Pallet
| Field | Value |
|-------|-------|
| **Priority** | P1 |
| **Status** | Open |
| **Area** | `core` |
| **Type** | Feature |

**Description:**
Define storage and extrinsics for emissions schedule and calculation of rewards per epoch.

**Acceptance Criteria:**
- [ ] Global emission schedule configuration
- [ ] Per-subnet emission allocation based on weights
- [ ] Miner reward calculation from weight matrix
- [ ] Validator reward calculation from reputation
- [ ] Epoch-based distribution mechanism
- [ ] Comprehensive test coverage

**Reward Formula:**
```
miner_reward[i] = subnet_emission * W_global[i] / sum(W_global)
validator_reward[j] = validator_emission * reputation[j] / sum(reputation)
```

---

## Consensus - Weight Mechanism

### CONS-001: Weight Matrix Storage & Compression
| Field | Value |
|-------|-------|
| **Priority** | P1 |
| **Status** | Open |
| **Area** | `consensus` |
| **Type** | Feature |

**Description:**
Design a storage structure for weight vectors and global weight matrices. Use compression to minimize on-chain storage costs.

**Acceptance Criteria:**
- [ ] Efficient storage schema for sparse weight matrices
- [ ] Compression algorithm (e.g., run-length encoding)
- [ ] Benchmarks show acceptable storage overhead
- [ ] Retrieval latency under 100ms for full matrix

**Storage Estimate:**
```
Per validator: ~1KB compressed (1000 miners, 16-bit weights)
Per subnet: ~100KB (100 validators)
Total: ~1MB (10 subnets)
```

---

### CONS-002: Validator Weight Submission Extrinsic
| Field | Value |
|-------|-------|
| **Priority** | P1 |
| **Status** | Open |
| **Area** | `consensus` |
| **Type** | Feature |

**Description:**
Implement an extrinsic allowing validators to submit their weight vectors for the current epoch.

**Acceptance Criteria:**
- [ ] Input validation (correct subnet, valid UID, proper format)
- [ ] Weight normalization (softmax or L1)
- [ ] Storage in epoch-specific bucket
- [ ] Duplicate submission handling
- [ ] Gas cost optimization

**Extrinsic Signature:**
```rust
fn submit_weights(
    origin: OriginFor<T>,
    subnet_id: SubnetId,
    weights: BoundedVec<(UID, Weight), MaxMiners>,
) -> DispatchResult;
```

---

### CONS-003: Global Weight Aggregation & Reputation Update
| Field | Value |
|-------|-------|
| **Priority** | P1 |
| **Status** | Open |
| **Area** | `consensus` |
| **Type** | Feature |

**Description:**
Implement the off-chain worker or on-chain logic to aggregate weight vectors and update validator reputations based on consensus agreement.

**Acceptance Criteria:**
- [ ] Weighted average aggregation of validator submissions
- [ ] Reputation update based on cosine similarity to consensus
- [ ] Outlier detection and reputation penalty
- [ ] Unit tests verify correct aggregation
- [ ] Performance benchmarks for 1000+ miners

**Algorithm:**
```python
W_global = sum(reputation[v] * W_v for v in validators) / sum(reputation)
for v in validators:
    similarity = cosine(W_v, W_global)
    reputation[v] = alpha * reputation[v] + (1 - alpha) * similarity
```

---

### CONS-004: Collusion / Cartel Detection Heuristics
| Field | Value |
|-------|-------|
| **Priority** | P2 |
| **Status** | Open |
| **Area** | `consensus` |
| **Type** | Feature |

**Description:**
Implement heuristics to detect highly correlated weight vectors indicating potential collusion between validators.

**Acceptance Criteria:**
- [ ] Pairwise correlation analysis of weight vectors
- [ ] Clustering detection for validator groups
- [ ] Alert mechanism for suspicious patterns
- [ ] Optional slashing for confirmed collusion
- [ ] False positive rate < 1%

**Detection Methods:**
1. Pearson correlation threshold (> 0.95)
2. Temporal pattern analysis
3. Network topology clustering

---

## Node - Miner & Validator Clients

### NODE-001: libp2p Networking Layer
| Field | Value |
|-------|-------|
| **Priority** | P0 |
| **Status** | Open |
| **Area** | `node` |
| **Type** | Feature |

**Description:**
Implement peer discovery and pub/sub topics for miner and validator metadata exchange.

**Acceptance Criteria:**
- [ ] Kademlia DHT for peer discovery
- [ ] Gossipsub for subnet topics
- [ ] Topic structure: `subnet-<id>-miners`, `subnet-<id>-validators`
- [ ] Peer status gossiping
- [ ] NAT traversal support

**Topics:**
```
/neuromesh/1.0.0/subnet/{id}/miners
/neuromesh/1.0.0/subnet/{id}/validators
/neuromesh/1.0.0/global/announcements
```

---

### NODE-002: Miner gRPC Inference Server (Reference)
| Field | Value |
|-------|-------|
| **Priority** | P0 |
| **Status** | In Progress |
| **Area** | `node` |
| **Type** | Feature |

**Description:**
Provide a reference Python implementation that wraps a model and exposes `Infer` and `HealthCheck` gRPC methods.

**Acceptance Criteria:**
- [ ] gRPC service definition (proto file)
- [ ] `Infer` method with configurable timeout
- [ ] `HealthCheck` method for liveness probes
- [ ] Docker containerization
- [ ] Integration test with validator client

**Proto Definition:**
```protobuf
service MinerService {
    rpc Infer(InferRequest) returns (InferResponse);
    rpc HealthCheck(Empty) returns (HealthStatus);
}

message InferRequest {
    string input = 1;
    map<string, string> metadata = 2;
}

message InferResponse {
    string output = 1;
    int64 latency_ms = 2;
}
```

---

### NODE-003: Validator Client
| Field | Value |
|-------|-------|
| **Priority** | P0 |
| **Status** | In Progress |
| **Area** | `node` |
| **Type** | Feature |

**Description:**
Implement a Python client that samples miners, sends queries, scores outputs, and submits weight vectors to the chain.

**Acceptance Criteria:**
- [ ] Miner discovery via chain state
- [ ] Random sampling with stratification
- [ ] Configurable scoring functions
- [ ] Weight vector computation and submission
- [ ] Local persistence of scores
- [ ] Retry logic and error handling

**Scoring Pipeline:**
```
1. Sample N miners from registry
2. Send inference request to each
3. Apply evaluation function to outputs
4. Normalize scores to weights
5. Submit weights on-chain
```

---

### NODE-004: Reference Subnet Implementation: CODE_GEN_BASE
| Field | Value |
|-------|-------|
| **Priority** | P1 |
| **Status** | Open |
| **Area** | `node` |
| **Type** | Feature |

**Description:**
Implement an example evaluation spec for a code generation subnet with defined input/output schemas and scoring logic.

**Acceptance Criteria:**
- [ ] Input schema: `{ prompt: string, language: string }`
- [ ] Output schema: `{ code: string, explanation: string }`
- [ ] Scoring: syntax validity, test pass rate, code quality
- [ ] Unit tests for scoring edge cases
- [ ] Documentation for subnet operators

---

## API - Aggregator & Public Interface

### API-001: Aggregator gRPC/HTTP API Spec
| Field | Value |
|-------|-------|
| **Priority** | P0 |
| **Status** | Open |
| **Area** | `api` |
| **Type** | Documentation |

**Description:**
Define the API contracts for submitting queries to a subnet, specifying constraints, top-k selection, and ensemble strategy.

**Acceptance Criteria:**
- [ ] OpenAPI 3.0 specification
- [ ] Request/response type definitions
- [ ] Error code documentation
- [ ] Rate limit specifications
- [ ] Authentication flow documentation

**Endpoints:**
```yaml
POST /v1/subnets/{id}/query
  - Submit inference request

GET /v1/subnets/{id}/miners
  - List active miners with weights

GET /v1/subnets/{id}/status
  - Get subnet health and metrics

GET /v1/health
  - Aggregator health check
```

---

### API-002: Implement Aggregator Service
| Field | Value |
|-------|-------|
| **Priority** | P1 |
| **Status** | In Progress |
| **Area** | `api` |
| **Type** | Feature |

**Description:**
Create a TypeScript/Node service that fetches `W_global`, selects miners, forwards queries, and aggregates results.

**Acceptance Criteria:**
- [ ] Chain state subscription for weight updates
- [ ] Top-k miner selection by weight
- [ ] Request forwarding with timeout
- [ ] Response aggregation (majority vote, weighted average)
- [ ] Caching layer for hot paths
- [ ] End-to-end test with mock miners

---

### API-003: Rate Limiting & Auth Plugin
| Field | Value |
|-------|-------|
| **Priority** | P2 |
| **Status** | Open |
| **Area** | `api` |
| **Type** | Feature |

**Description:**
Implement middleware for rate limiting and optional API key authentication.

**Acceptance Criteria:**
- [ ] Token bucket rate limiting
- [ ] Per-client quota tracking
- [ ] API key generation and validation
- [ ] Rate limit headers in responses
- [ ] Redis backend for distributed limiting

---

## SDK - Developer Tooling

### SDK-001: Rust & Python Client SDKs for NeuroChain
| Field | Value |
|-------|-------|
| **Priority** | P1 |
| **Status** | In Progress |
| **Area** | `sdk` |
| **Type** | Feature |

**Description:**
Provide libraries to interact with the chain, submit extrinsics, and fetch state.

**Acceptance Criteria:**
- [ ] Type-safe RPC wrappers
- [ ] Extrinsic builders for all pallets
- [ ] State query helpers
- [ ] Event subscription support
- [ ] Comprehensive examples
- [ ] Published to crates.io / PyPI

**Python Usage:**
```python
from neurochain_sdk import Client

client = Client("wss://testnet.neuromesh.io")
await client.register_miner(subnet_id=1, stake=1000)
await client.submit_weights(subnet_id=1, weights={1: 0.5, 2: 0.3, 3: 0.2})
```

---

### SDK-002: Miner/Validator Boilerplate Templates
| Field | Value |
|-------|-------|
| **Priority** | P2 |
| **Status** | Open |
| **Area** | `sdk` |
| **Type** | Chore |

**Description:**
Provide cookie-cutter templates for building new miners and validators quickly.

**Acceptance Criteria:**
- [ ] `cookiecutter` template for Python miner
- [ ] `cookiecutter` template for Python validator
- [ ] CLI scaffolding integration
- [ ] Pre-configured SDK integration
- [ ] GitHub Actions CI template

---

## Governance - Proposals & DAO

### GOV-001: Subnet Proposal & Bond Mechanism
| Field | Value |
|-------|-------|
| **Priority** | P2 |
| **Status** | Open |
| **Area** | `governance` |
| **Type** | Feature |

**Description:**
Implement extrinsics to propose new subnets with a bonded deposit and a review period.

**Acceptance Criteria:**
- [ ] Proposal creation with bond deposit
- [ ] Review period (configurable, default 7 days)
- [ ] Voting by token holders
- [ ] Automatic activation on approval
- [ ] Bond return/slash based on outcome

---

### GOV-002: On-Chain Governance Pallet (DAO v1)
| Field | Value |
|-------|-------|
| **Priority** | P3 |
| **Status** | Open |
| **Area** | `governance` |
| **Type** | Feature |

**Description:**
Introduce pallets for proposals, voting, and execution. Transition control of emissions and subnets to token holders.

**Acceptance Criteria:**
- [ ] Proposal types: Parameter change, Treasury spend, Upgrade
- [ ] Voting power proportional to stake
- [ ] Quorum requirements
- [ ] Timelock for execution
- [ ] Emergency multisig override

---

## Operations & Observability

### OPS-001: CI/CD Pipeline
| Field | Value |
|-------|-------|
| **Priority** | P0 |
| **Status** | Done |
| **Area** | `ops` |
| **Type** | Chore |

**Description:**
Configure GitHub Actions to run builds, linters, and tests for all components.

**Acceptance Criteria:**
- [x] Workflow passes for all commits and PRs
- [x] Failures block merges
- [x] Multi-language support (Rust, Python, TypeScript)
- [x] Caching for faster builds

---

### OPS-002: Monitoring Stack
| Field | Value |
|-------|-------|
| **Priority** | P2 |
| **Status** | Open |
| **Area** | `ops` |
| **Type** | Feature |

**Description:**
Provide a basic Prometheus & Grafana setup for observing node and aggregator metrics.

**Acceptance Criteria:**
- [ ] Prometheus scrape configuration
- [ ] Grafana dashboards for:
  - Peer count and connectivity
  - API latency percentiles
  - Consensus metrics (weight submissions, epoch progress)
  - Resource utilization
- [ ] Alerting rules for critical metrics

---

## Documentation

### DOC-001: Whitepaper & Protocol Spec v0.1
| Field | Value |
|-------|-------|
| **Priority** | P1 |
| **Status** | Open |
| **Area** | `docs` |
| **Type** | Documentation |

**Description:**
Draft a formal whitepaper describing the economics, consensus, and security model.

**Sections:**
1. Introduction & Motivation
2. Protocol Overview
3. Tokenomics & Incentive Design
4. Consensus Mechanism
5. Security Analysis
6. Roadmap

---

### DOC-002: Developer Onboarding Guide
| Field | Value |
|-------|-------|
| **Priority** | P1 |
| **Status** | Open |
| **Area** | `docs` |
| **Type** | Documentation |

**Description:**
Produce step-by-step instructions for running a miner or validator on testnet.

**Acceptance Criteria:**
- [ ] Hardware requirements
- [ ] Software prerequisites
- [ ] Step-by-step setup guide
- [ ] Troubleshooting section
- [ ] Video walkthrough

---

## Issue Summary

### By Priority

| Priority | Total | Open | In Progress | Done |
|----------|-------|------|-------------|------|
| P0 | 7 | 5 | 2 | 0 |
| P1 | 9 | 7 | 2 | 0 |
| P2 | 5 | 5 | 0 | 0 |
| P3 | 1 | 1 | 0 | 0 |

### By Area

| Area | Total | Open | In Progress | Done |
|------|-------|------|-------------|------|
| Core | 4 | 4 | 0 | 0 |
| Consensus | 4 | 4 | 0 | 0 |
| Node | 4 | 2 | 2 | 0 |
| API | 3 | 2 | 1 | 0 |
| SDK | 2 | 1 | 1 | 0 |
| Governance | 2 | 2 | 0 | 0 |
| Ops | 2 | 1 | 0 | 1 |
| Docs | 2 | 2 | 0 | 0 |

---

## Labels Reference

### Type Labels
- `type:feature` - New functionality
- `type:bug` - Something isn't working
- `type:chore` - Maintenance tasks
- `type:doc` - Documentation updates

### Area Labels
- `area:core` - Protocol chain & runtime
- `area:consensus` - Weight mechanism
- `area:node` - Miner & validator clients
- `area:api` - Aggregator & public interface
- `area:sdk` - Developer tooling
- `area:gov` - Governance & DAO
- `area:ops` - Operations & observability
- `area:doc` - Documentation

### Priority Labels
- `priority:P0` - Critical, blocks release
- `priority:P1` - High, core functionality
- `priority:P2` - Medium, important features
- `priority:P3` - Low, nice to have

### Status Labels
- `status:ready` - Ready to be picked up
- `status:blocked` - Waiting on dependency
- `status:in-progress` - Currently being worked on
- `status:review` - Awaiting code review

---

*Last updated: 2026-02-09*
