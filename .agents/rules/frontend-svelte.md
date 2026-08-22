---
description: >-
  Standar frontend Svelte 5 — berlaku saat bekerja dengan file di ui/.
  Mencakup Runes API, Tailwind v4, hexagonal architecture pattern,
  dan aturan zero-dependency domain.
globs: "ui/**"
alwaysApply: false
---

# Frontend Standards Rule — Svelte 5 + Tailwind v4

## Stack Wajib

- **Framework**: Svelte 5 dengan Runes (`$state`, `$derived`, `$props`, `$effect`)
- **Styling**: Tailwind CSS v4 (CSS-First via `@theme`, oklch() colors)
- **Charts**: TradingView Lightweight Charts v5
- **Icons**: Lucide Icons

## Aturan Arsitektur

1. **Interface-First**: Buat TypeScript `interface` di `ui/src/ports/` SEBELUM adapter
2. **No Global State**: DILARANG `window.appState`, singleton, atau global mutable
3. **Composition Root**: Semua wiring di `ui/src/index.ts`
4. **Zero-Dependency Domain**: `ui/src/domain/` TIDAK boleh import framework UI, DOM, atau `fetch`

## Svelte 5 Runes — Wajib Dipakai

```svelte
<script lang="ts">
  // ✅ Svelte 5 Runes
  let count = $state(0);
  let doubled = $derived(count * 2);
  
  // ❌ Svelte 4 (deprecated)
  // let count = 0; // reactive tanpa $state
</script>
```

## Tailwind v4 — CSS-First Config

```css
/* ✅ Tailwind v4: definisi di CSS */
@import "tailwindcss";
@theme {
  --color-primary: oklch(65% 0.25 250);
}

/* ❌ Tailwind v3 config (deprecated) */
/* tailwind.config.js */
```
