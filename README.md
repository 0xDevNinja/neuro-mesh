# NeuroMesh

<div align="center">

[![CI](https://github.com/0xDevNinja/neuro-mesh/actions/workflows/ci.yml/badge.svg)](https://github.com/0xDevNinja/neuro-mesh/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

**A Peer-to-Peer Intelligence Marketplace**

[Architecture](#architecture) | [Getting Started](#getting-started) | [Documentation](#documentation) | [Contributing](#contributing)

</div>

---

## Overview

**NeuroMesh** is a decentralized intelligence marketplace inspired by projects like Bittensor. It provides an open network where:

- **Miners** supply AI models or inference services
- **Validators** evaluate those services and compute quality scores
- A **weight-based consensus** mechanism distributes rewards to the most productive participants

The system is designed around a permissionless appchain (NeuroChain), specialized task arenas called **subnets**, and a public API that allows integrators to consume high-quality intelligence without trusting a central provider.

## Architecture

```
                                    NeuroMesh Protocol
    ┌──────────────────────────────────────────────────────────────────────────┐
    │                                                                          │
    │   ┌─────────────┐    Queries     ┌─────────────┐    Weights    ┌───────┐│
    │   │             │ ──────────────>│             │ ─────────────>│       ││
    │   │  Integrator │                │  Aggregator │               │ Chain ││
    │   │    (dApp)   │<───────────────│   Service   │<──────────────│       ││
    │   │             │   Responses    │             │    State      │       ││
    │   └─────────────┘                └──────┬──────┘               └───┬───┘│
    │                                         │                          │    │
    │                          ┌──────────────┼──────────────┐           │    │
    │                          │              │              │           │    │
    │                          ▼              ▼              ▼           │    │
    │                    ┌──────────┐   ┌──────────┐   ┌──────────┐      │    │
    │                    │  Miner   │   │  Miner   │   │  Miner   │      │    │
    │                    │  (GPU)   │   │  (GPU)   │   │  (GPU)   │      │    │
    │                    └────┬─────┘   └────┬─────┘   └────┬─────┘      │    │
    │                         │              │              │            │    │
    │                         └──────────────┼──────────────┘            │    │
    │                                        │                           │    │
    │                                        ▼                           │    │
    │                              ┌──────────────────┐                  │    │
    │                              │    Validators    │──────────────────┘    │
    │                              │ (Score & Weight) │                       │
    │                              └──────────────────┘                       │
    │                                                                          │
    └──────────────────────────────────────────────────────────────────────────┘
```

## Project Structure

```
neuro-mesh/
├── src/
│   └── chain/                      # NeuroChain (Substrate)
│       ├── src/
│       │   └── runtime.rs          # Runtime configuration
│       ├── pallets/
│       │   └── subnet-registry/    # Subnet Registry pallet (CORE-002)
│       │       ├── lib.rs
│       │       └── Cargo.toml
│       └── primitives/
│           └── sp-neuro-core/      # Core primitives & types
│               ├── src/lib.rs
│               └── Cargo.toml
│
├── docs/
│   ├── architecture.md             # Technical architecture
│   ├── backlog.md                  # Planned features and issues
│   └── CONTRIBUTING.md             # Contribution guidelines
│
├── .github/
│   └── workflows/
│       └── ci.yml                  # GitHub Actions CI pipeline
│
├── Cargo.toml                      # Rust workspace configuration
├── .gitignore
├── ISSUES.md                       # Issue tracking
├── SECURITY.md                     # Security policy
└── README.md                       # This file
```

## Getting Started

### Prerequisites

- **Rust** 1.75+ with `cargo`

### Installation

```bash
# Clone the repository
git clone https://github.com/0xDevNinja/neuro-mesh.git
cd neuro-mesh

# Install Rust toolchain
rustup toolchain install stable
rustup default stable
```

### Building & Testing

```bash
# Build all workspace crates
cargo build --all-targets

# Run all tests
cargo test --all

# Check formatting
cargo fmt --all -- --check

# Lint
cargo clippy --all-targets --all-features
```

## Current Status

| Component | Status | Description |
|-----------|--------|-------------|
| **Runtime Skeleton** (CORE-001) | Done | Basic Substrate runtime configuration |
| **Subnet Registry** (CORE-002) | Done | Pallet for subnet definitions, create/update/retire extrinsics |
| **Core Primitives** | Done | `sp-neuro-core` types and helpers |
| **Miner/Validator Registry** (CORE-003) | Next | Registration logic, UID allocation, stake deposits |
| **Emissions Pallet** (CORE-004) | Planned | Reward distribution per epoch |

See [ISSUES.md](ISSUES.md) for the full backlog.

## Documentation

| Document | Description |
|----------|-------------|
| [Architecture](docs/architecture.md) | Technical design and protocol specification |
| [Backlog](docs/backlog.md) | Planned features, issues, and roadmap |
| [Contributing](docs/CONTRIBUTING.md) | Guidelines for contributors |
| [Issues](ISSUES.md) | Current issues and tracking |
| [Security](SECURITY.md) | Security policy and vulnerability reporting |

## Roadmap

| Phase | Milestone | Status |
|-------|-----------|--------|
| **0: Genesis** | Testnet with single subnet, basic staking, weight submission | In Progress |
| **1: Multi-Subnet** | Multiple subnets, reputation system, public aggregator | Planned |
| **2: Permissionless** | Permissionless subnet creation, sybil detection, DeFi integration | Planned |
| **3: Governance** | DAO governance, on-chain proposals, treasury management | Planned |

## Contributing

Contributions are welcome! Please read our [Contributing Guide](docs/CONTRIBUTING.md) before submitting a pull request.

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'feat: add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

<div align="center">

**[Website](https://neuromesh.io)** | **[Documentation](https://docs.neuromesh.io)** | **[Discord](https://discord.gg/neuromesh)**

Built with passion by [0xDevNinja](https://github.com/0xDevNinja)

</div>
