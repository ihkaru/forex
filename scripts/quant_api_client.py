#!/usr/bin/env python3
"""
Official Hexagon Quant Terminal CLI & API Tool (2026)
Communicates with the Rust REST API Server (http://127.0.0.1:5000)
Supports querying, live inspection, and JSON exports for backtests, trades, monte-carlo, and EDA.
"""

import os
import sys
import json
import argparse
import urllib.request
import urllib.error

DEFAULT_API_BASE = os.environ.get("QUANT_API_BASE", "http://127.0.0.1:5000/api")

def request_api(endpoint: str, api_base: str = DEFAULT_API_BASE):
    url = f"{api_base.rstrip('/')}/{endpoint.lstrip('/')}"
    req = urllib.request.Request(
        url,
        headers={"User-Agent": "Hexagon-Quant-CLI/2026", "Accept": "application/json"}
    )
    try:
        with urllib.request.urlopen(req, timeout=10) as resp:
            data = resp.read().decode('utf-8')
            return json.loads(data)
    except urllib.error.URLError as e:
        print(f"❌ Error connecting to API ({url}): {e}", file=sys.stderr)
        print("💡 Pastikan backend Rust berjalan (./dev.sh atau cargo run -p api-server).", file=sys.stderr)
        sys.exit(1)

def export_json(data, output_path: str):
    os.makedirs(os.path.dirname(os.path.abspath(output_path)), exist_ok=True)
    with open(output_path, 'w', encoding='utf-8') as f:
        json.dump(data, f, indent=2)
    print(f"✅ Berhasil mengekspor JSON ke: {output_path} ({os.path.getsize(output_path)} bytes)")

def cmd_health(args):
    data = request_api("health", args.api_base)
    if args.json or args.export:
        if args.export:
            export_json(data, args.export)
        else:
            print(json.dumps(data, indent=2))
    else:
        print("═══════════════════════════════════════════════════════")
        print(f"🟢 API STATUS: {data.get('status', 'OK').upper()}")
        print(f"   Version : {data.get('version', '0.1.0')}")
        print(f"   Engine  : {data.get('engine', 'Rust Axum Hexagonal')}")
        print("═══════════════════════════════════════════════════════")

def cmd_backtest(args):
    data = request_api("backtest", args.api_base)
    if args.export:
        export_json(data, args.export)
    elif args.json:
        print(json.dumps(data, indent=2))
    else:
        print("═════════════════════════════════════════════════════════════════════════")
        print("📊 PORTFOLIO MULTI-PAIR BACKTEST SUMMARY (DUKASCOPY 10-YEAR DATASET)")
        print("═════════════════════════════════════════════════════════════════════════")
        print(f"Total Valued Pips   : {data.get('total_valued_pips', 0):+.1f} VP")
        print(f"Portfolio Win Rate  : {data.get('portfolio_win_rate_pct', 0):.1f}%")
        print(f"WFER OOS Stability  : {data.get('wfer_pct', 0):.1f}%")
        print(f"TF Reward Qualified : {'✅ YES (>= 300 VP)' if data.get('is_tf_qualified') else '❌ IN PROGRESS'}")
        print("─────────────────────────────────────────────────────────────────────────")
        print(f"{'PAIR':<10} {'TRADES':<8} {'WIN%':<8} {'RAW PIPS':<12} {'VALUED PIPS':<14} {'PF':<6} {'STATUS'}")
        print("─────────────────────────────────────────────────────────────────────────")
        for r in data.get('reports', []):
            status = '✅ TF OK' if r.get('is_tf_qualified') else '⚠️ REVIEW'
            print(f"{r.get('symbol'):<10} {r.get('total_trades'):<8} {r.get('win_rate_percent', 0):<8.1f} {r.get('total_raw_pips', 0):<12.1f} {r.get('total_valued_pips', 0):<14.1f} {r.get('profit_factor', 0):<6.2f} {status}")
        print("═════════════════════════════════════════════════════════════════════════")

