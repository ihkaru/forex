#!/usr/bin/env node
/**
 * Official Dukascopy Bank SA (Geneva, Switzerland) Historical Data Ingestion Pipeline
 * Fetches institutional True-Tick aggregated H1 candlestick dataset for Traders Family pairs.
 * Zero dependency on Yahoo Finance. 100% genuine Swiss Interbank ECN quotes.
 */

process.env.NODE_TLS_REJECT_UNAUTHORIZED = '0';

const fs = require('fs');
const path = require('path');
const { getHistoricRates } = require('dukascopy-node');

const PAIRS = [
  { instrument: 'nzdusd', symbolStr: 'NZDUSD', base: 'NZD', quote: 'USD', decimals: 5 },
  { instrument: 'audusd', symbolStr: 'AUDUSD', base: 'AUD', quote: 'USD', decimals: 5 },
  { instrument: 'eurgbp', symbolStr: 'EURGBP', base: 'EUR', quote: 'GBP', decimals: 5 },
  { instrument: 'usdchf', symbolStr: 'USDCHF', base: 'USD', quote: 'CHF', decimals: 5 },
  { instrument: 'eurusd', symbolStr: 'EURUSD', base: 'EUR', quote: 'USD', decimals: 5 },
  { instrument: 'gbpusd', symbolStr: 'GBPUSD', base: 'GBP', quote: 'USD', decimals: 5 },
  { instrument: 'usdjpy', symbolStr: 'USDJPY', base: 'USD', quote: 'JPY', decimals: 3 },
  { instrument: 'xauusd', symbolStr: 'XAUUSD', base: 'XAU', quote: 'USD', decimals: 2 },
];

const OUTPUT_DIR = path.resolve(__dirname, '../data/historical');
const START_YEAR = 2015;
const END_YEAR = 2026;

async function downloadPair(pair) {
  const { instrument, symbolStr, base, quote } = pair;
  console.log(`\n📡 Mengunduh data resmi Dukascopy Bank SA: ${symbolStr} (${instrument})...`);
  console.log(`   Rentang: ${START_YEAR} s/d ${END_YEAR} (10 Tahun H1 - BID)`);

  const startTime = Date.now();
  const cleanCandles = [];
  let invalidCount = 0;

  try {
    for (let year = START_YEAR; year <= END_YEAR; year++) {
      const from = `${year}-01-01`;
      const to = year === END_YEAR ? '2026-08-22' : `${year + 1}-01-01`;

      try {
        const rawRates = await getHistoricRates({
          instrument,
          dates: { from, to },
          timeframe: 'h1',
          priceType: 'bid',
          volumes: true,
          ignoreFlats: true,
          batchSize: 10,
          pauseBetweenBatchesMs: 50,
        });

        if (rawRates && rawRates.length > 0) {
          for (const bar of rawRates) {
            const ts = bar[0];
            const o = Number(bar[1]);
            const h = Number(bar[2]);
            const l = Number(bar[3]);
            const c = Number(bar[4]);
            const v = bar[5] !== undefined ? Number(bar[5]) : 1.0;

            if (isNaN(o) || isNaN(h) || isNaN(l) || isNaN(c)) {
              invalidCount++;
              continue;
            }

            if (h < l || h < o || h < c || l > o || l > c || o <= 0 || h <= 0 || l <= 0 || c <= 0) {
              invalidCount++;
              continue;
            }

            const epochSecs = Math.floor(ts / 1000);
            const isoStr = new Date(ts).toISOString().replace('.000Z', 'Z');

            const dec = pair.decimals || 5;
            cleanCandles.push({
              symbol: { base, quote },
              timeframe: 'H1',
              time: epochSecs,
              timestamp: isoStr,
              source: 'DukascopyEcn',
              open: o.toFixed(dec),
              high: h.toFixed(dec),
              low: l.toFixed(dec),
              close: c.toFixed(dec),
              volume: (v > 0 ? v : 1.0).toFixed(1),
            });
          }
        }
      } catch (e) {
        console.warn(`   ⚠️ Gagal menarik tahun ${year} untuk ${symbolStr}: ${e.message}`);
      }
    }

    fs.mkdirSync(OUTPUT_DIR, { recursive: true });
    const outFile = path.join(OUTPUT_DIR, `${symbolStr}_H1.json`);
    fs.writeFileSync(outFile, JSON.stringify(cleanCandles, null, 2), 'utf-8');

    const duration = ((Date.now() - startTime) / 1000).toFixed(1);
    console.log(`✅ ${symbolStr}: Berhasil menyimpan ${cleanCandles.length} Dukascopy H1 bars ke ${outFile} (${duration}s)`);
    if (invalidCount > 0) {
      console.warn(`   ⚠️ ${invalidCount} outlier bars difilter.`);
    }
    return cleanCandles.length;
  } catch (err) {
    console.error(`❌ Gagal mengunduh ${symbolStr}:`, err.message);
    return 0;
  }
}

async function main() {
  console.log('═════════════════════════════════════════════════════════════════════════');
  console.log('🇨🇭 DUKASCOPY BANK SA (SWISS) HISTORICAL DATA INGESTION ENGINE');
  console.log('═════════════════════════════════════════════════════════════════════════');
  console.log(`Target: 6 Core Traders Family Pairs (10 Tahun: 2015 – 2026)`);

  let total = 0;
  for (const pair of PAIRS) {
    total += await downloadPair(pair);
  }

  console.log('\n═════════════════════════════════════════════════════════════════════════');
  console.log(`🎉 SELESAI! Total ${total} bar H1 resmi Dukascopy Bank SA berhasil diarsip.`);
  console.log('═════════════════════════════════════════════════════════════════════════');
}

main();
