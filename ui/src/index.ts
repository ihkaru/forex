/**
 * COMPOSITION ROOT (ui/src/index.ts)
 * Wires together Ports, Adapters, and UI Controllers via Pure Constructor Dependency Injection.
 */

import { RestMarketDataAdapter } from './adapters/driven/RestMarketDataAdapter';
import { RestBacktestAdapter } from './adapters/driven/RestBacktestAdapter';

import { RestEdaAdapter } from './adapters/driven/RestEdaAdapter';
import { RestStrategyAdapter, RestMonteCarloAdapter } from './adapters/driven/RestStrategyAdapter';
import { RestTesterAdapter } from './adapters/driven/RestTesterAdapter';
import { RestDeltaSyncAdapter } from './adapters/driven/RestDeltaSyncAdapter';
import { LocalStoragePreferencesAdapter } from './adapters/driven/LocalStoragePreferencesAdapter';
import { TradingViewChartAdapter } from './adapters/driving/TradingViewChartAdapter';
import { ReplayKpiCalculatorService } from './services/ReplayKpiCalculatorService';
import type {
  IMarketDataPort,
  IBacktestPort,
  IEdaHealthPort,
  IStrategyPort,
  IMonteCarloPort,
  IDeltaSyncPort,
  IUserPreferencesPort,
  IReplayKpiPort,
} from './ports';
import type { ITesterPort } from './ports/ITesterPort';

export class AppCompositionRoot {
  public readonly marketDataPort: IMarketDataPort;
  public readonly backtestPort: IBacktestPort;
  public readonly edaPort: IEdaHealthPort;
  public readonly strategyPort: IStrategyPort;
  public readonly monteCarloPort: IMonteCarloPort;
  public readonly testerPort: ITesterPort;
  public readonly deltaSyncPort: IDeltaSyncPort;
  public readonly preferencesPort: IUserPreferencesPort;
  public readonly replayKpiPort: IReplayKpiPort;
  public readonly chartAdapter: TradingViewChartAdapter;

  constructor(apiBaseUrl?: string) {
    const defaultUrl = typeof window !== 'undefined'
      ? `${window.location.protocol}//${window.location.host}/api`
      : 'http://127.0.0.1:5000/api';
    const resolvedUrl = apiBaseUrl || defaultUrl;

    // 1. Instantiate Driven Adapters (I/O) & Services
    this.marketDataPort = new RestMarketDataAdapter(resolvedUrl);
    this.backtestPort = new RestBacktestAdapter(resolvedUrl);

    this.edaPort = new RestEdaAdapter(resolvedUrl);
    this.strategyPort = new RestStrategyAdapter(resolvedUrl);
    this.monteCarloPort = new RestMonteCarloAdapter(resolvedUrl);
    this.testerPort = new RestTesterAdapter(resolvedUrl);
    this.deltaSyncPort = new RestDeltaSyncAdapter(resolvedUrl);
    this.preferencesPort = new LocalStoragePreferencesAdapter();
    this.replayKpiPort = new ReplayKpiCalculatorService();

    // 2. Instantiate Driving Adapters via Composition
    this.chartAdapter = new TradingViewChartAdapter('tv-chart', this.marketDataPort);
  }


  async start(): Promise<void> {
    console.log('🚀 Hexagonal App Composition Root Initialized.');
    this.chartAdapter.init();
    await this.chartAdapter.renderSymbol('XAUUSD');
  }
}
