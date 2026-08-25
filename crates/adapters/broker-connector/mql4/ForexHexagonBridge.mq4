//+------------------------------------------------------------------+
//|                                         ForexHexagonBridge.mq4   |
//|               Zero-Key MetaTrader 4 Socket Bridge for Rust Quant |
//|                     Broker MRG Traders Family (Demo / Real)      |
//+------------------------------------------------------------------+
#property copyright "Forex Quant Team"
#property version   "1.00"
#property strict

// WinSock DLL Import untuk komunikasi TCP langsung ke Rust
#import "ws2_32.dll"
   int WSAStartup(ushort wVersionRequested, uchar &lpWSAData[]);
   int WSACleanup();
   int socket(int af, int type, int protocol);
   int connect(int s, uchar &name[], int namelen);
   int send(int s, uchar &buf[], int len, int flags);
   int closesocket(int s);
#import

input string InpHost         = "127.0.0.1"; // Host Rust Daemon
input int    InpPort         = 5555;        // Port TCP
input int    InpHistoryLimit = 100000;      // Jumlah maksimal candle H1 yang dikirim

int socket_handle = -1;
bool is_connected = false;

//+------------------------------------------------------------------+
//| Inisialisasi EA                                                  |
//+------------------------------------------------------------------+
//+------------------------------------------------------------------+
//| Inisialisasi EA                                                  |
//+------------------------------------------------------------------+
int OnInit()
{
   Print("🚀 Menghubungkan MRG MT4 ke Rust Quant Server (Port: ", InpPort, ")...");
   DumpHistoricalBarsToFile();
   EventSetTimer(2); // Heartbeat timer setiap 2 detik
   ConnectToRustDaemon();
   
   if(is_connected) {
      SendHistoricalH1Bars();
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
   Print("🛑 Bridge MT4 Terputus.");
}

//+------------------------------------------------------------------+
//| Timer Heartbeat & Auto-Reconnect                                 |
//+------------------------------------------------------------------+
void OnTimer()
{
   if(!is_connected) {
      ConnectToRustDaemon();
      if(is_connected) {
         SendHistoricalH1Bars();
      }
   } else {
      // Kirim live tick heartbeat
      SendCurrentTick();
   }
}

//+------------------------------------------------------------------+
//| Streaming Real-time Per Tick                                     |
//+------------------------------------------------------------------+
void OnTick()
{
   if(!is_connected) {
      ConnectToRustDaemon();
      if(is_connected) {
         SendHistoricalH1Bars();
      } else {
         return;
      }
   }

   SendCurrentTick();
}

void SendCurrentTick()
{
   if(!is_connected || socket_handle < 0) return;

   bool is_real = (AccountInfoInteger(ACCOUNT_TRADE_MODE) == ACCOUNT_TRADE_MODE_REAL);
   string src = is_real ? "MrgRealMt4" : "MrgDemoMt4";

   // Format Payload JSON
   string json = StringFormat(
      "{\"type\":\"TICK\",\"symbol\":\"%s\",\"source\":\"%s\",\"server\":\"%s\",\"bid\":%.5f,\"ask\":%.5f,\"spread_pts\":%d,\"time_gmt\":%lld}\n",
      _Symbol,
      src,
      AccountServer(),
      Bid,
      Ask,
      (int)((Ask - Bid) / _Point),
      (long)TimeGMT()
   );

   SendSocketData(json);
}


//+------------------------------------------------------------------+
//| Sinkronisasi Seluruh Candle H1 Histori                           |
//+------------------------------------------------------------------+
void SendHistoricalH1Bars()
{
   int total_h1_bars = iBars(_Symbol, PERIOD_H1);
   int bars_to_send = MathMin(InpHistoryLimit, total_h1_bars);
   bool is_real = (AccountInfoInteger(ACCOUNT_TRADE_MODE) == ACCOUNT_TRADE_MODE_REAL);
   string src = is_real ? "MrgRealMt4" : "MrgDemoMt4";

   Print("📦 Mengirim seluruh ", bars_to_send, " bar histori H1 ", _Symbol, " (Source: ", src, " [", AccountServer(), "]) ke Rust Quant Engine...");

   for(int i = bars_to_send - 1; i >= 0; i--)
   {
      datetime bar_time = iTime(_Symbol, PERIOD_H1, i);
      double bar_open   = iOpen(_Symbol, PERIOD_H1, i);
      double bar_high   = iHigh(_Symbol, PERIOD_H1, i);
      double bar_low    = iLow(_Symbol, PERIOD_H1, i);
      double bar_close  = iClose(_Symbol, PERIOD_H1, i);
      long   bar_vol    = iVolume(_Symbol, PERIOD_H1, i);

      string json = StringFormat(
         "{\"type\":\"BAR\",\"symbol\":\"%s\",\"source\":\"%s\",\"timeframe\":\"H1\",\"open\":%.5f,\"high\":%.5f,\"low\":%.5f,\"close\":%.5f,\"volume\":%.2f,\"time_gmt\":%lld}\n",
         _Symbol,
         src,
         bar_open, bar_high, bar_low, bar_close, (double)bar_vol,
         (long)bar_time
      );
      SendSocketData(json);
   }
   Print("✅ Sukses sinkronisasi ", bars_to_send, " bar H1 ke Rust Engine.");
   DumpHistoricalBarsToFile();
}

void DumpHistoricalBarsToFile()
{
   int total_h1_bars = iBars(_Symbol, PERIOD_H1);
   int bars_to_send = MathMin(InpHistoryLimit, total_h1_bars);
   bool is_real = (AccountInfoInteger(ACCOUNT_TRADE_MODE) == ACCOUNT_TRADE_MODE_REAL);
   string src = is_real ? "MrgRealMt4" : "MrgDemoMt4";

   string filename = "mrg_" + _Symbol + "_h1.json";
   int file_handle = FileOpen(filename, FILE_WRITE|FILE_TXT);
   if(file_handle != INVALID_HANDLE)
   {
      FileWriteString(file_handle, "[\n");
      for(int i = bars_to_send - 1; i >= 0; i--)
      {
         datetime bar_time = iTime(_Symbol, PERIOD_H1, i);
         double bar_open   = iOpen(_Symbol, PERIOD_H1, i);
         double bar_high   = iHigh(_Symbol, PERIOD_H1, i);
         double bar_low    = iLow(_Symbol, PERIOD_H1, i);
         double bar_close  = iClose(_Symbol, PERIOD_H1, i);
         long   bar_vol    = iVolume(_Symbol, PERIOD_H1, i);

         string bar_json = StringFormat(
            "  {\"time\":%lld,\"source\":\"%s\",\"open\":%.5f,\"high\":%.5f,\"low\":%.5f,\"close\":%.5f,\"volume\":%.2f}%s\n",
            (long)bar_time,
            src,
            bar_open, bar_high, bar_low, bar_close, (double)bar_vol,
            (i == 0 ? "" : ",")
         );
         FileWriteString(file_handle, bar_json);
      }
      FileWriteString(file_handle, "]\n");
      FileClose(file_handle);
      Print("💾 Berhasil menyimpan ", bars_to_send, " bar asli ke MQL4/Files/", filename);
   }
}




//+------------------------------------------------------------------+
//| Fungsi Helper Socket                                             |
//+------------------------------------------------------------------+
void ConnectToRustDaemon()
{
   uchar wsa_data[400];
   if(WSAStartup(0x0202, wsa_data) != 0) return;

   socket_handle = socket(2, 1, 6); // AF_INET, SOCK_STREAM, IPPROTO_TCP
   if(socket_handle < 0) return;

   uchar sockaddr[16];
   ArrayInitialize(sockaddr, 0);
   sockaddr[0] = 2; // AF_INET
   sockaddr[2] = (uchar)(InpPort >> 8);
   sockaddr[3] = (uchar)(InpPort & 0xFF);
   sockaddr[4] = 127; sockaddr[5] = 0; sockaddr[6] = 0; sockaddr[7] = 1; // 127.0.0.1

   if(connect(socket_handle, sockaddr, 16) == 0) {
      is_connected = true;
      Print("✅ Sukses terhubung ke Rust Server di 127.0.0.1:", InpPort);
   } else {
      is_connected = false;
   }
}

void SendSocketData(string msg)
{
   if(socket_handle < 0) return;
   uchar buf[];
   StringToCharArray(msg, buf);
   send(socket_handle, buf, ArraySize(buf) - 1, 0);
}
