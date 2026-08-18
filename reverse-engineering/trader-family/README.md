# 📱 Reverse Engineering: Trader Family Android App

Folder ini berisi seluruh artefak riset dekompilasi dan analisis protokol aplikasi Android **Trader Family**.

## Tujuan
Mengidentifikasi endpoint API internal untuk:
1. Autentikasi akun & token refresh.
2. Mengambil data channel & daftar subscriber.
3. Mengirimkan post sinyal baru (*create signal/post*) secara otomatis.
4. Memperbarui status sinyal (Take Profit hit, Stop Loss hit, Close).

## Folder & Alat
- `apks/`: Tempat meletakkan file `base.apk` dari perangkat Android.
- `docs/api_endpoints.md`: Dokumentasi spesifikasi API (Headers, Auth, Body, Responses).
- `frida/`: Script bypass SSL Pinning untuk mempermudah intercept HTTPS via mitmproxy.
- `jadx/`: Catatan hasil dekompilasi Java/Kotlin source code.

## Cara Mengambil APK dari HP Android
```bash
# 1. Cari package name Trader Family
adb shell pm list packages | grep trader

# 2. Ambil path APK
adb shell pm path com.traderfamily.app

# 3. Download APK ke folder apks/
adb pull /data/app/.../base.apk reverse-engineering/trader-family/apks/
```
