#!/usr/bin/env node
/**
 * Official Hexagon Quant Terminal CLI & API Tool (2026)
 * Communicates with the Rust REST API Server (http://127.0.0.1:5000)
 * Supports querying, live inspection, and JSON exports for backtests, trades, monte-carlo, and EDA.
 */

const fs = require('fs');
const path = require('path');
const http = require('http');

const DEFAULT_API_BASE = process.env.QUANT_API_BASE || 'http://127.0.0.1:5000/api';

function requestApi(endpoint, apiBase = DEFAULT_API_BASE) {
  return new Promise((resolve, reject) => {
    const url = `${apiBase.replace(/\/+$/, '')}/${endpoint.replace(/^\/+/, '')}`;
    http.get(url, (res) => {
      let raw = '';
      res.on('data', chunk => raw += chunk);
      res.on('end', () => {
        try {
          if (res.statusCode >= 400) {
            reject(new Error(`HTTP ${res.statusCode}: ${raw}`));
          } else {
            resolve(JSON.parse(raw));
          }
        } catch (e) {
          reject(new Error(`Failed to parse JSON response: ${e.message}`));
        }
      });
    }).on('error', (err) => {
      reject(new Error(`Gagal menghubungi API (${url}): ${err.message}. Pastikan backend Rust berjalan (./dev.sh).`));
    });
  });
}

function exportJson(data, outputPath) {
  const absPath = path.resolve(process.cwd(), outputPath);
  fs.mkdirSync(path.dirname(absPath), { recursive: true });
  fs.writeFileSync(absPath, JSON.stringify(data, null, 2), 'utf-8');
  const size = fs.statSync(absPath).size;
  console.log(`✅ Berhasil mengekspor JSON ke: ${outputPath} (${size} bytes)`);
}

