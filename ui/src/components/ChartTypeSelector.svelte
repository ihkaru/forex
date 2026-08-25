<script lang="ts">
  import {
    ChevronDown,
    Check,
    Star,
    BarChart2,
    TrendingUp,
    Activity,
    Sliders,
    Sparkles,
    Flame
  } from '@lucide/svelte';
  import type { ChartType, ChartTypeOption } from '../domain/models';

  interface Props {
    activeType: ChartType;
    onSelectType: (type: ChartType) => void;
  }

  let {
    activeType = 'CANDLES',
    onSelectType
  }: Props = $props();

  let isOpen = $state(false);
  let favorites = $state<Record<string, boolean>>({
    CANDLES: true,
    VOLUME_CANDLES: true,
    HEIKIN_ASHI: true,
    LINE: true,
  });

  const chartTypes: ChartTypeOption[] = [
    {
      id: 'CANDLES',
      name: 'Candlestick (Japanese Candlesticks)',
      shortLabel: 'Candles',
      category: 'PRICE',
      categoryLabel: 'Price-Based (Harga Pasar Asli)',
      description: 'Lilin OHLC riil standar pasar dengan harga pembukaan, tertinggi, terendah, dan penutupan.',
      hotkey: 'Alt + 1',
    },
    {
      id: 'VOLUME_CANDLES',
      name: 'Volume Candles (Liquidity Highlight)',
      shortLabel: 'Vol Candles',
      category: 'PRICE',
      categoryLabel: 'Price-Based (Harga Pasar Asli)',
      description: 'Candlestick dengan saturasi warna dinamis berbasis likuiditas tick Dukascopy ECN (>1.5x SMA20 menyala neon).',
      hotkey: 'Alt + 2',
    },
    {
      id: 'HEIKIN_ASHI',
      name: 'Heikin Ashi (Smoothed Trend)',
      shortLabel: 'Heikin Ashi',
      category: 'SYNTHETIC',
      categoryLabel: 'Trend-Filtered (Noise Filter)',
      description: 'Candle rata-rata yang menyaring noise fluktuasi minor untuk membaca arah tren makro.',
      hotkey: 'Alt + 3',
      isDerived: true,
    },
    {
      id: 'BARS',
      name: 'Bars (OHLC Western Bar Chart)',
      shortLabel: 'Bars',
      category: 'PRICE',
      categoryLabel: 'Price-Based (Harga Pasar Asli)',
      description: 'Grafik batang vertikal dengan garis tick harga pembukaan di kiri dan penutupan di kanan.',
      hotkey: 'Alt + 4',
    },
    {
      id: 'LINE',
      name: 'Line (Kurva Harga Penutupan)',
      shortLabel: 'Line',
      category: 'LINE',
      categoryLabel: 'Line-Based (Kurva Garis)',
      description: 'Kurva kontinu harga penutupan untuk pembacaan level support/resistance bersih.',
      hotkey: 'Alt + 5',
    },
    {
      id: 'AREA',
      name: 'Area (Gradient Shading)',
      shortLabel: 'Area',
      category: 'LINE',
      categoryLabel: 'Line-Based (Kurva Garis)',
      description: 'Kurva garis harga dengan gradasi bayangan di bawah kurva untuk estetika visual tinggi.',
      hotkey: 'Alt + 6',
    },
    {
      id: 'BASELINE',
      name: 'Baseline (Dua Warna Zona)',
      shortLabel: 'Baseline',
      category: 'LINE',
      categoryLabel: 'Line-Based (Kurva Garis)',
      description: 'Visualisasi deviasi harga terhadap garis dasar rata-rata (Hijau di atas, Merah di bawah).',
      hotkey: 'Alt + 7',
    },
  ];

  let activeOption = $derived(
    chartTypes.find((t) => t.id === activeType) || chartTypes[0]
  );

  let favoriteList = $derived(
    chartTypes.filter((t) => favorites[t.id])
  );

  function toggleFavorite(id: string, e: MouseEvent) {
    e.stopPropagation();
    favorites[id] = !favorites[id];
  }

  function handleSelect(id: ChartType) {
    onSelectType(id);
    isOpen = false;
  }
</script>

