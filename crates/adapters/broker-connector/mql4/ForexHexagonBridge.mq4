//+------------------------------------------------------------------+
//|                                         ForexHexagonBridge.mq4   |
//|               Zero-Key MetaTrader 4 Socket Bridge for Rust Quant |
//|                     Broker MT4 Connector (Demo / Real)           |
//+------------------------------------------------------------------+
#property copyright "Forex Quant Team"
#property version   "1.10"
#property strict

// WinSock DLL Import untuk komunikasi TCP langsung ke Rust Engine
#import "ws2_32.dll"
   int WSAStartup(ushort wVersionRequested, uchar &lpWSAData[]);
   int WSACleanup();
   int socket(int af, int type, int protocol);
   int connect(int s, uchar &name[], int namelen);
   int send(int s, uchar &buf[], int len, int flags);
   int closesocket(int s);
#import

input string InpHost             = "127.0.0.1"; // Host Rust Daemon
input int    InpPort             = 5555;        // Port TCP Bridge (default 5555)
input int    InpHistoryLimit     = 500;         // Jumlah candle H1 yang dikirim per simbol
input bool   InpSyncAllWatchlist = true;        // Sinkronkan seluruh pair di Market Watch (cukup pasang di 1 chart)
input int    InpTimerSeconds     = 2;           // Frekuensi timer update (detik)

int  socket_handle   = -1;
bool is_connected    = false;
bool historical_sent = false;

//+------------------------------------------------------------------+
//| Inisialisasi EA                                                  |
//+------------------------------------------------------------------+
int OnInit()
{
   Print("🚀 Menginisialisasi ForexHexagonBridge ke Rust Quant Server (Host: ", InpHost, ", Port: ", InpPort, ")...");
   EventSetTimer(MathMax(1, InpTimerSeconds));
   ConnectToRustDaemon();
   
   if(is_connected && !historical_sent) {
      SendHistoricalH1Bars();
      historical_sent = true;
   }
   return(INIT_SUCCEEDED);
}

//+------------------------------------------------------------------+
//| Deinitialization                                                 |
//+------------------------------------------------------------------+
void OnDeinit(const int reason)
{
   EventKillTimer();
   if(socket_handle >= 0) {
      closesocket(socket_handle);
      WSACleanup();
   }
   socket_handle   = -1;
   is_connected    = false;
   historical_sent = false;
   Print("🛑 Bridge MT4 Terputus.");
}

//+------------------------------------------------------------------+
//| Timer Heartbeat & Multi-Pair Stream                              |
//+------------------------------------------------------------------+
void OnTimer()
{
   if(!is_connected) {
      ConnectToRustDaemon();
   } else {
      if(InpSyncAllWatchlist) {
         int total = SymbolsTotal(true);
         for(int i = 0; i < total; i++) {
            string sym = SymbolName(i, true);
            SendCurrentTickForSymbol(sym);
            SendCurrentBarForSymbol(sym);
         }
      } else {
         SendCurrentTickForSymbol(_Symbol);
         SendCurrentBarForSymbol(_Symbol);
      }
   }
}

//+------------------------------------------------------------------+
//| Streaming Real-time Per Tick                                     |
//+------------------------------------------------------------------+
void OnTick()
{
   if(!is_connected) {
      ConnectToRustDaemon();
   }

   SendCurrentTickForSymbol(_Symbol);
   SendCurrentBarForSymbol(_Symbol);
}

//+------------------------------------------------------------------+
//| Helper Pengiriman Tick Simbol Tertentu                          |
//+------------------------------------------------------------------+
void SendCurrentTickForSymbol(string sym)
{
   if(!is_connected || socket_handle < 0) return;

   bool is_real = (AccountInfoInteger(ACCOUNT_TRADE_MODE) == ACCOUNT_TRADE_MODE_REAL);
   string src = is_real ? "MrgRealMt4" : "MrgDemoMt4";

   double sym_bid = MarketInfo(sym, MODE_BID);
   double sym_ask = MarketInfo(sym, MODE_ASK);
   double sym_pt  = MarketInfo(sym, MODE_POINT);
   if(sym_pt <= 0.0) sym_pt = 0.00001;
   if(sym_bid <= 0.0 || sym_ask <= 0.0) return;

   string json = StringFormat(
      "{\"type\":\"TICK\",\"symbol\":\"%s\",\"source\":\"%s\",\"server\":\"%s\",\"bid\":%.5f,\"ask\":%.5f,\"spread_pts\":%d,\"time_gmt\":%lld}\n",
      sym,
      src,
      AccountServer(),
      sym_bid,
      sym_ask,
      (int)((sym_ask - sym_bid) / sym_pt),
      (long)TimeGMT()
   );

   SendSocketData(json);
}

//+------------------------------------------------------------------+
//| Helper Pengiriman Bar Berjalan Simbol Tertentu                  |
//+------------------------------------------------------------------+
void SendCurrentBarForSymbol(string sym)
{
   if(!is_connected || socket_handle < 0) return;

   bool is_real = (AccountInfoInteger(ACCOUNT_TRADE_MODE) == ACCOUNT_TRADE_MODE_REAL);
   string src = is_real ? "MrgRealMt4" : "MrgDemoMt4";

   datetime bar_time = iTime(sym, PERIOD_H1, 0);
   if(bar_time <= 0) return;
   double bar_open   = iOpen(sym, PERIOD_H1, 0);
   double bar_high   = iHigh(sym, PERIOD_H1, 0);
   double bar_low    = iLow(sym, PERIOD_H1, 0);
   double bar_close  = iClose(sym, PERIOD_H1, 0);
   long   bar_vol    = iVolume(sym, PERIOD_H1, 0);

   string json = StringFormat(
      "{\"type\":\"BAR\",\"symbol\":\"%s\",\"source\":\"%s\",\"timeframe\":\"H1\",\"open\":%.5f,\"high\":%.5f,\"low\":%.5f,\"close\":%.5f,\"volume\":%.2f,\"time_gmt\":%lld}\n",
      sym,
      src,
      bar_open, bar_high, bar_low, bar_close, (double)bar_vol,
      (long)bar_time
   );

   SendSocketData(json);
}

