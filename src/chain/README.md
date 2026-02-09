# NeuroChain Runtime

This crate contains the Substrate runtime for **NeuroChain**, the
application‑specific blockchain that underpins the NeuroMesh protocol.
The runtime is responsible for managing on‑chain state such as
accounts, staking, subnets, registries, weight matrices, and
emissions.  It exposes extrinsics for participants to register as
miners or validators, submit weight vectors, propose new subnets, and
participate in governance.

## Pallet Overview

The runtime is composed of several pallets (modules) built using
Substrate’s [FRAME](https://substrate.dev/docs/en/knowledgebase/runtime/frame)
library.  Each pallet encapsulates a specific piece of state or logic:

* **Balances** – standard pallet for account balances and transfers.
* **Staking** – manages staking deposits and slashing.
* **Subnet Registry** – stores subnet definitions and parameters.
* **Miner Registry** – handles miner registration, endpoint metadata,
  and UID allocation.
* **Validator Registry** – handles validator registration and
  evaluation capacity.
* **Emissions** – computes and distributes token emissions to miners
  and validators based on the consensus weight matrix.
* **Governance** – (future) DAO module for proposals and voting.

## Building

To build the runtime, ensure you have Rust and Substrate
dependencies installed.  Then run:

```bash
cd neuro_mesh/src/chain
cargo build
```

At the moment, this crate is a scaffold.  Pallets and runtime
configuration will be added as issues from the backlog are
implemented.