import {
  CandlestickSeries,
  LineSeries,
  AreaSeries,
  BarSeries,
  BaselineSeries,
  type IChartApi,
  type ISeriesApi,
  type DeepPartial,
  type ChartOptions
} from 'lightweight-charts';
import type { ChartType } from '../domain/models';

/**
 * Default visual configuration and layout options for Lightweight Charts
 */
export function getChartDefaultOptions(containerWidth: number): DeepPartial<ChartOptions> {
  return {
    width: containerWidth,
    height: 500,
    layout: {
      background: { color: '#131722' },
      textColor: '#787b86',
    },
    grid: {
      vertLines: { color: '#1e222d' },
      horzLines: { color: '#1e222d' },
    },
    crosshair: {
      vertLine: {
        color: '#787b86',
        width: 1,
        style: 3,
      },
      horzLine: {
        color: '#787b86',
        width: 1,
        style: 3,
      },
    },
    localization: {
      timeFormatter: (timestamp: number) => {
        const d = new Date(timestamp * 1000);
        return new Intl.DateTimeFormat(undefined, {
          day: '2-digit',
          month: 'short',
          year: 'numeric',
          hour: '2-digit',
          minute: '2-digit',
          hour12: false,
        }).format(d);
      },
    },
    timeScale: {
      borderColor: '#2a2e39',
      timeVisible: true,
      secondsVisible: false,
    },
    rightPriceScale: {
      borderColor: '#2a2e39',
      autoScale: true,
      scaleMargins: {
        top: 0.12,
        bottom: 0.12,
      },
    },
  };
}

/**
 * Factory for creating chart series based on ChartType enum
 */
export function createSeriesForChart(
  chart: IChartApi,
  type: ChartType,
  currentPrice: number
): ISeriesApi<any> {
  if (type === 'LINE') {
    return chart.addSeries(LineSeries, {
      color: '#2962ff',
      lineWidth: 2,
      priceLineVisible: true,
      lastValueVisible: true,
    });
  }

  if (type === 'AREA') {
    return chart.addSeries(AreaSeries, {
      topColor: 'rgba(41, 98, 255, 0.4)',
      bottomColor: 'rgba(41, 98, 255, 0.02)',
      lineColor: '#2962ff',
      lineWidth: 2,
      priceLineVisible: true,
      lastValueVisible: true,
    });
  }

  if (type === 'BARS') {
    return chart.addSeries(BarSeries, {
      upColor: '#089981',
      downColor: '#f23645',
      priceLineVisible: true,
      lastValueVisible: true,
    });
  }

  if (type === 'BASELINE') {
    return chart.addSeries(BaselineSeries, {
      baseValue: { type: 'price', price: currentPrice },
      topFillColor1: 'rgba(8, 153, 129, 0.24)',
      topFillColor2: 'rgba(8, 153, 129, 0.04)',
      bottomFillColor1: 'rgba(242, 54, 69, 0.04)',
      bottomFillColor2: 'rgba(242, 54, 69, 0.24)',
      topLineColor: '#089981',
      bottomLineColor: '#f23645',
      priceLineVisible: true,
      lastValueVisible: true,
    });
  }

  // Default: CandlestickSeries (used for CANDLES, HEIKIN_ASHI, VOLUME_CANDLES)
  return chart.addSeries(CandlestickSeries, {
    upColor: '#089981',
    downColor: '#f23645',
    borderVisible: false,
    wickUpColor: '#089981',
    wickDownColor: '#f23645',
    priceLineVisible: true,
    lastValueVisible: true,
  });
}
