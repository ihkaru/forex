#!/usr/bin/env bash
# ==============================================================================
# cron_weekly_dukascopy_sync.sh
# Sinkronisasi Otomatis Data Historis Dukascopy ECN Setiap Akhir Pekan
# Jadwal Rekomendasi (Crontab): Sabtu 06:00 WIB (Jumat 23:00 UTC) saat pasar tutup
# Format Crontab: 0 23 * * 5 /root/projects/forex/scripts/cron_weekly_dukascopy_sync.sh >> /var/log/dukascopy_cron.log 2>&1
# ==============================================================================

set -euo pipefail

API_BASE="${API_BASE:-http://127.0.0.1:3000}"
CURRENT_YEAR=$(date +%Y)
LOG_TAG="[DUKASCOPY-CRON]"

echo "$LOG_TAG [$(date -u +"%Y-%m-%dT%H:%M:%SZ")] Memulai sinkronisasi mingguan data pasar Dukascopy ECN..."

PAIRS=("EURGBP" "EURUSD" "GBPUSD" "USDJPY" "USDCHF" "NZDUSD" "AUDUSD" "XAUUSD")

for SYMBOL in "${PAIRS[@]}"; do
    echo "$LOG_TAG Sinkronisasi $SYMBOL (H1) untuk tahun $CURRENT_YEAR..."
    HTTP_CODE=$(curl -s -o /tmp/ingest_resp.json -w "%{http_code}" -X POST "$API_BASE/api/market/ingest" \
        -H "Content-Type: application/json" \
        -d "{\"symbol\":\"$SYMBOL\",\"timeframe\":\"H1\",\"from_year\":$CURRENT_YEAR,\"to_year\":$CURRENT_YEAR}")
    
    if [ "$HTTP_CODE" -eq 200 ]; then
        SAVED_BARS=$(jq -r '.candles_saved // 0' /tmp/ingest_resp.json 2>/dev/null || echo "0")
        echo "$LOG_TAG ✅ $SYMBOL sukses disinkronkan ($SAVED_BARS bars baru tersimpan di SQLite WAL)."
    else
        echo "$LOG_TAG ⚠️ Gagal menyinkronkan $SYMBOL (HTTP $HTTP_CODE): $(cat /tmp/ingest_resp.json 2>/dev/null || true)"
    fi
done

echo "$LOG_TAG [$(date -u +"%Y-%m-%dT%H:%M:%SZ")] Sinkronisasi mingguan selesai."