def cmd_detailed(args):
    symbol = args.symbol.upper()
    data = request_api(f"backtest/detailed/{symbol}", args.api_base)
    if args.export:
        export_json(data, args.export)
    elif args.json:
        print(json.dumps(data, indent=2))
    else:
        print(f"═════════════════════════════════════════════════════════════════════════")
        print(f"📈 DETAILED BACKTEST REPORT: {symbol} (H1 Candlestick)")
        print(f"═════════════════════════════════════════════════════════════════════════")
        print(f"Total Trades        : {data.get('total_trades', 0)}")
        print(f"Win Rate            : {data.get('win_rate_pct', 0):.1f}%")
        print(f"Total Raw PnL       : {data.get('total_raw_pips', 0):+.1f} pips")
        print(f"Total Valued Pips   : {data.get('total_valued_pips', 0):+.1f} VP")
        print(f"Profit Factor       : {data.get('profit_factor', 0):.2f}")
        print(f"Recovery Factor     : {data.get('recovery_factor', 0):.2f}")
        print(f"Max Drawdown (Pips) : {data.get('max_drawdown_pips', 0):.1f} pips")
        print(f"WFER OOS Stability  : {data.get('wfer_pct', 0):.1f}%")
        print(f"Traders in List     : {len(data.get('trades', []))} settled trades")
        print(f"═════════════════════════════════════════════════════════════════════════")

def cmd_trades(args):
    symbol = args.symbol.upper()
    data = request_api(f"backtest/trades/{symbol}", args.api_base)
    if args.export:
        export_json(data, args.export)
    elif args.json:
        print(json.dumps(data, indent=2))
    else:
        print(f"═════════════════════════════════════════════════════════════════════════")
        print(f"📜 LIST OF SETTLED TRADES: {symbol} (Total: {len(data)} Trades)")
        print(f"═════════════════════════════════════════════════════════════════════════")
        print(f"{'#':<4} {'ACTION':<10} {'ENTRY':<10} {'EXIT':<10} {'PNL PIPS':<10} {'VALUED PIPS':<12} {'RESULT'}")
        print("─────────────────────────────────────────────────────────────────────────")
        for i, t in enumerate(data[: args.limit if args.limit else 50], 1):
            res_icon = '🟢 WIN' if t.get('pnl_pips', 0) > 0 else '🔴 LOSS'
            print(f"{i:<4} {t.get('action', 'BUY_LIMIT'):<10} {t.get('entry_price', 0):<10.5f} {t.get('exit_price', 0):<10.5f} {t.get('pnl_pips', 0):<+10.1f} {t.get('valued_pips', 0):<+12.1f} {res_icon}")
        if len(data) > 50 and not args.limit:
            print(f"... dan {len(data) - 50} trade lainnya (gunakan --limit 500 atau --export file.json)")
        print("═════════════════════════════════════════════════════════════════════════")

def cmd_monte_carlo(args):
    symbol = args.symbol.upper()
    data = request_api(f"monte-carlo/{symbol}", args.api_base)
    if args.export:
        export_json(data, args.export)
    elif args.json:
        print(json.dumps(data, indent=2))
    else:
        print(f"═════════════════════════════════════════════════════════════════════════")
        print(f"🎲 MONTE CARLO 1,000-PATH SIMULATION: {symbol}")
        print(f"═════════════════════════════════════════════════════════════════════════")
        print(f"Iterations          : {data.get('iterations', 1000)} permutations")
        print(f"Risk of Ruin        : {data.get('risk_of_ruin_pct', 0):.2f}%")
        print(f"Worst-Case Max DD   : {data.get('worst_case_max_dd_pct', 0):.1f} VP")
        print(f"Median Expected VP  : {data.get('median_ending_vp', 0):+.1f} VP")
        ci = data.get('confidence_interval_95', [0, 0])
        print(f"95% Confidence Int  : [{ci[0]:.1f}, {ci[1]:.1f}] VP")
        print(f"═════════════════════════════════════════════════════════════════════════")