<div class="relative font-mono text-xs select-none">
  <!-- Active Chart Type Trigger Button -->
  <button
    onclick={() => (isOpen = !isOpen)}
    class="flex items-center gap-1.5 px-2.5 py-1 rounded-lg font-bold bg-[#131722] hover:bg-[#2a2e39] border border-[#2a2e39] hover:border-[#2962ff]/60 text-white transition-all shadow-sm group"
    title="Pilih Jenis Tampilan Grafik (Candles, Volume Candles, Heikin Ashi, Line, Area, Bars)"
  >
    <div class="flex items-center gap-1">
      {#if activeType === 'LINE' || activeType === 'AREA'}
        <TrendingUp class="w-3.5 h-3.5 text-[#2962ff]" />
      {:else if activeType === 'HEIKIN_ASHI'}
        <Sparkles class="w-3.5 h-3.5 text-[#f5c344]" />
      {:else if activeType === 'VOLUME_CANDLES'}
        <Flame class="w-3.5 h-3.5 text-[#00f2fe]" />
      {:else}
        <BarChart2 class="w-3.5 h-3.5 text-[#089981]" />
      {/if}
      <span class="text-[11px]">{activeOption.shortLabel}</span>
    </div>
    <ChevronDown class="w-3 h-3 text-[#787b86] group-hover:text-white transition-transform {isOpen ? 'rotate-180' : ''}" />
  </button>

  <!-- Flyout Dropdown Menu (TradingView Native Style) -->
  {#if isOpen}
    <!-- Backdrop dismiss -->
    <div
      tabindex="-1"
      role="button"
      onclick={() => (isOpen = false)}
      onkeydown={(e) => { if (e.key === 'Escape') isOpen = false; }}
      class="fixed inset-0 z-40 bg-transparent"
    ></div>

    <div class="absolute left-0 top-full mt-1.5 w-72 bg-[#1e222d] border border-[#2a2e39] rounded-xl shadow-2xl z-50 p-1.5 divide-y divide-[#2a2e39]/60 animate-in fade-in zoom-in-95 duration-100">
      <!-- Section: Quick Favorites if available -->
      {#if favoriteList.length > 0}
        <div class="pb-1 mb-1">
          <div class="text-[9px] font-bold text-[#787b86] px-2 py-1 uppercase tracking-wider flex items-center gap-1">
            <Star class="w-3 h-3 text-[#f5c344] fill-[#f5c344]" />
            <span>Favorit Cepat</span>
          </div>
          <div class="grid grid-cols-2 gap-1">
            {#each favoriteList as fav}
              <button
                onclick={() => handleSelect(fav.id)}
                class="flex items-center justify-between px-2 py-1.5 rounded-lg text-left transition-all {activeType === fav.id ? 'bg-[#2962ff] text-white font-bold' : 'text-[#d1d4dc] hover:bg-[#131722] hover:text-white'}"
              >
                <span class="text-[10px] truncate">{fav.shortLabel}</span>
                {#if activeType === fav.id}
                  <Check class="w-3 h-3 text-white flex-shrink-0" />
                {/if}
              </button>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Section: All Chart Types List -->
      <div class="pt-1 space-y-0.5 max-h-64 overflow-y-auto">
        <div class="text-[9px] font-bold text-[#787b86] px-2 py-1 uppercase tracking-wider">
          Semua Tipe Grafik
        </div>

        {#each chartTypes as type}
          {@const isSelected = activeType === type.id}
          <div
            role="button"
            tabindex="0"
            onclick={() => handleSelect(type.id)}
            onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); handleSelect(type.id); } }}
            class="flex items-center justify-between px-2 py-1.5 rounded-lg hover:bg-[#131722] cursor-pointer transition-colors group {isSelected ? 'bg-[#2962ff]/10 text-white' : 'text-[#d1d4dc]'}"
          >
            <div class="flex items-center gap-2 min-w-0">
              <button
                onclick={(e) => toggleFavorite(type.id, e)}
                class="p-0.5 text-[#787b86] hover:text-[#f5c344] transition-colors"
                title={favorites[type.id] ? 'Hapus dari favorit' : 'Tambah ke favorit'}
              >
                <Star class="w-3 h-3 {favorites[type.id] ? 'text-[#f5c344] fill-[#f5c344]' : 'opacity-20 group-hover:opacity-100'}" />
              </button>

              <div class="min-w-0">
                <div class="flex items-center gap-1.5">
                  <span class="text-[11px] font-bold truncate group-hover:text-[#2962ff] transition-colors {isSelected ? 'text-[#2962ff]' : ''}">
                    {type.name}
                  </span>
                  {#if type.isDerived}
                    <span class="text-[8px] px-1 rounded bg-[#f5c344]/20 text-[#f5c344]">Smoothed</span>
                  {/if}
                </div>
              </div>
            </div>

            {#if isSelected}
              <Check class="w-3.5 h-3.5 text-[#2962ff] flex-shrink-0" />
            {/if}
          </div>
        {/each}
      </div>

      {#if activeOption.isDerived}
        <div class="pt-1.5 px-2 text-[9px] text-[#f5c344] flex items-center gap-1">
          <Sparkles class="w-3 h-3 flex-shrink-0" />
          <span>Heikin Ashi menggunakan rata-rata harga untuk menyaring noise.</span>
        </div>
      {/if}
    </div>
  {/if}
</div>
