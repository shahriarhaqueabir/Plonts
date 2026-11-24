# Design Decisions — LovelyGardening Frontend

**Date:** 2026-11-19
**Purpose:** Second-opinion review. Documents the design decisions behind each page and section of the app so a reviewer can evaluate them.

---

## 1. Overall Architecture

- **Stack:** Rust + Leptos 0.8 (CSR → WASM) + Tauri v2 desktop shell + Tailwind CSS 3.4
- **Shared core:** `garden-core` crate used by both web and desktop targets
- **State management:** Leptos signals (`RwSignal`, `LocalResource`) — no external state library
- **IPC:** Tauri commands via `ipc::call` / `ipc::call0` — all backend calls go through this
- **Styling:** Tailwind utility classes, dark theme (stone-900 bg, stone-100 text), garden-green accent (garden-400/500)
- **No accounts/auth:** All data local-first (SQLite in Tauri shell). No login wall.

---

## 2. App Shell (`src/app.rs`)

- **Tab-based navigation:** Left sidebar with 10 tabs (Virtual Garden, Layout, Plant KB, Sowing Calendar, Guides, Journal, Coach, Problems, Seeds & Harvest, Settings). Single `Tab` enum drives view switching.
- **Image cache:** `PlantImageCache` provided via Leptos context at app root — shared across all components.
- **Toast host:** `ToastHost` mounted in `AppShell`, driven by `RwSignal<Option<String>>` context. Used for error feedback.
- **Onboarding gate:** App checks `first_load_complete` in settings; shows `Onboarding` component if false.
- **Settings loaded async:** `spawn_local` fetches settings; shows "Loading..." until resolved.

---

## 3. Onboarding Page (`src/pages/onboarding.rs`)

- **Purpose:** First-run setup. Captures hemisphere, city, experience level, units.
- **Design decisions:**
  - Simple single-page form, no multi-step wizard (reduces friction for beginners).
  - Skip option with confirmation panel (prevents accidental skip).
  - "All set — welcome to your garden!" success message on completion.
  - Offline models, image attribution, local-first data explained in bullet list.
  - Experience level: Beginner/Learning/Confident/Expert (affects content difficulty).
  - Units: Metric/Imperial (affects all displays).
  - No progress bar yet (identified as P0 improvement from Planta critique).
  - No back button (identified as P3 improvement).

---

## 4. Virtual Garden Page (`src/pages/virtual_garden.rs`)

- **Purpose:** Track what's planted. Add plantings, view inventory, see growth stages.
- **Design decisions:**
  - Two-column layout: "Add planting" form (left) + "Inventory" list (right).
  - Growth stage display: emoji + label + day count + progress % per planting.
  - Time scrubber: -90 to +180 days. All stages recompute on scrub.
  - PlantThumbnail with image cache + skeleton loading fallback.
  - GenericLeaf illustration for empty state.
  - Toast on add_planting failure.
  - Bootstrap: auto-creates "My Garden" + "Main bed" if none exist.
  - Remove button per planting (with confirmation? No — currently instant).

---

## 5. Layout Page (`src/pages/layout.rs`)

- **Purpose:** Garden bed planning canvas. Drag-and-drop beds and plants, spacing warnings, crop rotation, version comparison, PNG export.
- **Design decisions:**
  - Canvas: 1000×700 px, soil-colored background, subtle grid.
  - Tools: Add Bed, Add Plant, Select/Move (toolbar buttons).
  - Drag behaviors: NewBed (draw rect), Bed (move), Plant (move).
  - Zoom/pan: CSS transform on wrapper (scale + translate). Zoom buttons +/−/reset + wheel. Hit-testing uses base scale only (zoom is visual).
  - Multi-select: Shift+click adds to selection; plain click replaces; empty canvas clears. Selected items highlighted (yellow ring/border). "Delete selected" button.
  - Unified `LayoutState` signal: mode + drag (single source of truth).
  - Warnings panel: overcrowding, companion conflict, shade conflict.
  - Crop rotation panel: season, plant family, notes.
  - Version comparison: duplicate layout, compare two layouts (added/removed/moved counts).
  - Export: PNG via canvas to_data_url.
  - Bed sun exposure: dropdown (full sun / partial shade / shade) for new beds.
  - Spacing: per-plant spacing_cm from catalog, fallback to category default.