def cmd_scorecard(args):
    data = request_api("scorecard", args.api_base)
    if args.export:
        export_json(data, args.export)
    elif args.json:
        print(json.dumps(data, indent=2))
    else:
        print("═════════════════════════════════════════════════════════════════════════")
        print("🏆 TRADERS FAMILY 7-PILLAR SCORECARD & REVENUE SHARE ELIGIBILITY")
        print("═════════════════════════════════════════════════════════════════════════")
        print(f"Total Score         : {data.get('total_score', 0)} / 28 ({data.get('total_score', 0)/28*100:.1f}%)")
        print(f"Revenue Share Tier  : {data.get('revenue_share_tier', 'LEGEND_PRIORITY')}")
        print(f"Max Revenue Share   : {data.get('max_revenue_share_pct', 80)}%")
        print("═════════════════════════════════════════════════════════════════════════")

def cmd_sync(args):
    symbol = args.symbol.upper()
    data = request_api(f"market/sync/{symbol}", args.api_base)
    if args.export:
        export_json(data, args.export)
    elif args.json:
        print(json.dumps(data, indent=2))
    else:
        print("═════════════════════════════════════════════════════════════════════════")
        print(f"⚡ DELTA SYNC REPORT: {symbol}")
        print("═════════════════════════════════════════════════════════════════════════")
        print(f"Source              : {data.get('source', 'DukascopyEcn')}")
        print(f"Synced Bars         : {data.get('synced_bars_count', 0)}")
        print(f"Duration            : {data.get('duration_ms', 0)} ms")
        print(f"Status              : {'🟢 100% Up-to-Date' if data.get('is_up_to_date') else '🔄 Synchronized'}")
        print(f"Message             : {data.get('message', '')}")
        print("═════════════════════════════════════════════════════════════════════════")

def cmd_dump_all(args):
    out_dir = args.out_dir or "reports/backtest"
    os.makedirs(out_dir, exist_ok=True)
    print(f"🚀 Memulai export seluruh data kuantitatif ke direktori: {out_dir}/ ...\n")
    
    # 1. Backtest Summary
    bt = request_api("backtest", args.api_base)
    export_json(bt, os.path.join(out_dir, "portfolio_backtest_summary.json"))
    
    # 2. Scorecard
    sc = request_api("scorecard", args.api_base)
    export_json(sc, os.path.join(out_dir, "scorecard_7_pillar.json"))
    
    # 3. Strategies
    strats = request_api("strategies", args.api_base)
    export_json(strats, os.path.join(out_dir, "strategies_catalog.json"))
    
    # 4. Per Pair Detailed, Trades, Monte Carlo, EDA
    pairs = ["EURGBP", "USDCHF", "GBPUSD", "EURUSD", "NZDUSD", "AUDUSD"]
    for p in pairs:
        print(f"\n📦 Mengambil dataset lengkap untuk pair: {p}...")
        try:
            det = request_api(f"backtest/detailed/{p}", args.api_base)
            export_json(det, os.path.join(out_dir, f"{p}_detailed_backtest.json"))
        except Exception as e:
            print(f"   ⚠️ Detailed backtest {p}: {e}")
            
        try:
            tr = request_api(f"backtest/trades/{p}", args.api_base)
            export_json(tr, os.path.join(out_dir, f"{p}_trades_list.json"))
        except Exception as e:
            print(f"   ⚠️ Trades {p}: {e}")
            
        try:
            mc = request_api(f"monte-carlo/{p}", args.api_base)
            export_json(mc, os.path.join(out_dir, f"{p}_monte_carlo.json"))
        except Exception as e:
            print(f"   ⚠️ Monte Carlo {p}: {e}")
            
        try:
            eda = request_api(f"eda/{p}", args.api_base)
            export_json(eda, os.path.join(out_dir, f"{p}_eda_health.json"))
        except Exception as e:
            print(f"   ⚠️ EDA {p}: {e}")

    print(f"\n🎉 SELESAI! Seluruh data JSON berhasil diekspor ke folder: {out_dir}/")

