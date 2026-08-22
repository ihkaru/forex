#!/usr/bin/env bash
# .agents/scripts/rust_quality_gate.sh
#
# Stop Hook — Forex Workspace Agent Self-Correction Gate & Data Integrity Guard
#
# Konteks: Dipanggil oleh Antigravity setiap kali agen hendak berhenti.
# Tujuan:
#   1. Mencegah agen "selesai" jika cargo clippy masih punya error / warnings (-D warnings).
#   2. Menjalankan AST Fallback Scanner untuk mencegah silent fallback harga/volume.
#   Jika ada error, injeksi pesan balik ke agen untuk self-correct secara deterministik.
#
# Input (stdin): JSON berisi conversationId, workspacePaths, terminationReason, dll.
# Output (stdout): JSON { "decision": "continue"|"", "reason": "..." }

set -euo pipefail

# ─── Parse input ─────────────────────────────────────────────────────────────
INPUT=$(cat)
WORKSPACE=$(echo "$INPUT" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d.get('workspacePaths', ['.'])[0])" 2>/dev/null || echo ".")
TERMINATION_REASON=$(echo "$INPUT" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d.get('terminationReason', ''))" 2>/dev/null || echo "")

# Hanya jalankan gate saat agen berhenti secara normal (bukan error/timeout)
if [ "$TERMINATION_REASON" != "model_stop" ]; then
  echo "{}"
  exit 0
fi

# ─── Cek apakah ada file Rust yang berubah ───────────────────────────────────
CHANGED_RS=$(git -C "$WORKSPACE" diff --name-only HEAD 2>/dev/null | grep '\.rs$' | wc -l || echo "1")
if [ "$CHANGED_RS" -eq 0 ]; then
  echo "{}"
  exit 0
fi

cd "$WORKSPACE"

# ─── GATE 1: Jalankan AST Fallback Scanner (Deterministic Data Integrity) ───
SCANNER_SCRIPT="$WORKSPACE/.agents/scripts/ast_fallback_scanner.py"
if [ -f "$SCANNER_SCRIPT" ]; then
  SCANNER_OUTPUT=$(python3 "$SCANNER_SCRIPT" 2>&1)
  SCANNER_EXIT=$?
  if [ $SCANNER_EXIT -ne 0 ]; then
    python3 -c "
import json
msg = '''❌ AST FALLBACK SCANNER GAGAL — Ditemukan silent fallback yang merusak integritas data finansial!

Detail:
$SCANNER_OUTPUT

Aturan Integritas Data (docs/DATA_INTEGRITY.md):
- DILARANG hardcoded harga/volume fallback (.unwrap_or(dec!(...)))
- DILARANG silent float parsing fallback (.parse::<f64>().unwrap_or(...))
- Wajib gunakan explicit error propagation ('?') atau .ok_or_else().

Perbaiki file-file di atas sekarang.'''

print(json.dumps({
  'decision': 'continue',
  'reason': msg
}))
"
    exit 0
  fi
fi

# ─── GATE 2: Jalankan cargo clippy (-D warnings) ────────────────────────────
CLIPPY_OUTPUT=$(cargo clippy --workspace --all-targets -- -D warnings 2>&1)
CLIPPY_EXIT=$?

if [ $CLIPPY_EXIT -ne 0 ]; then
  ERRORS=$(echo "$CLIPPY_OUTPUT" | grep -E "^error" | head -15 | sed 's/"/\\"/g' | tr '\n' '\\n')
  
  python3 -c "
import json
msg = '''❌ CARGO CLIPPY GAGAL — perbaiki sebelum selesai.

ERROR yang ditemukan:
$ERRORS

Langkah:
1. Perbaiki semua error clippy di atas
2. Jalankan: cargo clippy --workspace --all-targets -- -D warnings
3. Pastikan 0 error sebelum selesai

Ingat: panic!(), unwrap() tanpa justifikasi, f64 arithmetic, dan silent fallback DILARANG.
Gunakan Result/anyhow untuk error handling, rust_decimal::Decimal untuk kalkulasi harga.'''

print(json.dumps({
  'decision': 'continue',
  'reason': msg
}))
"
  exit 0
fi

# ─── Semua Gate Lolos: Izinkan Selesai ──────────────────────────────────────
echo "{}"
