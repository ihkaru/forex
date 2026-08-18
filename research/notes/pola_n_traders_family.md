# 📊 Formulasi Kuantitatif: Pola N Traders Family (Signature Strategy)

Dokumen ini membedah spesifikasi teknis, kaidah matematis, dan integrasi strategi **Pola N** khas **Traders Family** ke dalam arsitektur quant workspace Rust kita.

---

## 🎯 1. Prinsip Decoupling & Interface-First Strategy

Sesuai standar arsitektur sistem kita:
- **Zero I/O & Decoupled Data**: Strategi `PolaNStrategy` **sama sekali tidak mengetahui** dari mana asal data candle/tick (Dukascopy Swiss True-Tick, MetaTrader 5 Socket Bridge, cTrader ProtoBuf, atau Mock Test Vector).
- **Interface Contract (`StrategyPort`)**: Menerima `MarketContext` murni yang berisi slice `&[Candle]`, `&Tick`, dan `&RiskProfile`, kemudian mengembalikan `Result<Option<Signal>, DomainError>`.

---

## 📐 2. Kaidah Geometris & Rumus Matematika Pola N

```
       POLA N (BULLISH CONTINUATION)               POLA N TERBALIK (BEARISH CONTINUATION)
       -----------------------------               --------------------------------------
                   (High 1)                                    (Start H1)
                      /\                                          \
                     /  \  (Retest Support/                        \   /\  (Retest Resistance/
                    /    \  Higher Low L2)                          \ /  \  Lower High H2)
                   /      \/ (ENTRY BUY)                             V    \/ (ENTRY SELL)
                  /                                                        \
                 /                                                          \
             (Start L1)                                                   (Low 1)
```

### A. Pola N Bullish
1. **Swing Low Awal ($L_1$)**: Titik lembah sebelum ekspansi impulsif.
2. **Swing High Puncak ($H_1$)**: Titik tertinggi yang menembus resisten sebelumnya ($H_1 > L_1$).
3. **Higher Low Retest ($L_2$)**: Titik koreksi/pullback yang **wajib tertahan** di atas $L_1$ ($L_1 < L_2 < H_1$).
4. **Kalkulasi Level Eksekusi**:
   - $\text{Entry Price} = P_{\text{current}} \ge L_2$
   - $\text{Stop Loss} = L_2 - \text{Pip Buffer}$ (buffer 1.5–2.5 pips)
   - $\text{Take Profit 1} = H_1$
   - $\text{Take Profit 2} = \text{Entry} + (\text{Risk Distance} \times 2.5)$
   - $\text{Target } R:R \ge 1:2.0$

### B. Pola N Bearish (N Terbalik)
1. **Swing High Awal ($H_1$)**: Titik puncak sebelum dorongan turun impulsif.
2. **Swing Low Dasar ($L_1$)**: Titik terendah yang menembus support ($L_1 < H_1$).
3. **Lower High Retest ($H_2$)**: Titik pullback naik yang **wajib tertahan** di bawah $H_1$ ($L_1 < H_2 < H_1$).
4. **Kalkulasi Level Eksekusi**:
   - $\text{Entry Price} = P_{\text{current}} \le H_2$
   - $\text{Stop Loss} = H_2 + \text{Pip Buffer}$ (buffer 1.5–2.5 pips)
   - $\text{Take Profit 1} = L_1$
   - $\text{Take Profit 2} = \text{Entry} - (\text{Risk Distance} \times 2.5)$
   - $\text{Target } R:R \ge 1:2.0$

---

## 🏆 3. Kepatuhan Standar Kualifikasi Analis Traders Family

Untuk mencapai level **Elite / Master / Legend** dan memaksimalkan *Revenue Sharing Subscribers* + *TF Point Cash-Out*:

1. **Target Valued Pips (VP)**: Minimal **300.0 VP / bulan**.
2. **Target Sinyal Settle**: Minimal **5 sinyal tereksekusi / bulan**.
3. **Akumulasi Pips**: Wajib $> 0$ pada tutup buku bulanan.
4. **Profit Factor Target**: Menargetkan $PF \ge 2.1$ (mendapatkan skor maksimal 4 poin pada sistem scoring).
5. **Drawdown Protection**: Stop loss ketat berbasis swing leg + buffer mencegah monthly loss $> 10\%$ (menjaga skor loss ratio 4 poin).

---

## 🧩 4. Struktur Komposisi Kode Rust

- **`SwingPointDetector`**: Komponen modular pendeteksi fraktal swing high & low dengan parameter `left_bars` dan `right_bars`.
- **`PolaNFormationEngine`**: Komponen validator pembentukan kaki-kaki N dan kalkulator SL/TP.
- **`PolaNStrategy`**: Komposisi dari kedua komponen di atas yang mengimplementasikan trait `domain::ports::StrategyPort`.
