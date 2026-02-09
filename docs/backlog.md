# NeuroMesh Backlog

This document enumerates the planned issues and tasks derived from the
Product Requirement Document (PRD).  Each issue should be created in
GitHub with a clear title, description, acceptance criteria, and labels.
Milestones (e.g., _MVP_, _Phase 0_, _v1_, _post‑v1_) group related
work.  Use the following labels where appropriate:

* **type:feature**, **type:bug**, **type:chore** – categorize the
  nature of the work.
* **area:core**, **area:consensus**, **area:node**, **area:api**,
  **area:sdk**, **area:gov**, **area:ops**, **area:doc** – component
  ownership.
* **priority:P0**, **priority:P1**, **priority:P2** – relative
  priority.
* **status:ready**, **status:blocked**, **status:in-progress** –
  workflow status.

## Core – Protocol Chain & Runtime

| ID | Title | Description | Acceptance Criteria | Labels |
|----|-------|-------------|---------------------|--------|
| **CORE‑001** | NeuroChain Substrate Runtime Skeleton | Scaffold a new Substrate runtime with basic pallets for balances, staking, extrinsics, and a custom `neurochain` pallet. | A compilable Rust crate that runs with `cargo run`; includes placeholder pallets and tests. | `type:feature`, `area:core`, `priority:P0` |
| **CORE‑002** | Subnet Registry Pallet | Implement a pallet to store subnet definitions (ID, task type, schemas, evaluation spec, emission weights, staking thresholds, owner). | Extrinsics to create, update, and retire subnets; on‑chain state accessible via RPC. | `type:feature`, `area:core`, `priority:P0` |
| **CORE‑003** | Miner & Validator Registry Pallets | Implement registration logic for miners and validators, including UID allocation and stake deposits. | Tests that miners and validators can register, update endpoints, and deregister. | `type:feature`, `area:core`, `priority:P0` |
| **CORE‑004** | Emissions & Reward Distribution Pallet | Define storage and extrinsics for emissions schedule and calculation of rewards per epoch. | Includes functions to compute miner and validator rewards based on weight matrices. | `type:feature`, `area:core`, `priority:P1` |

## Consensus – Weight Mechanism

| ID | Title | Description | Acceptance Criteria | Labels |
|----|-------|-------------|---------------------|--------|
| **CONS‑001** | Weight Matrix Storage & Compression | Design a storage structure for weight vectors and global weight matrices.  Use compression to minimize on‑chain storage. | Benchmarks show acceptable storage overhead and retrieval latency. | `type:feature`, `area:consensus`, `priority:P1` |
| **CONS‑002** | Validator Weight Submission Extrinsic | Implement an extrinsic allowing validators to submit their weight vectors. | Validates input, normalizes weights, and stores them for the epoch. | `type:feature`, `area:consensus`, `priority:P1` |
| **CONS‑003** | Global Weight Aggregation & Reputation Update | Implement the off‑chain worker or on‑chain logic to aggregate weight vectors and update validator reputations. | Unit tests verify correct aggregation and reputation updates. | `type:feature`, `area:consensus`, `priority:P1` |
| **CONS‑004** | Collusion / Cartel Detection Heuristics (v1) | Implement simple heuristics to detect highly correlated weight vectors indicating collusion. | Alerts or slashes are triggered when heuristics detect anomalies. | `type:feature`, `area:consensus`, `priority:P2` |

## Node – Miner & Validator Clients

| ID | Title | Description | Acceptance Criteria | Labels |
|----|-------|-------------|---------------------|--------|
| **NODE‑001** | libp2p Networking Layer | Implement peer discovery, pub/sub topics for miner and validator metadata. | Nodes can discover peers within the same subnet and gossip status. | `type:feature`, `area:node`, `priority:P0` |
| **NODE‑002** | Miner gRPC Inference Server (Reference) | Provide a reference Python implementation that wraps a trivial model and exposes `Infer` and `HealthCheck` gRPC methods. | Integration test confirms the server responds to inference requests. | `type:feature`, `area:node`, `priority:P0` |
| **NODE‑003** | Validator Client | Implement a Python client that samples miners, sends queries, scores outputs, and submits weight vectors. | Validators can register, sample miners, and persist scores locally. | `type:feature`, `area:node`, `priority:P0` |
| **NODE‑004** | Reference Subnet Implementation: CODE_GEN_BASE | Implement an example evaluation spec for a code generation subnet.  Define input/output schemas and simple unit test scoring. | Unit tests ensure that miners producing correct code receive higher scores. | `type:feature`, `area:node`, `priority:P1` |

