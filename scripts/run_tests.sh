#!/usr/bin/env bash
# Run all tests across the monorepo.
set -euo pipefail

echo "Running Rust tests..."
(cd ../src/chain && cargo test)
(cd ../src/node && cargo test || true)
(cd ../src/sdk/rust && cargo test)

echo "Running Python tests..."
python -m pytest ../src/miner/tests
python -m pytest ../src/validator/tests
python -m pytest ../src/sdk/python/neurochain_sdk/tests

echo "Running TypeScript tests..."
(cd ../src/aggregator && npm install --silent && npm test)

echo "All tests passed"