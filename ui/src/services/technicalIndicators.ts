import type { Candle } from '../domain/models';

export function computeLastEma(candleList: Candle[], period: number): number {
  if (candleList.length === 0) return 0;
  if (candleList.length < period) return candleList[candleList.length - 1].close;
  // Windowed to the last 200 bars for O(1) instantaneous calculation on replay
  const list = candleList.length > 200 ? candleList.slice(candleList.length - 200) : candleList;
  const k = 2 / (period + 1);
  let ema = list[0].close;
  for (let i = 1; i < list.length; i++) {
    ema = list[i].close * k + ema * (1 - k);
  }
  return ema;
}

export function computeLastRsi(candleList: Candle[], period: number = 14): number {
  if (candleList.length < period + 1) return 50;
  // Windowed to the last 150 bars for O(1) instantaneous calculation on replay
  const list = candleList.length > 150 ? candleList.slice(candleList.length - 150) : candleList;
  let gains = 0;
  let losses = 0;
  for (let i = 1; i <= period; i++) {
    const diff = list[i].close - list[i - 1].close;
    if (diff >= 0) gains += diff;
    else losses -= diff;
  }
  let avgGain = gains / period;
  let avgLoss = losses / period;

  for (let i = period + 1; i < list.length; i++) {
    const diff = list[i].close - list[i - 1].close;
    if (diff >= 0) {
      avgGain = (avgGain * (period - 1) + diff) / period;
      avgLoss = (avgLoss * (period - 1)) / period;
    } else {
      avgGain = (avgGain * (period - 1)) / period;
      avgLoss = (avgLoss * (period - 1) - diff) / period;
    }
  }
  if (avgLoss === 0) return 100;
  const rs = avgGain / avgLoss;
  return 100 - (100 / (1 + rs));
}
