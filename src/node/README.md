# NeuroMesh Node

This crate provides the networking layer for the NeuroMesh protocol.  It
wraps [libp2p](https://github.com/libp2p/rust-libp2p) to enable peer
discovery and pub/sub communication between miners and validators, and
uses gRPC/HTTP to expose inference and coordination endpoints.

## Overview

* **Peer Discovery** – Nodes discover each other using libp2p’s mDNS
  and Kademlia protocols.  Each subnet has its own pub/sub topics for
  miners and validators.
* **Inference Server** – Miners expose a gRPC service with methods
  such as `Infer` and `HealthCheck`.  Validators call these
  endpoints when sampling miners.
* **Validator Coordination** – Validators expose endpoints for
  signalling evaluation rounds and submitting scores.  These will be
  defined using gRPC and integrated with the chain.

The current implementation includes a simple mDNS example
(`start_mdns_node`) to illustrate the libp2p setup.  Full
functionality will be added in upcoming issues.

## Running the Example

To run the mDNS example, execute:

```bash
cd neuro_mesh/src/node
cargo run --example mdns
```

This will start a node that advertises itself on the local network.
When multiple instances run, they will discover each other and print
peer IDs.  This is a placeholder for the future peer discovery
mechanism.