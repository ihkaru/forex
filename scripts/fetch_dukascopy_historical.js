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
  { instrument: 'nzdusd', symbolStr: 'NZDUSD', base: 'NZD', quote: 'USD' },
  { instrument: 'audusd', symbolStr: 'AUDUSD', base: 'AUD', quote: 'USD' },
  { instrument: 'eurgbp', symbolStr: 'EURGBP', base: 'EUR', quote: 'GBP' },
  { instrument: 'usdchf', symbolStr: 'USDCHF', base: 'USD', quote: 'CHF' },
  { instrument: 'eurusd', symbolStr: 'EURUSD', base: 'EUR', quote: 'USD' },
  { instrument: 'gbpusd', symbolStr: 'GBPUSD', base: 'GBP', quote: 'USD' },
];

const OUTPUT_DIR = path.resolve(__dirname, '../data/historical');
// Rentang waktu: 10 Tahun Data Historis Modern (2015 s/d 2026)
const FROM_DATE = new Date('2015-01-01T00:00:00Z');
const TO_DATE = new Date('2026-08-22T00:00:00Z');

async function downloadPair(pair) {
  const { instrument, symbolStr, base, quote } = pair;
  console.log(`\n📡 Mengunduh data resmi Dukascopy Bank SA: ${symbolStr} (${instrument})...`);
  console.log(`   Rentang: ${FROM_DATE.toISOString().slice(0, 10)} s/d ${TO_DATE.toISOString().slice(0, 10)} (H1 - BID)`);

  const startTime = Date.now();
  try {
    const rawRates = await getHistoricRates({
      instrument,
      dates: {
        from: FROM_DATE,
        to: TO_DATE,
      },
      timeframe: 'h1',
      priceType: 'bid',
      volumes: true,
      ignoreFlats: true,
      batchSize: 15,
      pauseBetweenBatchesMs: 200,
    });

    if (!rawRates || rawRates.length === 0) {
      console.error(`❌ Gagal: 0 bar diterima untuk ${symbolStr}`);
      return 0;
    }

    const cleanCandles = [];
    let invalidCount = 0;

    for (const bar of rawRates) {
      // Format: [timestamp, open, high, low, close, volume]
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

      // Mathematical Invariants Check
      if (h < l || h < o || h < c || l > o || l > c || o <= 0 || h <= 0 || l <= 0 || c <= 0) {
        invalidCount++;
        continue;
      }

      const isoStr = new Date(ts).toISOString().replace('.000Z', 'Z');

      cleanCandles.push({
        symbol: { base, quote },
        timeframe: 'H1',
        timestamp: isoStr,
        open: o.toFixed(5),
        high: h.toFixed(5),
        low: l.toFixed(5),
        close: c.toFixed(5),
        volume: (v > 0 ? v : 1.0).toFixed(1),
      });
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
