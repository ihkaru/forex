//+------------------------------------------------------------------+
//|                                         ForexHexagonBridge.mq5   |
//|                   Zero-Key MetaTrader 5 Socket Bridge for Rust   |
//|                                  Hexagonal Architecture Adapter  |
//+------------------------------------------------------------------+
#property copyright "Forex Quant Research Team"
#property link      "https://github.com/forex"
#property version   "1.00"
#property strict

// WinSock DLL Import untuk komunikasi TCP Socket ke Rust tanpa external library
#import "ws2_32.dll"
   int WSAStartup(ushort wVersionRequested, uchar &lpWSAData[]);
   int WSACleanup();
   int socket(int af, int type, int protocol);
   int connect(int s, uchar &name[], int namelen);
   int send(int s, uchar &buf[], int len, int flags);
   int recv(int s, uchar &buf[], int len, int flags);
   int closesocket(int s);
#import

input string InpHost = "127.0.0.1"; // Host Rust Daemon
input int    InpPort = 5555;        // Port Stream Ticks (TCP)
input int    InpHistoryLimit = 500; // Jumlah candle awal

int socket_handle = -1;
bool is_connected = false;

//+------------------------------------------------------------------+
//| Expert initialization function                                   |
//+------------------------------------------------------------------+
int OnInit()
{
   Print("🚀 Inisialisasi ForexHexagonBridge EA (Zero-Key Socket Bridge)...");
   EventSetTimer(1); // Heartbeat timer
   ConnectToRustDaemon();
   return(INIT_SUCCEEDED);
}

//+------------------------------------------------------------------+
//| Expert deinitialization function                                 |
//+------------------------------------------------------------------+
void OnDeinit(const int reason)
{
   EventKillTimer();
   if(socket_handle != -1) {
      closesocket(socket_handle);
      WSACleanup();
   }
   Print("🛑 ForexHexagonBridge EA dihentikan.");
}

//+------------------------------------------------------------------+
//| Expert tick function (Streaming real-time setiap pergerakan harga)|
//+------------------------------------------------------------------+
void OnTick()
{
   MqlTick last_tick;
   if(SymbolInfoTick(_Symbol, last_tick))
   {
      // Format JSON Payload: Real-time Normalized Tick
      string json_payload = StringFormat(
         "{\"type\":\"TICK\",\"symbol\":\"%s\",\"bid\":%.5f,\"ask\":%.5f,\"spread_pts\":%d,\"time_gmt\":%lld}\n",
         _Symbol,
         last_tick.bid,
         last_tick.ask,
         (int)((last_tick.ask - last_tick.bid) / _Point),
         (long)TimeGMT()
      );
      
      SendSocketData(json_payload);
   }
}

//+------------------------------------------------------------------+
//| Fungsi Helper Koneksi Socket WinSock                             |
//+------------------------------------------------------------------+
void ConnectToRustDaemon()
{
   // Inisialisasi WinSock
   uchar wsa_data[400];
   if(WSAStartup(0x0202, wsa_data) != 0) {
      Print("❌ Gagal WSAStartup");
      return;
   }
   
   socket_handle = socket(2, 1, 6); // AF_INET = 2, SOCK_STREAM = 1, IPPROTO_TCP = 6
   if(socket_handle < 0) {
      Print("❌ Gagal membuat socket");
      return;
   }
   
   // Konfigurasi sockaddr_in
   uchar sockaddr[16];
   ArrayInitialize(sockaddr, 0);
   sockaddr[0] = 2; // AF_INET
   sockaddr[2] = (uchar)(InpPort >> 8);
   sockaddr[3] = (uchar)(InpPort & 0xFF);
   
   // 127.0.0.1
   sockaddr[4] = 127;
   sockaddr[5] = 0;
   sockaddr[6] = 0;
   sockaddr[7] = 1;
   
   int res = connect(socket_handle, sockaddr, 16);
   if(res == 0) {
      is_connected = true;
      Print("✅ Sukses terhubung ke Rust Signal Daemon di ", InpHost, ":", InpPort);
      SendHistoricalCandles();
   } else {
      is_connected = false;
      Print("⚠️ Belum terhubung ke Rust Daemon (Pastikan signal-daemon berjalan).");
   }
}

void SendSocketData(string msg)
{
   if(!is_connected || socket_handle < 0) return;
   
   uchar buf[];
   StringToCharArray(msg, buf, 0, WHOLE_ARRAY, CP_UTF8);
   int len = ArraySize(buf) - 1; // buang null terminator
   
   if(send(socket_handle, buf, len, 0) <= 0) {
      is_connected = false;
      Print("⚠️ Socket disconnect saat kirim data.");
   }
}

void SendHistoricalCandles()
{
   MqlRates rates[];
   ArraySetAsSeries(rates, true);
   int copied = CopyRates(_Symbol, _Period, 0, InpHistoryLimit, rates);
   
   if(copied > 0)
   {
      for(int i = copied - 1; i >= 0; i--)
      {
         string json_candle = StringFormat(
            "{\"type\":\"BAR\",\"symbol\":\"%s\",\"timeframe\":\"%s\",\"open\":%.5f,\"high\":%.5f,\"low\":%.5f,\"close\":%.5f,\"volume\":%d,\"time_gmt\":%lld}\n",
            _Symbol,
            EnumToString(_Period),
            rates[i].open,
            rates[i].high,
            rates[i].low,
            rates[i].close,
            rates[i].tick_volume,
            (long)rates[i].time
         );
         SendSocketData(json_candle);
      }
      PrintFormat("✅ Mengirim %d candle historis %s ke Rust Engine.", copied, _Symbol);
   }
}

void OnTimer()
{
   if(!is_connected) {
      ConnectToRustDaemon();
   }
}
//+------------------------------------------------------------------+
