#!/usr/bin/env bash
set -euo pipefail

echo "================================================================="
echo "⚙️ MEMASANG FOREX SIGNAL DAEMON SEBAGAI SYSTEMD SERVICE (24/7)"
echo "================================================================="

PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
SERVICE_FILE="$PROJECT_DIR/deploy/systemd/forex-signal-daemon.service"

echo "1. Mengompilasi binary release..."
cd "$PROJECT_DIR"
cargo build --release --bin signal-daemon

echo "2. Menyalin unit service ke /etc/systemd/system/..."
sudo cp "$SERVICE_FILE" /etc/systemd/system/forex-signal-daemon.service

echo "3. Reload systemd daemon & mengaktifkan service..."
sudo systemctl daemon-reload
sudo systemctl enable --now forex-signal-daemon.service

echo "================================================================="
echo "✅ SERVICE BERHASIL DIAKTIFKAN!"
echo "• Cek status:  sudo systemctl status forex-signal-daemon.service"
echo "• Pantau log:  sudo journalctl -u forex-signal-daemon.service -f"
echo "• Hentikan:    sudo systemctl stop forex-signal-daemon.service"
echo "================================================================="
