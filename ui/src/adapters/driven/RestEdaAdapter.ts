import type { EdaReport } from '../../domain/models';
import type { IEdaHealthPort } from '../../ports';

export class RestEdaAdapter implements IEdaHealthPort {
  constructor(private readonly baseUrl: string = 'http://127.0.0.1:5000/api') {}

  async getEdaHealth(symbol: string): Promise<EdaReport> {
    const sym = symbol.replace('/', '').toUpperCase();
    const res = await fetch(`${this.baseUrl}/eda/${sym}`);
    if (!res.ok) {
      throw new Error(`[RestEdaAdapter] HTTP error ${res.status} for symbol ${sym}`);
    }
    const data = await res.json();
    return {
      symbol: sym,
      totalCandles: data.total_candles,
      durationDays: data.total_duration_days,
      mathematicalIntegrityPct: data.mathematical_integrity_pct,
      avgBarRangePips: Number(data.avg_bar_range_pips),
      maxSingleBarPips: Number(data.max_single_bar_pips),
      healthStatus: data.health_status,
    };
  }
}