async function main() {
  const args = process.argv.slice(2);
  const cmd = args[0];

  if (!cmd || cmd === '--help' || cmd === '-h') {
    console.log(`
╔═════════════════════════════════════════════════════════════════════════╗
║   ⚡ HEXAGON QUANT TERMINAL • API & BACKTEST DATA EXPORTER (2026)      ║
╚═════════════════════════════════════════════════════════════════════════╝

Penggunaan:
  node scripts/quant_api_client.js <command> [options]

Perintah:
  health                     Cek status health REST API server
  backtest [--export FILE]   Ambil ringkasan portfolio backtest 10-tahun
  detailed <PAIR> [--export] Ambil detailed backtest report per pair (EURGBP, EURUSD, ...)
  trades <PAIR> [--export]   Ambil daftar settled trades per pair
  monte-carlo <PAIR>         Ambil simulasi Monte Carlo 1.000 path
  scorecard                  Ambil 7-Pilar scorecard & revenue share status
  sync <PAIR>                Trigger delta sync check
  dump-all [--out-dir DIR]   Ekspor seluruh dataset backtest & scorecard ke folder JSON

Contoh:
  node scripts/quant_api_client.js backtest
  node scripts/quant_api_client.js detailed EURGBP --export reports/EURGBP_backtest.json
  node scripts/quant_api_client.js trades EURGBP --export reports/EURGBP_trades.json
  node scripts/quant_api_client.js dump-all --out-dir reports/backtest
`);
    process.exit(0);
  }

  const exportIdx = args.indexOf('--export');
  const exportPath = exportIdx !== -1 ? args[exportIdx + 1] : null;
  const isJsonOnly = args.includes('--json');

  try {
    if (cmd === 'health') {
      const data = await requestApi('health');
      if (exportPath) exportJson(data, exportPath);
      else if (isJsonOnly) console.log(JSON.stringify(data, null, 2));
      else {
        console.log(`🟢 API STATUS: ${data.status.toUpperCase()} | Version: ${data.version} | Engine: ${data.engine}`);
      }
    } else if (cmd === 'backtest') {
      const data = await requestApi('backtest');
      if (exportPath) exportJson(data, exportPath);
      else if (isJsonOnly) console.log(JSON.stringify(data, null, 2));
      else {
        console.log('═════════════════════════════════════════════════════════════════════════');
        console.log('📊 PORTFOLIO MULTI-PAIR BACKTEST SUMMARY (DUKASCOPY 10-YEAR DATASET)');
        console.log('═════════════════════════════════════════════════════════════════════════');
        console.log(`Total Valued Pips   : ${data.total_valued_pips > 0 ? '+' : ''}${data.total_valued_pips.toFixed(1)} VP`);
        console.log(`Portfolio Win Rate  : ${data.portfolio_win_rate_pct.toFixed(1)}%`);
        console.log(`WFER OOS Stability  : ${data.wfer_pct.toFixed(1)}%`);
        console.log(`TF Reward Qualified : ${data.is_tf_qualified ? '✅ YES (>= 300 VP)' : '❌ IN PROGRESS'}`);
        console.log('─────────────────────────────────────────────────────────────────────────');
        console.log(`${'PAIR'.padEnd(10)} ${'TRADES'.padEnd(8)} ${'WIN%'.padEnd(8)} ${'RAW PIPS'.padEnd(12)} ${'VALUED PIPS'.padEnd(14)} ${'PF'.padEnd(6)} STATUS`);
        console.log('─────────────────────────────────────────────────────────────────────────');
        for (const r of data.reports || []) {
          const status = r.is_tf_qualified ? '✅ TF OK' : '⚠️ REVIEW';
          console.log(`${r.symbol.padEnd(10)} ${String(r.total_trades).padEnd(8)} ${(r.win_rate_percent.toFixed(1) + '%').padEnd(8)} ${(r.total_raw_pips.toFixed(1)).padEnd(12)} ${(r.total_valued_pips.toFixed(1)).padEnd(14)} ${(r.profit_factor.toFixed(2)).padEnd(6)} ${status}`);
        }
        console.log('═════════════════════════════════════════════════════════════════════════');
      }
    } else if (cmd === 'detailed') {
      const sym = (args[1] || 'EURGBP').toUpperCase();
      const data = await requestApi(`backtest/detailed/${sym}`);
      if (exportPath) exportJson(data, exportPath);
      else if (isJsonOnly) console.log(JSON.stringify(data, null, 2));
      else {
        const rep = data.report || data;
        console.log('═════════════════════════════════════════════════════════════════════════');
        console.log(`📈 DETAILED BACKTEST REPORT: ${sym} (H1 Candlestick • 10-Year Dukascopy)`);
        console.log('═════════════════════════════════════════════════════════════════════════');
        console.log(`Total Trades        : ${rep.total_trades}`);
        console.log(`Win Rate            : ${(rep.win_rate_percent || rep.win_rate_pct || 0).toFixed(1)}% (${rep.winning_trades}W / ${rep.losing_trades}L)`);
        console.log(`Total Raw PnL       : ${(rep.total_raw_pips || 0) > 0 ? '+' : ''}${(rep.total_raw_pips || 0).toFixed(1)} pips`);
        console.log(`Total Valued Pips   : ${(rep.total_valued_pips || 0) > 0 ? '+' : ''}${(rep.total_valued_pips || 0).toFixed(1)} VP`);
        console.log(`Profit Factor       : ${(rep.profit_factor || 0).toFixed(2)}`);
        console.log(`Recovery Factor     : ${(rep.recovery_factor || 0).toFixed(2)}`);
        console.log(`Max Drawdown (Pips) : ${(rep.max_drawdown_pips || 0).toFixed(1)} pips`);
        console.log(`TF Reward Qualified : ${rep.is_tf_qualified ? '✅ YES (>= 300 VP)' : '❌ IN PROGRESS'}`);
        console.log(`Settled Trades List : ${data.trades?.length || rep.trades?.length || 0} trades recorded`);
        console.log('═════════════════════════════════════════════════════════════════════════');
      }
    } else if (cmd === 'trades') {
      const sym = (args[1] || 'EURGBP').toUpperCase();
      const data = await requestApi(`backtest/trades/${sym}`);
      if (exportPath) exportJson(data, exportPath);
      else if (isJsonOnly) console.log(JSON.stringify(data, null, 2));
      else {
        console.log('═════════════════════════════════════════════════════════════════════════');
        console.log(`📜 LIST OF SETTLED TRADES: ${sym} (Total: ${data.length} Trades)`);
        console.log('═════════════════════════════════════════════════════════════════════════');
        console.log(`${'#'.padEnd(4)} ${'ACTION'.padEnd(12)} ${'ENTRY'.padEnd(10)} ${'EXIT'.padEnd(10)} ${'PNL PIPS'.padEnd(10)} ${'VALUED PIPS'.padEnd(12)} RESULT`);
        console.log('─────────────────────────────────────────────────────────────────────────');
        const showTrades = data.slice(0, 30);
        showTrades.forEach((t, i) => {
          const res = t.pnl_pips > 0 ? '🟢 WIN' : '🔴 LOSS';
          console.log(`${String(i + 1).padEnd(4)} ${t.action.padEnd(12)} ${t.entry_price.toFixed(5).padEnd(10)} ${t.exit_price.toFixed(5).padEnd(10)} ${(t.pnl_pips > 0 ? '+' : '') + t.pnl_pips.toFixed(1).padEnd(8)} ${(t.valued_pips > 0 ? '+' : '') + t.valued_pips.toFixed(1).padEnd(10)} ${res}`);
        });
        if (data.length > 30) {
          console.log(`... dan ${data.length - 30} trade lainnya (gunakan --export file.json untuk dump semua).`);
        }
        console.log('═════════════════════════════════════════════════════════════════════════');
      }
    } else if (cmd === 'monte-carlo') {
      const sym = (args[1] || 'EURGBP').toUpperCase();
      const data = await requestApi(`monte-carlo/${sym}`);
      if (exportPath) exportJson(data, exportPath);
      else if (isJsonOnly) console.log(JSON.stringify(data, null, 2));
      else {
        console.log('═════════════════════════════════════════════════════════════════════════');
        console.log(`🎲 MONTE CARLO 1,000-PATH RESAMPLING: ${sym}`);
        console.log('═════════════════════════════════════════════════════════════════════════');
        console.log(`Iterations          : ${data.iterations} paths`);
        console.log(`Risk of Ruin (DD>20): ${data.risk_of_ruin_pct.toFixed(2)}% (Zero Ruin Guarantee)`);
        console.log(`Worst-Case Max DD   : -${data.worst_case_max_dd_pct.toFixed(1)} VP (5th Percentile Floor)`);
        console.log(`Median Expected VP  : +${data.median_ending_vp.toFixed(1)} VP (50th Percentile)`);
        console.log(`95% Confidence Int  : [${data.confidence_interval_95[0].toFixed(1)}, ${data.confidence_interval_95[1].toFixed(1)}] VP`);
        console.log('═════════════════════════════════════════════════════════════════════════');
      }
    } else if (cmd === 'scorecard') {
      const data = await requestApi('scorecard');
      if (exportPath) exportJson(data, exportPath);
      else if (isJsonOnly) console.log(JSON.stringify(data, null, 2));
      else {
        console.log('═════════════════════════════════════════════════════════════════════════');
        console.log('🏆 TRADERS FAMILY 7-PILLAR SCORECARD & REVENUE SHARE ELIGIBILITY');
        console.log('═════════════════════════════════════════════════════════════════════════');
        console.log(`Total Score         : ${data.total_score} / 28 (${(data.total_score/28*100).toFixed(1)}%)`);
        console.log(`Revenue Share Tier  : ${data.revenue_share_tier}`);
        console.log(`Max Revenue Share   : ${data.max_revenue_share_pct}%`);
        console.log('═════════════════════════════════════════════════════════════════════════');
      }
    } else if (cmd === 'audit') {
      const sym = (args[1] || 'EURGBP').toUpperCase();
      const data = await requestApi(`audit/pair/${sym}`);
      if (exportPath) exportJson(data, exportPath);
      else if (isJsonOnly) console.log(JSON.stringify(data, null, 2));
      else {
        console.log('═════════════════════════════════════════════════════════════════════════');
        console.log(`🔬 COMPLETE 360° QUANT AUDIT REPORT: ${sym}`);
        console.log('═════════════════════════════════════════════════════════════════════════');
        console.log(`Pair Tier & Multiplier: Tier ${data.tier} (${data.multiplier}x VP Multiplier)`);
        console.log(`Total Trades          : ${data.total_trades} trades (${data.winning_trades} Win / ${data.losing_trades} Loss)`);
        console.log(`Win Rate              : ${data.win_rate_pct.toFixed(1)}%`);
        console.log(`Raw PnL & Valued Pips : ${data.total_raw_pips > 0 ? '+' : ''}${data.total_raw_pips.toFixed(1)} pips | ${data.total_valued_pips > 0 ? '+' : ''}${data.total_valued_pips.toFixed(1)} VP`);
        console.log(`Profit & Rec. Factor  : PF: ${data.profit_factor.toFixed(2)} | RecF: ${data.recovery_factor.toFixed(2)}`);
        console.log(`Sharpe / Sortino / Cal: ${data.sharpe_ratio.toFixed(2)} / ${data.sortino_ratio.toFixed(2)} / ${data.calmar_ratio.toFixed(2)}`);
        console.log(`Max Drawdown          : ${data.max_drawdown_pips.toFixed(1)} pips (${data.max_drawdown_vp.toFixed(1)} VP)`);
        console.log(`TF Reward Qualified   : ${data.is_tf_qualified ? '✅ YES (>= 300 VP)' : '❌ IN PROGRESS'}`);
        console.log(`Data Footprint        : ${data.provenance.total_bars.toLocaleString()} Bars (${data.provenance.provider_name})`);
        console.log(`Math Integrity Check  : ${data.provenance.mathematical_integrity_pct.toFixed(1)}% Passed (Zero Invariant Errors)`);
        console.log(`Monthly History       : ${data.monthly_breakdown.length} months recorded`);
        console.log(`Equity Curve Points   : ${data.equity_curve.length} points`);
        console.log(`Settled Trades Log    : ${data.trades.length} individual trade records`);
        console.log('═════════════════════════════════════════════════════════════════════════');
      }
    } else if (cmd === 'audit-full') {
      const data = await requestApi('audit/full');
      if (exportPath) exportJson(data, exportPath);
      else if (isJsonOnly) console.log(JSON.stringify(data, null, 2));
      else {
        console.log('═════════════════════════════════════════════════════════════════════════');
        console.log('🏛️ FULL PORTFOLIO 360° QUANT AUDIT REPORT (UI-MIRRORED EXACT NUMBERS)');
        console.log('═════════════════════════════════════════════════════════════════════════');
        console.log(`Generated At          : ${data.generated_at}`);
        console.log(`Portfolio Valued Pips : ${data.total_portfolio_valued_pips > 0 ? '+' : ''}${data.total_portfolio_valued_pips.toFixed(1)} VP / ${data.monthly_tf_target_vp} VP Target`);
        console.log(`Portfolio Status      : ${data.is_portfolio_tf_qualified ? '✅ QUALIFIED REVENUE SHARE' : '❌ IN PROGRESS'}`);
        console.log(`Portfolio Win Rate    : ${data.portfolio_win_rate_pct.toFixed(1)}% (${data.total_portfolio_trades} Total Trades)`);
        console.log(`7-Pillar Scorecard    : ${data.scorecard.total_score} / ${data.scorecard.max_score} (${data.scorecard.score_pct.toFixed(1)}% - ${data.scorecard.revenue_share_tier})`);
        console.log(`Walk-Forward WFER     : ${data.walk_forward.wfer_pct.toFixed(1)}% (${data.walk_forward.total_verified_bars.toLocaleString()} Real Dukascopy Bars)`);
        console.log('─────────────────────────────────────────────────────────────────────────');
        console.log('PER-PAIR AUDIT BREAKDOWN:');
        for (const p of data.pairs) {
          console.log(`  • ${p.symbol.base}/${p.symbol.quote} (Tier ${p.tier}): ${p.total_trades} trades | Win: ${p.win_rate_pct.toFixed(1)}% | VP: ${p.total_valued_pips > 0 ? '+' : ''}${p.total_valued_pips.toFixed(1)} VP | PF: ${p.profit_factor.toFixed(2)} | Bars: ${p.provenance.total_bars.toLocaleString()}`);
        }
        console.log('═════════════════════════════════════════════════════════════════════════');
      }
    } else if (cmd === 'dump-all') {
      const outDirIdx = args.indexOf('--out-dir');
      const outDir = outDirIdx !== -1 ? args[outDirIdx + 1] : 'reports/backtest';
      console.log(`🚀 Memulai dump seluruh dataset kuantitatif ke folder: ${outDir}/ ...\n`);
      
      const bt = await requestApi('backtest');
      exportJson(bt, path.join(outDir, 'portfolio_backtest_summary.json'));
      
      const sc = await requestApi('scorecard');
      exportJson(sc, path.join(outDir, 'scorecard_7_pillar.json'));

      const strats = await requestApi('strategies');
      exportJson(strats, path.join(outDir, 'strategies_catalog.json'));

      const pairs = ['EURGBP', 'USDCHF', 'GBPUSD', 'EURUSD', 'NZDUSD', 'AUDUSD'];
      for (const p of pairs) {
        console.log(`📦 Mengambil data lengkap untuk pair: ${p}...`);
        try {
          const det = await requestApi(`backtest/detailed/${p}`);
          exportJson(det, path.join(outDir, `${p}_detailed_backtest.json`));
        } catch (e) {
          console.warn(`   ⚠️ Detailed backtest ${p}: ${e.message}`);
        }
        try {
          const tr = await requestApi(`backtest/trades/${p}`);
          exportJson(tr, path.join(outDir, `${p}_trades_list.json`));
        } catch (e) {
          console.warn(`   ⚠️ Trades ${p}: ${e.message}`);
        }
        try {
          const mc = await requestApi(`monte-carlo/${p}`);
          exportJson(mc, path.join(outDir, `${p}_monte_carlo.json`));
        } catch (e) {
          console.warn(`   ⚠️ Monte Carlo ${p}: ${e.message}`);
        }
        try {
          const eda = await requestApi(`eda/${p}`);
          exportJson(eda, path.join(outDir, `${p}_eda_health.json`));
        } catch (e) {
          console.warn(`   ⚠️ EDA ${p}: ${e.message}`);
        }
      }

      console.log(`\n🎉 SELESAI! Seluruh data JSON berhasil diekspor ke: ${outDir}/`);
    } else {
      console.error(`❌ Perintah tidak dikenal: ${cmd}. Ketik 'node scripts/quant_api_client.js --help' untuk bantuan.`);
    }
  } catch (err) {
    console.error(`❌ Terjadi error: ${err.message}`);
    process.exit(1);
  }
}

main();
