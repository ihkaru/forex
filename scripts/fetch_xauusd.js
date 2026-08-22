#!/usr/bin/env node
/**
 * Official Dukascopy Bank SA (Geneva, Switzerland) XAUUSD Ingestion Pipeline
 * Connects directly to Dukascopy's global CDN endpoint bypassing ISP DNS poisoning.
 * Range: 2015 s.d. 2026 (10+ Tahun H1 BID Bars)
 */

const https = require('https');
const fs = require('fs');
const path = require('path');

const CLOUDFRONT_IPS = ['3.170.229.74', '3.170.229.120', '3.170.229.22', '3.170.229.69'];
const OUTPUT_FILE = path.resolve(__dirname, '../data/historical/XAUUSD_H1.json');

const START_YEAR = 2015;
const END_YEAR = 2026;

function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

function fetchMonthData(year, month1to12) {
  return new Promise((resolve) => {
    const ip = CLOUDFRONT_IPS[month1to12 % CLOUDFRONT_IPS.length];
    const req = https.request({
      host: ip,
      port: 443,
      path: `/v1/candles/hour/XAU-USD/BID/${year}/${month1to12}`,
      method: 'GET',
      headers: {
        'Host': 'jetta.dukascopy.com',
        'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'
      },
      rejectUnauthorized: false,
      timeout: 10000
    }, (res) => {
      if (res.statusCode !== 200) {
        return resolve(null);
      }
      const chunks = [];
      res.on('data', c => chunks.push(c));
      res.on('end', () => {
        try {
          const raw = JSON.parse(Buffer.concat(chunks).toString('utf8'));
          resolve(raw);
        } catch {
          resolve(null);
        }
      });
    });

    req.on('error', () => resolve(null));
    req.on('timeout', () => { req.destroy(); resolve(null); });
    req.end();
  });
}

function unpackCandles(data) {
  if (!data || !Array.isArray(data.times) || !Number.isFinite(data.timestamp)) {
    return [];
  }
  const length = data.times.length;
  if (length === 0) return [];

  let timestamp = data.timestamp;
  let openUnits = Math.round(data.open / data.multiplier);
  let highUnits = Math.round(data.high / data.multiplier);
  let lowUnits = Math.round(data.low / data.multiplier);
  let closeUnits = Math.round(data.close / data.multiplier);

  const candles = [];

  for (let i = 0; i < length; i++) {
    const timeDelta = data.times[i];
    timestamp += timeDelta * data.shift;
    openUnits += data.opens[i];
    highUnits += data.highs[i];
    lowUnits += data.lows[i];
    closeUnits += data.closes[i];

    const open = Number((openUnits * data.multiplier).toFixed(2));
    const high = Number((highUnits * data.multiplier).toFixed(2));
    const low = Number((lowUnits * data.multiplier).toFixed(2));
    const close = Number((closeUnits * data.multiplier).toFixed(2));
    const vol = data.volumes && data.volumes[i] !== undefined ? Number(data.volumes[i]) : 1.0;

    // Filter invalid flat / off-market zeros
    if (open <= 0 || high <= 0 || low <= 0 || close <= 0 || high < low) {
      continue;
    }

    const epochSecs = Math.floor(timestamp / 1000);
    const isoStr = new Date(timestamp).toISOString().replace('.000Z', 'Z');

    candles.push({
      symbol: { base: 'XAU', quote: 'USD' },
      timeframe: 'H1',
      time: epochSecs,
      timestamp: isoStr,
      source: 'DukascopyEcn',
      open: open.toFixed(2),
      high: high.toFixed(2),
      low: low.toFixed(2),
      close: close.toFixed(2),
      volume: vol.toFixed(1)
    });
  }

  return candles;
}

async function main() {
  console.log('═════════════════════════════════════════════════════════════════════════');
  console.log('🥇 DUKASCOPY BANK SA (GENEVA) XAUUSD (GOLD) INGESTION PIPELINE');
  console.log('═════════════════════════════════════════════════════════════════════════');
  console.log(`Rentang: ${START_YEAR} s/d ${END_YEAR} (10+ Tahun H1 True-Tick BID Data)`);

  const allCandles = [];

  for (let year = START_YEAR; year <= END_YEAR; year++) {
    process.stdout.write(`\n📅 Mengunduh Tahun ${year}: `);
    let yearCandles = 0;

    for (let month = 1; month <= 12; month++) {
      if (year === END_YEAR && month > 8) break; // Sampai Agustus 2026

      let data = await fetchMonthData(year, month);
      if (!data) {
        await sleep(200);
        data = await fetchMonthData(year, month); // 1x retry
      }

      if (data) {
        const unpacked = unpackCandles(data);
        allCandles.push(...unpacked);
        yearCandles += unpacked.length;
        process.stdout.write(`M${month}:${unpacked.length} `);
      } else {
        process.stdout.write(`M${month}:[gap] `);
      }

      await sleep(50);
    }
    process.stdout.write(`| Total ${year}: ${yearCandles} bars`);
  }

  // Deduplicate and sort by epoch timestamp
  const map = new Map();
  for (const c of allCandles) {
    map.set(c.time, c);
  }
  const cleanCandles = Array.from(map.values()).sort((a, b) => a.time - b.time);

  fs.mkdirSync(path.dirname(OUTPUT_FILE), { recursive: true });
  fs.writeFileSync(OUTPUT_FILE, JSON.stringify(cleanCandles, null, 2), 'utf-8');

  console.log('\n\n═════════════════════════════════════════════════════════════════════════');
  console.log(`✅ SELESAI! Berhasil menyimpan ${cleanCandles.length} bar XAUUSD H1 resmi Dukascopy`);
  console.log(`💾 File: ${OUTPUT_FILE}`);
  const first = cleanCandles[0];
  const last = cleanCandles[cleanCandles.length - 1];
  console.log(`⏱️  Rentang: ${first?.timestamp} ($${first?.open}) → ${last?.timestamp} ($${last?.close})`);
  console.log('═════════════════════════════════════════════════════════════════════════');
}

main().catch(err => {
  console.error('Fatal error:', err);
  process.exit(1);
});
