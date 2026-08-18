#!/usr/bin/env bash
set -e

echo "🚀 Menjalankan Signal Engine CLI Preview..."
RUST_LOG=info cargo run -p cli
