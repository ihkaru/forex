#!/usr/bin/env bash
set -euo pipefail

echo "================================================================="
echo "📊 MENJALANKAN TRADERS FAMILY QUANT BACKTEST LAB"
echo "================================================================="

cd "$(dirname "$0")/.."
cargo run --release --bin cli
