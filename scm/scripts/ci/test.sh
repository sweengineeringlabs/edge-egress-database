#!/usr/bin/env bash
set -euo pipefail

# Run tests for all feature combinations.
cargo test
cargo test --features sqlite
cargo test --features postgres
cargo test --features sqlite,postgres
