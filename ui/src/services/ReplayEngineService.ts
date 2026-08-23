import type { Candle } from '../domain/models';
import type { IReplayEnginePort, ReplayState, ReplayStateListener } from '../ports/IReplayEnginePort';

/**
 * Service Layer: ReplayEngineService
 * Menerapkan IReplayEnginePort dan Observer Pattern untuk mengelola state simulasi Bar Replay
 * secara presisi tanpa lookahead bias.
 */
export class ReplayEngineService implements IReplayEnginePort {
  private fullCandles: Candle[] = [];
  private state: ReplayState = {
    isActive: false,
    isSelectingCutPoint: false,
    isPlaying: false,
    currentIndex: 0,
    startIndex: 0,
    totalBars: 0,
    speedMs: 1000,
  };
  private timerId: any = null;
  private listeners = new Set<ReplayStateListener>();

  loadDataset(candles: Candle[]): void {
    this.fullCandles = candles || [];
    this.state.totalBars = this.fullCandles.length;
    if (!this.state.isActive) {
      this.state.currentIndex = Math.max(0, this.fullCandles.length - 1);
      this.state.startIndex = this.state.currentIndex;
      this.updateTimestamps();
    }
  }

  getState(): ReplayState {
    return { ...this.state };
  }

  getSlicedCandles(): Candle[] {
    if (!this.state.isActive || this.fullCandles.length === 0) {
      return this.fullCandles;
    }
    const end = Math.min(this.fullCandles.length, this.state.currentIndex + 1);
    return this.fullCandles.slice(0, end);
  }

  setSelectingCutPoint(selecting: boolean): void {
    this.state.isSelectingCutPoint = selecting;
    this.notify();
  }

  startReplay(cutIndex: number): void {
    if (this.fullCandles.length === 0) return;
    this.pause();

    const clampedIndex = Math.max(0, Math.min(this.fullCandles.length - 1, cutIndex));
    this.state.isActive = true;
    this.state.isSelectingCutPoint = false;
    this.state.startIndex = clampedIndex;
    this.state.currentIndex = clampedIndex;
    this.state.totalBars = this.fullCandles.length;
    this.updateTimestamps();
    this.notify();
  }

  startReplayAtTime(timestampSec: number): void {
    if (this.fullCandles.length === 0) return;
    // Cari candle dengan timestamp terdekat
    let closestIndex = 0;
    let minDiff = Infinity;

    for (let i = 0; i < this.fullCandles.length; i++) {
      const diff = Math.abs(this.fullCandles[i].time - timestampSec);
      if (diff < minDiff) {
        minDiff = diff;
        closestIndex = i;
      }
    }

    this.startReplay(closestIndex);
  }

  stepForward(): void {
    if (!this.state.isActive || this.fullCandles.length === 0) return;
    if (this.state.currentIndex < this.fullCandles.length - 1) {
      this.state.currentIndex++;
      this.updateTimestamps();
      this.notify();
    } else {
      this.pause();
    }
  }

  stepBackward(): void {
    if (!this.state.isActive || this.fullCandles.length === 0) return;
    if (this.state.currentIndex > 10) {
      this.state.currentIndex--;
      this.updateTimestamps();
      this.notify();
    }
  }

  jumpToIndex(index: number): void {
    if (!this.state.isActive || this.fullCandles.length === 0) return;
    this.state.currentIndex = Math.max(0, Math.min(this.fullCandles.length - 1, index));
    this.updateTimestamps();
    this.notify();
  }

  play(): void {
    if (!this.state.isActive || this.state.isPlaying) return;
    if (this.state.currentIndex >= this.fullCandles.length - 1) {
      // Jika sudah di ujung, reset ke awal titik cut
      this.state.currentIndex = this.state.startIndex;
    }

    this.state.isPlaying = true;
    this.notify();

    this.clearInterval();
    this.timerId = setInterval(() => {
      if (this.state.currentIndex < this.fullCandles.length - 1) {
        this.state.currentIndex++;
        this.updateTimestamps();
        this.notify();
      } else {
        this.pause();
      }
    }, this.state.speedMs);
  }

  pause(): void {
    this.clearInterval();
    if (this.state.isPlaying) {
      this.state.isPlaying = false;
      this.notify();
    }
  }

  setSpeed(speedMs: number): void {
    this.state.speedMs = Math.max(100, speedMs);
    if (this.state.isPlaying) {
      this.pause();
      this.play();
    } else {
      this.notify();
    }
  }

  stopReplay(): void {
    this.pause();
    this.state.isActive = false;
    this.state.isSelectingCutPoint = false;
    this.state.currentIndex = Math.max(0, this.fullCandles.length - 1);
    this.state.startIndex = this.state.currentIndex;
    this.updateTimestamps();
    this.notify();
  }

  subscribe(listener: ReplayStateListener): () => void {
    this.listeners.add(listener);
    // Berikan snapshot inisial
    listener(this.getState(), this.getSlicedCandles());
    return () => {
      this.listeners.delete(listener);
    };
  }

  private updateTimestamps(): void {
    if (this.fullCandles.length > 0 && this.state.currentIndex < this.fullCandles.length) {
      const currentCandle = this.fullCandles[this.state.currentIndex];
      this.state.currentTimestamp = currentCandle.time;
      this.state.currentIsoDate = new Date(currentCandle.time * 1000).toISOString().replace('.000Z', 'Z');
    }
  }

  private clearInterval(): void {
    if (this.timerId) {
      clearInterval(this.timerId);
      this.timerId = null;
    }
  }

  private notify(): void {
    const stateSnapshot = this.getState();
    const sliced = this.getSlicedCandles();
    for (const listener of this.listeners) {
      try {
        listener(stateSnapshot, sliced);
      } catch (e) {
        console.error('[ReplayEngineService] Error notifying listener:', e);
      }
    }
  }
}
