# NeuroMesh

<div align="center">

[![CI](https://github.com/0xDevNinja/neuro-mesh/actions/workflows/ci.yml/badge.svg)](https://github.com/0xDevNinja/neuro-mesh/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/python-3.10%2B-blue.svg)](https://www.python.org/)
[![TypeScript](https://img.shields.io/badge/typescript-5.0%2B-blue.svg)](https://www.typescriptlang.org/)

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

### Core Components

| Component | Language | Description |
|-----------|----------|-------------|
| **NeuroChain** | Rust (Substrate) | Application-specific blockchain storing global state: accounts, subnets, registrations, weights, and emissions |
| **Node** | Rust | libp2p networking layer for peer discovery and communication |
| **Miner** | Python | Reference client hosting AI models and serving inference requests |
| **Validator** | Python | Reference client sampling miners, scoring outputs, and submitting weight vectors |
| **Aggregator** | TypeScript | Public API service routing queries to top-ranked miners |
| **SDK** | Rust & Python | Client libraries for chain interaction |

## Project Structure

```
neuro-mesh/
├── src/
│   ├── chain/                    # Substrate Runtime (Rust)
│   │   ├── src/
│   │   │   ├── lib.rs           # Runtime configuration
│   │   │   └── pallets/         # FRAME pallets
│   │   │       └── mod.rs       # Pallet definitions
│   │   ├── Cargo.toml
│   │   └── README.md
│   │
│   ├── node/                     # P2P Networking Layer (Rust)
│   │   ├── src/
│   │   │   └── lib.rs           # libp2p implementation
│   │   ├── Cargo.toml
│   │   └── README.md
│   │
│   ├── miner/                    # Reference Miner Client (Python)
│   │   ├── miner.py             # Main miner implementation
│   │   ├── tests/
│   │   │   └── test_miner.py
│   │   ├── Dockerfile
│   │   ├── requirements.txt
│   │   └── README.md
│   │
│   ├── validator/                # Reference Validator Client (Python)
│   │   ├── validator.py         # Main validator implementation
│   │   ├── tests/
│   │   │   └── test_validator.py
│   │   ├── requirements.txt
│   │   └── README.md
│   │
│   ├── aggregator/               # Public API Service (TypeScript)
│   │   ├── src/
│   │   │   └── index.ts         # Express API server
│   │   ├── tests/
│   │   │   └── index.test.ts
│   │   ├── Dockerfile
│   │   ├── package.json
│   │   └── tsconfig.json
│   │
│   └── sdk/                      # Client SDKs
│       ├── rust/
│       │   ├── src/
│       │   │   ├── lib.rs
│       │   │   └── client.rs
│       │   ├── tests/
│       │   └── Cargo.toml
│       └── python/
│           ├── neurochain_sdk/
│           │   ├── __init__.py
│           │   ├── client.py
│           │   └── tests/
│           └── requirements.txt
│
├── docs/
│   ├── architecture.md           # Technical architecture
│   ├── backlog.md               # Planned features and issues
│   └── CONTRIBUTING.md          # Contribution guidelines
│
├── scripts/
│   ├── docker-compose.yml       # Container orchestration
│   └── run_tests.sh             # Test runner script
│
├── .github/
│   └── workflows/
│       └── ci.yml               # GitHub Actions CI pipeline
│
├── Cargo.toml                   # Rust workspace configuration
├── foundry.toml                 # Foundry configuration
├── .gitignore
├── ISSUES.md                    # Issue tracking
├── SECURITY.md                  # Security policy
└── README.md                    # This file
```

## Getting Started

### Prerequisites

- **Rust** 1.75+ with `cargo`
- **Python** 3.10+ with `pip`
- **Node.js** 18+ with `npm`
- **Docker** (optional, for containerized deployment)

### Installation

```bash
# Clone the repository
git clone https://github.com/0xDevNinja/neuro-mesh.git
cd neuro-mesh

# Install Rust toolchain
rustup toolchain install stable
rustup default stable

# Install Python dependencies
python3 -m venv .venv
source .venv/bin/activate
pip install -r src/miner/requirements.txt
pip install -r src/validator/requirements.txt

# Install Node.js dependencies
cd src/aggregator && npm install && cd ../..
```

### Quick Start

**1. Start a Miner**
```bash
cd src/miner
python miner.py --host 0.0.0.0 --port 5000
```

**2. Start a Validator**
```bash
cd src/validator
python validator.py --miners 127.0.0.1:5000 --query "What is 2+2?"
```

**3. Start the Aggregator**
```bash
cd src/aggregator
npm run build && npm start
```

**4. Query the API**
```bash
curl -X POST http://localhost:3000/v1/subnets/1/query \
  -H "Content-Type: application/json" \
  -d '{"input": "Hello, NeuroMesh!"}'
```

### Docker Deployment

```bash
cd scripts
docker-compose up -d
```

## Development

### Building

```bash
# Build Rust components
cargo build --release

# Build TypeScript aggregator
cd src/aggregator && npm run build
```

### Testing

```bash
# Run all tests
bash scripts/run_tests.sh

# Run Rust tests
cargo test --all

# Run Python tests
pytest src/miner/tests src/validator/tests

# Run TypeScript tests
cd src/aggregator && npm test
```

### Code Quality

| Tool | Language | Purpose |
|------|----------|---------|
| `rustfmt` | Rust | Code formatting |
| `clippy` | Rust | Linting |
| `black` | Python | Code formatting |
| `flake8` | Python | Linting |
| `prettier` | TypeScript | Code formatting |
| `eslint` | TypeScript | Linting |

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