def main():
    parser = argparse.ArgumentParser(
        description="Hexagon Quant Terminal CLI & API Tool (2026)",
        formatter_class=argparse.RawTextHelpFormatter
    )
    parser.add_argument("--api-base", default=DEFAULT_API_BASE, help="REST API Base URL (default: http://127.0.0.1:5000/api)")
    subparsers = parser.add_subparsers(dest="command", help="Perintah yang tersedia")

    # health
    p_health = subparsers.add_parser("health", help="Cek status server API")
    p_health.add_argument("--json", action="store_true", help="Output raw JSON")
    p_health.add_argument("--export", metavar="PATH", help="Simpan hasil ke file JSON")

    # backtest
    p_bt = subparsers.add_parser("backtest", help="Ambil ringkasan portfolio backtest")
    p_bt.add_argument("--json", action="store_true", help="Output raw JSON")
    p_bt.add_argument("--export", metavar="PATH", help="Simpan hasil ke file JSON")

    # detailed
    p_det = subparsers.add_parser("detailed", help="Ambil detailed backtest report per pair")
    p_det.add_argument("symbol", help="Simbol pair (misal: EURGBP, EURUSD, NZDUSD)")
    p_det.add_argument("--json", action="store_true", help="Output raw JSON")
    p_det.add_argument("--export", metavar="PATH", help="Simpan hasil ke file JSON")

    # trades
    p_tr = subparsers.add_parser("trades", help="Ambil daftar settled trades per pair")
    p_tr.add_argument("symbol", help="Simbol pair (misal: EURGBP, EURUSD, NZDUSD)")
    p_tr.add_argument("--limit", type=int, help="Batasi jumlah trade yang ditampilkan di terminal")
    p_tr.add_argument("--json", action="store_true", help="Output raw JSON")
    p_tr.add_argument("--export", metavar="PATH", help="Simpan hasil ke file JSON")

    # monte-carlo
    p_mc = subparsers.add_parser("monte-carlo", help="Ambil laporan simulasi Monte Carlo 1.000 path")
    p_mc.add_argument("symbol", help="Simbol pair (misal: EURGBP, EURUSD, NZDUSD)")
    p_mc.add_argument("--json", action="store_true", help="Output raw JSON")
    p_mc.add_argument("--export", metavar="PATH", help="Simpan hasil ke file JSON")

    # scorecard
    p_sc = subparsers.add_parser("scorecard", help="Ambil skor 7-Pilar Traders Family")
    p_sc.add_argument("--json", action="store_true", help="Output raw JSON")
    p_sc.add_argument("--export", metavar="PATH", help="Simpan hasil ke file JSON")

    # sync
    p_sy = subparsers.add_parser("sync", help="Trigger atau cek delta sync status pair")
    p_sy.add_argument("symbol", help="Simbol pair (misal: EURGBP, EURUSD, NZDUSD)")
    p_sy.add_argument("--json", action="store_true", help="Output raw JSON")
    p_sy.add_argument("--export", metavar="PATH", help="Simpan hasil ke file JSON")

    # dump-all
    p_dump = subparsers.add_parser("dump-all", help="Ekspor seluruh data kuantitatif ke folder JSON")
    p_dump.add_argument("--out-dir", default="reports/backtest", help="Direktori output (default: reports/backtest)")

    args = parser.parse_args()
    if not args.command:
        parser.print_help()
        sys.exit(0)

    dispatch = {
        "health": cmd_health,
        "backtest": cmd_backtest,
        "detailed": cmd_detailed,
        "trades": cmd_trades,
        "monte-carlo": cmd_monte_carlo,
        "scorecard": cmd_scorecard,
        "sync": cmd_sync,
        "dump-all": cmd_dump_all,
    }

    if args.command in dispatch:
        dispatch[args.command](args)

if __name__ == "__main__":
    main()
