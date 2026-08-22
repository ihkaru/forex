import type { IChartLayer, ChartLayerContext } from '../../ports/layers';
import { BacktestTradesLayer } from './BacktestTradesLayer';
import { PolaNSwingPointsLayer } from './PolaNSwingPointsLayer';
import { DualEmaLayer } from './DualEmaLayer';
import { ActiveSignalOverlayLayer } from './ActiveSignalOverlayLayer';
import { IctOrderBlockLayer } from './IctOrderBlockLayer';

export class ChartLayerManager {
  private layers: Map<string, IChartLayer> = new Map();

  constructor(initialLayers?: IChartLayer[]) {
    if (initialLayers && initialLayers.length > 0) {
      initialLayers.forEach((l) => this.layers.set(l.id, l));
    } else {
      // Default Strategy-Adaptive Composition
      this.registerLayer(new BacktestTradesLayer());
      this.registerLayer(new PolaNSwingPointsLayer());
      this.registerLayer(new DualEmaLayer());
      this.registerLayer(new IctOrderBlockLayer());
      this.registerLayer(new ActiveSignalOverlayLayer());
    }
  }

  registerLayer(layer: IChartLayer): void {
    this.layers.set(layer.id, layer);
  }

  getLayer(layerId: string): IChartLayer | undefined {
    return this.layers.get(layerId);
  }

  getAllLayers(): IChartLayer[] {
    return Array.from(this.layers.values());
  }

  /**
   * Mengembalikan daftar layer yang relevan untuk strategi yang aktif.
   * Universal layers selalu disertakan, sedangkan layer spesifik hanya muncul jika sesuai.
   */
  getApplicableLayers(strategyId: string, category?: string): IChartLayer[] {
    return Array.from(this.layers.values()).filter((layer) =>
      layer.isApplicableForStrategy(strategyId, category)
    );
  }

  toggleLayer(layerId: string, context?: ChartLayerContext): boolean {
    const layer = this.layers.get(layerId);
    if (layer) {
      if (context) {
        return layer.toggle(context);
      } else {
        layer.visible = !layer.visible;
        return layer.visible;
      }
    }
    return false;
  }

  renderAll(context: ChartLayerContext): void {
    const stratId = context.activeStrategyId || 'pola-n-v2';
    const category = context.activeStrategyCategory;

    for (const layer of this.layers.values()) {
      if (layer.isApplicableForStrategy(stratId, category)) {
        layer.render(context);
      } else {
        layer.clear();
      }
    }
  }

  clearAll(): void {
    for (const layer of this.layers.values()) {
      layer.clear();
    }
  }
}
