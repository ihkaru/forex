#!/usr/bin/env bash
set -e

echo "🔄 Menginisialisasi Git Submodules untuk repositori Forex Workspace..."
git submodule update --init --recursive

echo "📦 Memeriksa integritas Cargo Workspace..."
cargo check --workspace

echo "✅ Selesai! Semua modul siap digunakan."