//+------------------------------------------------------------------+
//| Sinkronisasi Histori Bar H1 untuk Simbol Tertentu                |
//+------------------------------------------------------------------+
void SendHistoricalBarsForSymbol(string sym)
{
   int total_h1_bars = iBars(sym, PERIOD_H1);
   if(total_h1_bars <= 0) return;
   int bars_to_send = MathMin(InpHistoryLimit, total_h1_bars);
   bool is_real = (AccountInfoInteger(ACCOUNT_TRADE_MODE) == ACCOUNT_TRADE_MODE_REAL);
   string src = is_real ? "MrgRealMt4" : "MrgDemoMt4";

   Print("📦 Mengirim ", bars_to_send, " bar histori H1 ", sym, " (Source: ", src, " [", AccountServer(), "]) ke Rust Quant Engine...");

   for(int i = bars_to_send - 1; i >= 0; i--)
   {
      datetime bar_time = iTime(sym, PERIOD_H1, i);
      double bar_open   = iOpen(sym, PERIOD_H1, i);
      double bar_high   = iHigh(sym, PERIOD_H1, i);
      double bar_low    = iLow(sym, PERIOD_H1, i);
      double bar_close  = iClose(sym, PERIOD_H1, i);
      long   bar_vol    = iVolume(sym, PERIOD_H1, i);

      string json = StringFormat(
         "{\"type\":\"BAR\",\"symbol\":\"%s\",\"source\":\"%s\",\"timeframe\":\"H1\",\"open\":%.5f,\"high\":%.5f,\"low\":%.5f,\"close\":%.5f,\"volume\":%.2f,\"time_gmt\":%lld}\n",
         sym,
         src,
         bar_open, bar_high, bar_low, bar_close, (double)bar_vol,
         (long)bar_time
      );
      SendSocketData(json);
   }
}

//+------------------------------------------------------------------+
//| Sinkronisasi Seluruh Candle H1 Histori                           |
//+------------------------------------------------------------------+
void SendHistoricalH1Bars()
{
   if(InpSyncAllWatchlist) {
      int total = SymbolsTotal(true);
      Print("📋 Menyinkronkan seluruh ", total, " simbol dari Market Watch ke Rust Quant Engine...");
      for(int i = 0; i < total; i++) {
         string sym = SymbolName(i, true);
         SendHistoricalBarsForSymbol(sym);
      }
      Print("✅ Sukses sinkronisasi seluruh pair Market Watch ke Rust Engine.");
   } else {
      SendHistoricalBarsForSymbol(_Symbol);
      Print("✅ Sukses sinkronisasi bar H1 ", _Symbol, " ke Rust Engine via TCP socket.");
   }
}

//+------------------------------------------------------------------+
//| Fungsi Helper Socket TCP (WinSock)                               |
//+------------------------------------------------------------------+
void ConnectToRustDaemon()
{
   if(socket_handle >= 0) {
      closesocket(socket_handle);
      socket_handle = -1;
   }

   uchar wsa_data[400];
   if(WSAStartup(0x0202, wsa_data) != 0) return;

   socket_handle = socket(2, 1, 6); // AF_INET, SOCK_STREAM, IPPROTO_TCP
   if(socket_handle < 0) return;

   uchar sockaddr[16];
   ArrayInitialize(sockaddr, 0);
   sockaddr[0] = 2; // AF_INET
   sockaddr[2] = (uchar)(InpPort >> 8);
   sockaddr[3] = (uchar)(InpPort & 0xFF);

   // Parse IPv4 address
   uchar ip_octets[4];
   ParseIpAddress(InpHost, ip_octets);
   sockaddr[4] = ip_octets[0];
   sockaddr[5] = ip_octets[1];
   sockaddr[6] = ip_octets[2];
   sockaddr[7] = ip_octets[3];

   if(connect(socket_handle, sockaddr, 16) == 0) {
      is_connected = true;
      Print("✅ Sukses terhubung ke Rust Server di ", InpHost, ":", InpPort);
      SendHistoricalH1Bars();
      historical_sent = true;
   } else {
      is_connected = false;
      closesocket(socket_handle);
      socket_handle = -1;
   }
}

void ParseIpAddress(string ip_str, uchar &octets[])
{
   ArrayResize(octets, 4);
   octets[0] = 127; octets[1] = 0; octets[2] = 0; octets[3] = 1; // Default fallback

   string parts[];
   int count = StringSplit(ip_str, '.', parts);
   if(count == 4) {
      octets[0] = (uchar)StringToInteger(parts[0]);
      octets[1] = (uchar)StringToInteger(parts[1]);
      octets[2] = (uchar)StringToInteger(parts[2]);
      octets[3] = (uchar)StringToInteger(parts[3]);
   }
}

void SendSocketData(string msg)
{
   if(socket_handle < 0 || !is_connected) return;
   uchar buf[];
   StringToCharArray(msg, buf);
   int len = ArraySize(buf) - 1; // Jangan kirim null terminator string C
   if(len <= 0) return;

   int res = send(socket_handle, buf, len, 0);
   if(res < 0) {
      is_connected = false;
      historical_sent = false;
      closesocket(socket_handle);
      socket_handle = -1;
      Print("⚠️ Socket terputus, akan mencoba rekoneksi otomatis...");
   }
}
