#!/usr/bin/env bash
set -euo pipefail

echo "================================================================="
echo "🚀 MEMULAI FOREX AUTONOMOUS SIGNAL DAEMON (TRADERS FAMILY)"
echo "================================================================="

cd "$(dirname "$0")/.."
export RUST_LOG=info
cargo run --release --bin signal-daemon
