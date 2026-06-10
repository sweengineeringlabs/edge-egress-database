#!/usr/bin/env bash
set -euo pipefail

# Build all feature combinations to ensure none are broken.
cargo build
cargo build --features sqlite
cargo build --features postgres
cargo build --features sqlite,postgres
