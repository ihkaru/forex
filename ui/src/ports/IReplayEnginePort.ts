import type { Candle } from '../domain/models';

export interface ReplayState {
  /** Apakah mode replay sedang aktif */
  isActive: boolean;
  /** Apakah sedang dalam mode memilih titik potong di chart (kursor gunting) */
  isSelectingCutPoint: boolean;
  /** Apakah playback sedang berjalan otomatis */
  isPlaying: boolean;
  /** Index candle saat ini yang sedang ditampilkan pada replay */
  currentIndex: number;
  /** Index titik potong awal (cut point) */
  startIndex: number;
  /** Total seluruh candle yang tersedia untuk replay */
  totalBars: number;
  /** Kecepatan playback per bar dalam milidetik (misal: 500ms, 1000ms) */
  speedMs: number;
  /** Timestamp candle saat ini dalam detik UTC */
  currentTimestamp?: number;
  /** Tanggal/waktu candle yang sedang ditampilkan (ISO format string) */
  currentIsoDate?: string;
}

export type ReplayStateListener = (
  state: ReplayState,
  slicedCandles: Candle[],
  latestCandle?: Candle,
  isStepForward?: boolean
) => void;


/**
 * Interface First Pattern: IReplayEnginePort
 * Mendefinisikan kontrak interface murni untuk simulasi bar-by-bar market replay
 * tanpa lookahead bias.
 */
export interface IReplayEnginePort {
  /** Memuat seluruh candle dataset ke dalam engine */
  loadDataset(candles: Candle[]): void;
  /** Mendapatkan snapshot state replay saat ini */
  getState(): ReplayState;
  /** Mendapatkan array candle terpotong saat ini (0 .. currentIndex) */
  getSlicedCandles(): Candle[];
  /** Mengaktifkan/menonaktifkan mode pemilihan titik potong di chart */
  setSelectingCutPoint(selecting: boolean): void;
  /** Memulai replay pada cut index tertentu */
  startReplay(cutIndex: number): void;
  /** Memulai replay berdasarkan timestamp candle */
  startReplayAtTime(timestampSec: number): void;
  /** Maju tepat 1 bar ke depan */
  stepForward(): void;
  /** Mundur 1 bar ke belakang */
  stepBackward(): void;
  /** Lompat ke index tertentu dalam rentang replay */
  jumpToIndex(index: number): void;
  /** Memulai playback otomatis */
  play(): void;
  /** Menjeda playback otomatis */
  pause(): void;
  /** Mengatur kecepatan playback (dalam ms) */
  setSpeed(speedMs: number): void;
  /** Menghentikan replay dan kembali ke real-time */
  stopReplay(): void;
  /** Mendaftar listener untuk update state & sliced candles (Observer Pattern) */
  subscribe(listener: ReplayStateListener): () => void;
}
