import type { IChartLayer, ChartLayerContext } from '../../ports/layers';
import { BacktestTradesLayer } from './BacktestTradesLayer';
import { PolaNSwingPointsLayer } from './PolaNSwingPointsLayer';
import { DualEmaLayer } from './DualEmaLayer';
import { ActiveSignalOverlayLayer } from './ActiveSignalOverlayLayer';

export class ChartLayerManager {
  private layers: Map<string, IChartLayer> = new Map();

  constructor(initialLayers?: IChartLayer[]) {
    if (initialLayers && initialLayers.length > 0) {
      initialLayers.forEach((l) => this.layers.set(l.id, l));
    } else {
      // Default Layers Composition
      this.registerLayer(new BacktestTradesLayer());
      this.registerLayer(new PolaNSwingPointsLayer());
      this.registerLayer(new DualEmaLayer());
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

  toggleLayer(layerId: string, context: ChartLayerContext): boolean {
    const layer = this.layers.get(layerId);
    if (layer) {
      return layer.toggle(context);
    }
    return false;
  }

  renderAll(context: ChartLayerContext): void {
    for (const layer of this.layers.values()) {
      layer.render(context);
    }
  }

  clearAll(): void {
    for (const layer of this.layers.values()) {
      layer.clear();
    }
  }
}
