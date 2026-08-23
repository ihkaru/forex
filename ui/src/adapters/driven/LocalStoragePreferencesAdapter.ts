import type { IUserPreferencesPort, UserPreferences } from '../../ports/IUserPreferencesPort';

const PREF_STORAGE_KEY = 'tf_quant_user_preferences_v1';

const DEFAULT_PREFERENCES: UserPreferences = {
  activeStrategyId: 'pola-n-v2',
  activeSymbol: 'XAUUSD',
  activeZoomRange: '1W',
  layerVisibility: {},
};

/**
 * Driven Adapter: LocalStoragePreferencesAdapter
 * Bertanggung jawab tunggal (Single Responsibility) untuk persistensi preferensi user
 * pada browser LocalStorage dengan proteksi fail-safe (SSR / Private Browsing / Corrupt Storage).
 */
export class LocalStoragePreferencesAdapter implements IUserPreferencesPort {
  private inMemoryCache: UserPreferences;

  constructor() {
    this.inMemoryCache = { ...DEFAULT_PREFERENCES };
  }

  loadPreferences(): UserPreferences {
    if (typeof window === 'undefined' || typeof localStorage === 'undefined') {
      return { ...this.inMemoryCache };
    }

    try {
      const raw = localStorage.getItem(PREF_STORAGE_KEY);
      if (!raw) {
        return { ...DEFAULT_PREFERENCES };
      }

      const parsed = JSON.parse(raw);
      if (typeof parsed !== 'object' || parsed === null) {
        return { ...DEFAULT_PREFERENCES };
      }

      const prefs: UserPreferences = {
        activeStrategyId: typeof parsed.activeStrategyId === 'string' && parsed.activeStrategyId ? parsed.activeStrategyId : DEFAULT_PREFERENCES.activeStrategyId,
        activeSymbol: typeof parsed.activeSymbol === 'string' && parsed.activeSymbol ? parsed.activeSymbol.toUpperCase() : DEFAULT_PREFERENCES.activeSymbol,
        activeZoomRange: typeof parsed.activeZoomRange === 'string' ? parsed.activeZoomRange : DEFAULT_PREFERENCES.activeZoomRange,
        layerVisibility: typeof parsed.layerVisibility === 'object' && parsed.layerVisibility !== null ? parsed.layerVisibility : {},
      };

      this.inMemoryCache = prefs;
      return prefs;
    } catch (e) {
      console.warn('⚠️ Gagal membaca preferensi pengguna dari LocalStorage, fallback ke default:', e);
      return { ...DEFAULT_PREFERENCES };
    }
  }

  savePreferences(prefs: Partial<UserPreferences>): void {
    const current = this.loadPreferences();
    const updated: UserPreferences = {
      ...current,
      ...prefs,
      layerVisibility: {
        ...current.layerVisibility,
        ...(prefs.layerVisibility || {}),
      },
    };

    this.inMemoryCache = updated;

    if (typeof window === 'undefined' || typeof localStorage === 'undefined') {
      return;
    }

    try {
      localStorage.setItem(PREF_STORAGE_KEY, JSON.stringify(updated));
    } catch (e) {
      console.warn('⚠️ Gagal menyimpan preferensi pengguna ke LocalStorage:', e);
    }
  }
}
