/*
 * Universal Android SSL Pinning Bypass Script (Frida)
 * Digunakan untuk menangkap traffic HTTPS aplikasi Trader Family via mitmproxy
 *
 * Cara Menjalankan:
 * frida -U -f com.traderfamily.app -l reverse-engineering/trader-family/frida/ssl_unpinning.js
 */

Java.perform(function () {
    console.log("[*] Memulai Frida Universal SSL Unpinning untuk Trader Family...");

    try {
        var TrustManagerImpl = Java.use('com.android.org.conscrypt.TrustManagerImpl');
        TrustManagerImpl.verifyChain.implementation = function (untrustedChain, trustAnchorChain, host, clientAuth, ocspData, tlsSctData) {
            console.log("[+] Conscrypt TrustManagerImpl bypass untuk host: " + host);
            return untrustedChain;
        };
    } catch (err) {
        console.log("[-] TrustManagerImpl not found: " + err);
    }

    try {
        var OkHttpClient = Java.use("okhttp3.OkHttpClient$Builder");
        OkHttpClient.certificatePinner.implementation = function (certificatePinner) {
            console.log("[+] OkHttp3 CertificatePinner bypassed!");
            return this;
        };
    } catch (err) {
        console.log("[-] OkHttp3 not found: " + err);
    }
});
