#!/usr/bin/env bash
# .agents/scripts/tf_reminder.sh
#
# PreInvocation Hook — TF Compliance Quick Reminder
#
# Konteks: Dipanggil sebelum setiap model invocation.
# Tujuan: Ingatkan agen tentang invariant kritis TF Compliance
#         tanpa perlu GEMINI.md menjadi encyclopedia.
#
# Pesan yang diinjeksi bersifat EPHEMERAL — tidak masuk conversation history.
# Hanya muncul sebagai "sistem" hint untuk model pada invocation tersebut.

set -euo pipefail

# Injeksi sebagai ephemeral system message
python3 -c "
import json
print(json.dumps({
  'injectSteps': [
    {
      'ephemeralMessage': '''[TF Compliance Quick Check]
Sebelum membuat/memodifikasi sinyal:
- Pending Order ONLY (BuyLimit/SellLimit/BuyStop/SellStop)
- RR: 1:1.0 s/d 1:3.0 (DILARANG > 1:3.0)
- SL <= 1.5 x TP
- Maks 2 sinyal aktif per pair
- Durasi: 1-48 jam (96 jam jika Jumat)
- Kode Rust: gunakan Decimal, bukan f64. Max 600 baris per file.
Detail: docs/TF_COMPLIANCE.md'''
    }
  ]
}))
"