---

## 6. Canvas Module (`src/canvas.rs`)

- **Purpose:** 2D canvas drawing functions for the layout page.
- **Design decisions:**
  - `clear`: warm brown soil background with scattered texture dots.
  - `draw_grid`: very subtle (6% opacity) guide lines.
  - `draw_bed`: soil fill + wood-like border + inner shadow + label.
  - `draw_plant`: spacing circle (green, semi-transparent) + plant emoji icon + label.
  - `plant_emoji`: maps plant name keywords to emoji (🍅🥕🥬 etc.).
  - `px`: converts cm to pixels (BASE_SCALE = 2.0).
  - `snap_to_grid`: rounds to nearest 10cm.

---

## 7. Plant KB Page (`src/pages/plant_kb.rs`)

- **Purpose:** Browse/search plant catalog. View plant details, images, relations.
- **Design decisions:** (Not fully reviewed — check file in review/frontend/src/pages/)

---

## 8. Other Pages

- **Sowing Calendar** (`sowing_calendar.rs`): Calendar view with planting eligibility by hemisphere.
- **Guides** (`guides.rs`): Gardening guides with steps, materials, progress tracking, bookmarks.
- **Journal** (`journal.rs`): Garden journal entries with photos, measurements, timeline, search.
- **Coach** (`coach.rs`): AI chat with local LLM models, conversation management, attachments.
- **Problems** (`problems.rs`): Plant problem diagnosis (symptom → cause → treatment).
- **Seeds & Harvest** (`seeds_harvest.rs`): Seed inventory with viability, harvest tracking.
- **Settings** (`settings.rs`): Theme, units, hemisphere, city, experience level, model management, credits.

---

## 9. Component Decisions

- **PlantImage / PlantThumbnail** (`src/components/plant_image.rs`): Fallback chain (override → species → category → generic). Skeleton loading (animate-pulse). Soft fallback (leaf emoji) when all fail. Client-side cache via `PlantImageCache`.
- **Toast** (`src/components/toast.rs`): Global `RwSignal<Option<String>>` context. Fixed bottom-right, rose-colored, dismissible.
- **GenericLeaf**: Reusable leaf illustration for empty states.

---

## 10. Design System

- **Colors:** Dark theme — stone-900 bg, stone-100 text, garden-400/500 green accent, rose for errors, amber for warnings.
- **Typography:** Tailwind defaults (system fonts), bold uppercase tracking for section headers.
- **Spacing:** Tailwind scale (p-4, gap-2, etc.).
- **Cards:** `.card` class (defined in CSS) — rounded, bordered, padded.
- **Buttons:** `.btn-primary` (green), `.btn-ghost` (transparent hover).
- **Inputs:** `.input` class (defined in CSS).

---

## 11. Known Issues / Open Items

- Toast wiring incomplete on some pages (only layout save + garden bootstrap).
- Canvas zoom/pan is CSS-transform only (not wired into draw functions).
- No progress bar in onboarding.
- No back button in onboarding.
- Plant detail page not fully reviewed.
- Growth engine uses emoji (not SVG icons yet).
- No weather integration yet.
- No task to-do list with reminders yet.
- No "What can I plant today?" quick action yet.

---

## 12. Second-Opinion Questions

1. **Is the tab-based navigation right for beginners?** Research suggests bottom nav or simpler structure. 10 tabs may be overwhelming.
2. **Is the canvas (1000×700 fixed) the right metaphor?** For beginners, a simple "my garden" list with growth stages may be better than a planning canvas.
3. **Should the Virtual Garden page be the home page?** Currently Virtual Garden is the default tab. Good.
4. **Is the dark theme appropriate for a gardening app?** Garden apps often use light/earthy themes. Dark may feel less "garden-like."
5. **Are emoji plant icons acceptable, or should we invest in SVG icons?** User said "clean vector (modern, free icons)" — emoji is a stopgap.
6. **Is the onboarding too long?** 4 fields (hemisphere, city, experience, units) + 3 info bullets. Could be shorter.
7. **Should there be a "What can I plant today?" quick action on the home page?** Research says this is the #1 beginner need.