import type {
  CandlestickData,
  LineData,
  AreaData,
  BarData,
  BaselineData
} from 'lightweight-charts';
import type { Candle, ChartType } from '../domain/models';

export function formatSeriesDataset(type: ChartType, activeCandles: Candle[], currentPrice: number) {
  if (activeCandles.length === 0) return [];

  if (type === 'LINE' || type === 'AREA' || type === 'BASELINE') {
    return activeCandles.map((c) => ({
      time: c.time as any,
      value: c.close,
    }));
  }

  if (type === 'VOLUME_CANDLES') {
    const len = activeCandles.length;
    const volData: any[] = [];
    for (let i = 0; i < len; i++) {
      const c = activeCandles[i];
      const start = Math.max(0, i - 19);
      let sum = 0;
      for (let j = start; j <= i; j++) {
        sum += activeCandles[j].volume || 1.0;
      }
      const avgVol = sum / (i - start + 1);
      const vol = c.volume || 1.0;
      const isHighVol = vol > avgVol * 1.5;
      const isUp = c.close >= c.open;
      const color = isHighVol ? (isUp ? '#00f2fe' : '#ff0055') : (isUp ? '#089981' : '#f23645');

      volData.push({
        time: c.time as any,
        open: c.open,
        high: c.high,
        low: c.low,
        close: c.close,
        color,
        wickColor: color,
      });
    }
    return volData;
  }

  if (type === 'HEIKIN_ASHI') {
    const haData: CandlestickData[] = [];
    for (let i = 0; i < activeCandles.length; i++) {
      const c = activeCandles[i];
      const close = (c.open + c.high + c.low + c.close) / 4;
      const open = i === 0 ? (c.open + c.close) / 2 : (haData[i - 1].open + haData[i - 1].close) / 2;
      const high = Math.max(c.high, open, close);
      const low = Math.min(c.low, open, close);
      haData.push({
        time: c.time as any,
        open,
        high,
        low,
        close,
      });
    }
    return haData;
  }

  // Standard Candles & Bars
  return activeCandles.map((c) => ({
    time: c.time as any,
    open: c.open,
    high: c.high,
    low: c.low,
    close: c.close,
  }));
}

export function formatSingleCandleUpdate(type: ChartType, c: Candle, activeCandles: Candle[]) {
  if (type === 'LINE' || type === 'AREA' || type === 'BASELINE') {
    return {
      time: c.time as any,
      value: c.close,
    };
  }

  if (type === 'VOLUME_CANDLES') {
    const lastIdx = activeCandles.length - 1;
    const start = Math.max(0, lastIdx - 19);
    let sum = 0;
    for (let j = start; j <= lastIdx; j++) {
      sum += activeCandles[j]?.volume || 1.0;
    }
    const avgVol = sum / (lastIdx - start + 1);
    const vol = c.volume || 1.0;
    const isHighVol = vol > avgVol * 1.5;
    const isUp = c.close >= c.open;
    const color = isHighVol ? (isUp ? '#00f2fe' : '#ff0055') : (isUp ? '#089981' : '#f23645');
    return {
      time: c.time as any,
      open: c.open,
      high: c.high,
      low: c.low,
      close: c.close,
      color,
      wickColor: color,
    };
  }

  if (type === 'HEIKIN_ASHI') {
    const lastIdx = activeCandles.length - 1;
    const prev = lastIdx > 0 ? activeCandles[lastIdx - 1] : c;
    const prevClose = (prev.open + prev.high + prev.low + prev.close) / 4;
    const prevOpen = (prev.open + prev.close) / 2;
    const close = (c.open + c.high + c.low + c.close) / 4;
    const open = (prevOpen + prevClose) / 2;
    const high = Math.max(c.high, open, close);
    const low = Math.min(c.low, open, close);
    return {
      time: c.time as any,
      open,
      high,
      low,
      close,
    };
  }

  return {
    time: c.time as any,
    open: c.open,
    high: c.high,
    low: c.low,
    close: c.close,
  };
}