## API – Aggregator & Public Interface

| ID | Title | Description | Acceptance Criteria | Labels |
|----|-------|-------------|---------------------|--------|
| **API‑001** | Aggregator gRPC/HTTP API Spec | Define the API contracts for submitting queries to a subnet, specifying constraints, top‑k selection, and ensemble strategy. | API documented in OpenAPI format with request/response types. | `type:doc`, `area:api`, `priority:P0` |
| **API‑002** | Implement Aggregator Service | Create a TypeScript/Node service that fetches `W_global`, selects miners, forwards queries, and aggregates results. | End‑to‑end test demonstrates the aggregator returning an aggregated response. | `type:feature`, `area:api`, `priority:P1` |
| **API‑003** | Rate Limiting & Auth Plugin | Implement middleware for rate limiting and optional API key authentication. | Tests show rate limits enforce per‑client quotas. | `type:feature`, `area:api`, `priority:P2` |

## SDK – Developer Tooling

| ID | Title | Description | Acceptance Criteria | Labels |
|----|-------|-------------|---------------------|--------|
| **SDK‑001** | Rust & Python Client SDKs for NeuroChain | Provide libraries to interact with the chain, submit extrinsics, and fetch state. | Examples in documentation show how to use the SDKs to register miners and submit weights. | `type:feature`, `area:sdk`, `priority:P1` |
| **SDK‑002** | Miner/Validator Boilerplate Templates | Provide cookie‑cutter templates for building new miners and validators. | Templates include CLI scaffolding and integration with the SDK. | `type:chore`, `area:sdk`, `priority:P2` |

## Governance – Proposals & DAO

| ID | Title | Description | Acceptance Criteria | Labels |
|----|-------|-------------|---------------------|--------|
| **GOV‑001** | Subnet Proposal & Bond Mechanism | Implement extrinsics to propose new subnets with a bonded deposit and a review period. | Proposal can be created, reviewed, and activated by governance. | `type:feature`, `area:gov`, `priority:P2` |
| **GOV‑002** | On‑Chain Governance Pallet (DAO v1) | Introduce pallets for proposals, voting, and execution.  Transition control of emissions and subnets to token holders. | Passing proposals result in runtime parameter changes; tests verify DAO logic. | `type:feature`, `area:gov`, `priority:P3` |

## Operations & Observability

| ID | Title | Description | Acceptance Criteria | Labels |
|----|-------|-------------|---------------------|--------|
| **OPS‑001** | CI/CD Pipeline | Configure GitHub Actions to run builds, linters, and tests for all components. | Workflow passes for all commits and PRs; failures block merges. | `type:chore`, `area:ops`, `priority:P0` |
| **OPS‑002** | Monitoring Stack | Provide a basic Prometheus & Grafana setup for observing node and aggregator metrics. | Dashboards display peer counts, API latency, and consensus metrics. | `type:feature`, `area:ops`, `priority:P2` |

## Documentation

| ID | Title | Description | Acceptance Criteria | Labels |
|----|-------|-------------|---------------------|--------|
| **DOC‑001** | Whitepaper & Protocol Spec v0.1 | Draft a formal whitepaper describing the economics, consensus, and security model. | Document reviewed and approved by core contributors. | `type:doc`, `area:doc`, `priority:P1` |
| **DOC‑002** | Developer Onboarding Guide | Produce step‑by‑step instructions for running a miner or validator on testnet. | New participants can follow the guide and successfully join the network. | `type:doc`, `area:doc`, `priority:P1` |

---

Each of the above issues should be tracked in GitHub with the suggested
labels and acceptance criteria.  Feel free to break tasks down
further as implementation details emerge.  Maintain clear titles and
explanations so contributors can easily understand and pick up work.