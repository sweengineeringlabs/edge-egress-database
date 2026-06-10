#!/usr/bin/env bash
set -euo pipefail

cargo fmt --check
cargo clippy -- -D warnings
cargo clippy --features sqlite -- -D warnings
cargo clippy --features postgres -- -D warnings
cargo clippy --features sqlite,postgres -- -D warnings
