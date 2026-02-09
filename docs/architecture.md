# NeuroMesh Architecture

This document provides a high‑level overview of the NeuroMesh
architecture.  For a more detailed specification, refer to the
[Product Requirement Document (PRD)](../NeuroMesh_PRD_and_Architecture.md) and
the issues in the backlog.

## Overview

NeuroMesh is a decentralized intelligence market.  It aims to provide a
neutral platform where AI service providers (miners) and evaluators
(validators) can participate in a competitive marketplace to supply
high‑quality inference services.  The protocol combines an
application‑specific blockchain (NeuroChain), off‑chain networking via
libp2p and gRPC, and a thin API layer for integrators.

### Key Roles

* **Miners** – Participants who host AI models or pipelines and serve
  inference requests.  Miners stake tokens on the chain, register
  themselves in a subnet, and expose a standard gRPC/HTTP API.

* **Validators** – Participants who design sampling and scoring
  mechanisms for a subnet.  Validators query miners, compute scores,
  produce a weight vector, and submit that vector on‑chain.  Their
  reputation is updated based on agreement with global consensus, and
  they earn a portion of the emissions.

* **Integrators** – dApps and services that consume AI from the
  network.  They interact with a public aggregator service or call
  miners directly, selecting the best miners according to on‑chain
  weights.

* **Governance** – Initially, protocol administrators approve new
  subnets and parameter changes.  In later phases, governance will be
  decentralized via a DAO.

## Components

### NeuroChain (Substrate Runtime)

NeuroChain is an appchain built on the [Substrate](https://substrate.io)
framework.  It maintains the following state:

* **Accounts** – balances, staking deposits, and nonces.
* **Subnets** – configuration for each task type (input/output
  schemas, evaluation spec, minimum stakes, emissions weight, etc.).
* **Miner & Validator Registries** – mappings from (subnet, UID) to
  registration info and status.
* **Weights** – compressed representation of the consensus weight
  vector `W_global` for each subnet and epoch.
* **Reputation** – per‑validator reputation scores.
* **Emissions** – global emission schedule and per‑subnet allocation.

The runtime exposes extrinsics for registration, staking, weight
submission, subnet proposals, and governance actions.  Pallets are
organized under `src/chain/pallets/`.

### Subnet Layer

A **subnet** represents a specialized intelligence task.  Each subnet
specifies:

* `task_type` – one of `CODE_GEN`, `IMAGE_GEN`, `PROTEIN_FOLDING`,
  `CUSTOM`.
* `input_schema` and `output_schema` – definitions of allowed inputs
  and outputs.
* `evaluation_spec` – a URI (e.g., to a repository or container
  image) containing the scoring logic used by validators.
* `emission_weight` – the fraction of total emissions allocated to
  this subnet.
* `min_stake_miner` and `min_stake_validator` – thresholds for
  registration.
* `owner` – the initial controlling entity, to be replaced by DAO
  governance in a later phase.

New subnets are proposed with a bond and undergo governance review
before activation.

### Consensus & Weight Mechanism

NeuroMesh uses a **weight‑based consensus** inspired by the Yuma
protocol.  Validators submit weight vectors over miners; those vectors
are aggregated to obtain a global weight `W_global`.  Validators are
scored based on their agreement with the consensus and adjust their
reputation accordingly.  Reward distribution for each epoch is derived
from the weight matrix and validator reputations:

1. **Sampling & scoring** – Validators sample miners, execute
   inference tasks, and compute scores using the subnet’s evaluation
   logic.
2. **Weight submission** – Validators convert scores to weight
   vectors via softmax or another normalization function and submit
   them on‑chain.
3. **Aggregation** – The runtime aggregates weight vectors to compute
   a consensus weight.  Reputation is updated to reward validators
   aligned with consensus and penalize outliers.
4. **Reward distribution** – Miner and validator rewards are
   calculated based on the aggregated weights and reputations.

### Networking & API

* **libp2p overlay** – Peers discover each other and exchange status
  information via pub/sub topics (`subnet-<id>-miners`, `subnet-<id>-validators`).
* **gRPC & HTTP** – Miners expose inference endpoints; validators
  expose control‑plane endpoints; an aggregator service provides a
  unified REST interface (`/v1/subnets/{id}/query`).
* **Aggregator** – Queries the chain for the latest weights,
  selects top miners, forwards requests, and aggregates outputs.

## Development Phases

The product roadmap is divided into four phases:

| Phase | Milestones |
|------ |----------- |
| **0: Genesis (Testnet)** | Launch testnet with a single subnet (`CODE_GEN_BASE`), basic staking, weight submission, and reference SDKs. |
| **1: Multi‑Subnet & Incentives** | Support multiple subnets, implement reputation and robust consensus, release 3–5 subnets, and provide a public aggregator. |
| **2: Permissionless Subnets** | Enable permissionless creation of subnets with bonding, add sybil/collusion detection, and integrate DeFi primitives. |
| **3: Decentralized Governance** | Transition governance to a DAO, implement on‑chain proposals and treasury, and provide governance tooling. |

## Source Guide

For detailed design notes, see the files in `src/chain` (Rust),
`src/miner` (Python), `src/validator` (Python), and `src/aggregator`
(TypeScript).  Each subdirectory contains its own README with
implementation details and references to relevant pallets, services,
and protocols.