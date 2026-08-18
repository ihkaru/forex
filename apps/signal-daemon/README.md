# 🤖 App: `signal-daemon`

Daemon utama yang berjalan di latar belakang (background service) secara terus-menerus.

## Tanggung Jawab
1. Mengonsumsi data tick / candlestick secara real-time dari broker.
2. Mengevaluasi seluruh strategi kuantitatif yang terpasang.
3. Melakukan kalkulasi manajemen risiko (Risk:Reward, position sizing).
4. Mempublikasikan sinyal yang valid secara instan ke Channel **Trader Family** dan Telegram.

## Menjalankan Daemon
```bash
cargo run -p signal-daemon
```
