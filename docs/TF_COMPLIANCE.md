# Regulasi Traders Family — Compliance & Scoring Guide

> File ini berisi spesifikasi lengkap aturan Traders Family.
> Dimuat oleh agen on-demand saat task berkaitan dengan signal generation atau scoring.
> GEMINI.md hanya menyimpan pointer ke sini.

---

## 1. Sistem Valued Pips (VP)

$$\text{Valued Pips} = \text{Pips Bersih} \times \text{Value Multiplier}$$

| Tier | Multiplier | Pairs | Min SL/TP | Max SL/TP | Gap |
|:---:|:---:|:---|:---:|:---:|:---:|
| **Tier 1** | 2.0x | `NZDUSD`, `AUDUSD`, `EURGBP`, `USDCHF` | 10 pips | 200 pips | 50 pips |
| **Tier 2** | 1.5x | `USDCAD`, `EURUSD`, `GBPUSD`, `NZDJPY`, `CADJPY`, `AUDJPY` | 15 pips | 300 pips | 75 pips |
| **Tier 3** | 1.0x | `USDJPY`, `EURJPY`, `GBPJPY`, `CHFJPY`, `EURNZD` | 20 pips | 400 pips | 100 pips |
| **Tier 4** | 0.5x | `XAUUSD` (Gold) | 30 pips | 500 pips | 100 pips |

**Target Bulanan**: ≥ 300 VP + ≥ 5 sinyal settled.

---

## 2. Invariant Kepatuhan Sinyal — Zero-Penalty Guarantee

Diimplementasikan di `TfComplianceGuard` (`crates/core/domain/src/compliance/`).

| # | Aturan | Detail |
|:---:|:---|:---|
| 1 | **Pending Order Only** | Hanya `BuyLimit`, `SellLimit`, `BuyStop`, `SellStop`. Instant execution DITOLAK. |
| 2 | **Risk:Reward 1:1 – 1:3** | Wajib `1:1.0 ≤ R:R ≤ 1:3.0`. DILARANG > 1:3. |
| 3 | **SL ≤ 1.5 × TP** | Stop Loss tidak boleh lebih dari 1.5× Take Profit dalam pips. |
| 4 | **Maks 2 Sinyal/Pair** | Maksimal 2 sinyal aktif per currency pair. |
| 5 | **Durasi 1–48 jam** | Expiry 1–48 jam. Jika dibuat hari Jumat: maks 96 jam. |
| 6 | **No Intervention** | Sinyal Running: DILARANG modifikasi SL/TP. Exit hanya via TP atau SL hit. |

---

## 3. Matriks 7 Pilar Skor Channel (Update Juli 2026)

| Pilar | Bobot | Target Skor Penuh (4 Poin) |
|:---|:---:|:---|
| **Recovery Factor** | 23.53% | Nett P/L ÷ Max DD ≥ 8.0 |
| **Level Channel** | 17.65% | Legend (bukan Master) |
| **Status Kemitraan** | 17.65% | Priority Channel (bukan Prospect) |
| **Profit Factor** | 17.65% | PF ≥ 2.10 dalam 6 bulan terakhir |
| **Monthly Loss Ratio** | 11.76% | Loss 0%–10% dari rata-rata profit |
| **Profit Months** | 5.88% | 6 dari 6 bulan terakhir profit |
| **Subscribers** | 5.88% | ≥ 501 subscriber berbayar |

**Skor Maksimal**: 4 × 7 = 28 poin.

---

## 4. Implementasi di Kode

```rust
// Validasi sinyal sebelum publikasi
let compliance = TfComplianceGuard::new(pair_spec);
compliance.validate(&signal)?;  // Error jika ada invariant yang dilanggar

// Kalkulasi VP
let vp = compliance.calculate_valued_pips(net_pips);
```

Lihat: `crates/core/domain/src/compliance/tf_compliance_guard.rs`
