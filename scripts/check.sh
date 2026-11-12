#!/bin/bash
# Runs all tests, stops on first failure.

set -euo pipefail
cd "$(dirname "$0")/.."

cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --quiet
