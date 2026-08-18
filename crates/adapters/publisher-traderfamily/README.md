# 📡 Crate: `publisher-traderfamily` (Trader Family Adapter)

Adapter untuk mengotomasi publikasi sinyal ke channel berbayar (**VIP Signal Channel**) di aplikasi Android **Trader Family**.

## Fitur
- Mengimplementasikan `domain::ports::SignalPublisherPort`.
- Melakukan auto-formatting sinyal sesuai standar channel Trader Family (Pair, Action, Entry, SL, Multi-TP).
- Manajemen sesi token autentikasi dan auto-refresh token.
- Mendukung mode standalone repo (Git Submodule).
