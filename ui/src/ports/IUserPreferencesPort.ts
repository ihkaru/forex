export interface UserPreferences {
  activeStrategyId: string;
  activeSymbol: string;
  activeZoomRange?: string;
  layerVisibility?: Record<string, boolean>;
}

export interface IUserPreferencesPort {
  /** Memuat preferensi terakhir pengguna dengan fallback aman ke default */
  loadPreferences(): UserPreferences;
  /** Menyimpan preferensi pengguna */
  savePreferences(prefs: Partial<UserPreferences>): void;
}
