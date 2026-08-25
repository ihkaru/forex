import type { IStrategyPort, IMonteCarloPort, StrategyDescriptor, MonteCarloReport } from '../../ports';

export class RestStrategyAdapter implements IStrategyPort {
  constructor(private readonly baseUrl: string = 'http://127.0.0.1:5000/api') {}

  async getStrategies(): Promise<StrategyDescriptor[]> {
    const res = await fetch(`${this.baseUrl}/strategies`);
    if (!res.ok) {
      throw new Error(`[RestStrategyAdapter] HTTP ${res.status}: ${res.statusText}`);
    }
    const data = await res.json();
    return data.map((s: any) => ({
      id: s.id,
      name: s.name,
      code: s.code,
      description: s.description,
      category: s.category,
      author: s.author,
      winRatePct: Number(s.win_rate_pct),
      profitFactor: Number(s.profit_factor),
      recoveryFactor: Number(s.recovery_factor),
      sharpeRatio: Number(s.sharpe_ratio),
      sortinoRatio: Number(s.sortino_ratio),
      calmarRatio: Number(s.calmar_ratio),
      wferPct: Number(s.wfer_pct),
      isTfCompliant: s.is_tf_compliant,
      supportedSymbols: s.supported_symbols || [],
      isSpecialist: s.is_specialist || false,
      specialistLabel: s.specialist_label || null,
      activeParametersSummary: s.active_parameters_summary || '',
      parameters: s.parameters || [],
    }));
  }

}

export class RestMonteCarloAdapter implements IMonteCarloPort {
  constructor(private readonly baseUrl: string = 'http://127.0.0.1:5000/api') {}

  async getMonteCarloReport(symbol: string): Promise<MonteCarloReport> {
    const res = await fetch(`${this.baseUrl}/monte-carlo/${symbol}`);
    if (!res.ok) {
      throw new Error(`[RestMonteCarloAdapter] HTTP ${res.status}: ${res.statusText}`);
    }
    return await res.json();
  }
}
