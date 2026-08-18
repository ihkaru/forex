import type { Signal } from './models';

export interface PairSpec {
  tier: number;
  multiplier: number;
  pipSize: number;
  minPips: number;
  maxPips: number;
  name: string;
}

export class TfPairSpec {
  static getSpec(symbolStr: string): PairSpec {
    const sym = symbolStr.replace('/', '').toUpperCase();
    switch (sym) {
      case 'NZDUSD':
      case 'AUDUSD':
      case 'EURGBP':
      case 'USDCHF':
        return { tier: 1, multiplier: 2.0, pipSize: 0.00010, minPips: 10.0, maxPips: 200.0, name: 'Tier 1' };
      case 'USDCAD':
      case 'EURUSD':
      case 'GBPUSD':
      case 'NZDJPY':
      case 'CADJPY':
      case 'AUDJPY':
        return { tier: 2, multiplier: 1.5, pipSize: 0.00010, minPips: 15.0, maxPips: 300.0, name: 'Tier 2' };
      case 'USDJPY':
      case 'EURJPY':
      case 'GBPJPY':
      case 'CHFJPY':
      case 'EURNZD':
        return { tier: 3, multiplier: 1.0, pipSize: 0.010, minPips: 20.0, maxPips: 400.0, name: 'Tier 3' };
      case 'XAUUSD':
        return { tier: 4, multiplier: 0.5, pipSize: 0.010, minPips: 30.0, maxPips: 500.0, name: 'Tier 4' };
      default:
        return { tier: 2, multiplier: 1.5, pipSize: 0.00010, minPips: 15.0, maxPips: 300.0, name: 'Tier 2' };
    }
  }

  static priceDiffToPips(diff: number, symbolStr: string): number {
    const spec = this.getSpec(symbolStr);
    return Math.abs(diff) / spec.pipSize;
  }
}

export class TfComplianceGuard {
  static validateSignal(signal: Signal): { isValid: boolean; reason?: string } {
    // Invariant 1: Pending Order Only
    if (!['BUY_LIMIT', 'SELL_LIMIT', 'BUY_STOP', 'SELL_STOP'].includes(signal.action)) {
      return { isValid: false, reason: 'Hanya Pending Order yang diperbolehkan oleh Traders Family' };
    }

    const spec = TfPairSpec.getSpec(signal.symbol);
    const slPips = TfPairSpec.priceDiffToPips(signal.entryPrice - signal.stopLoss, signal.symbol);
    const tpPips = TfPairSpec.priceDiffToPips(signal.takeProfit1 - signal.entryPrice, signal.symbol);

    // Invariant 2: Min/Max Pips
    if (slPips < spec.minPips || slPips > spec.maxPips) {
      return { isValid: false, reason: `SL ${slPips.toFixed(1)} pips di luar batas regulasi ${spec.minPips}-${spec.maxPips}` };
    }

    // Invariant 3: SL <= 1.5 x TP
    if (slPips > tpPips * 1.5) {
      return { isValid: false, reason: 'Stop Loss melebihi 1.5x Take Profit' };
    }

    return { isValid: true };
  }
}
