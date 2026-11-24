# Review Follow-ups Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the deferred high/medium items from the design review: layout interaction state, canvas zoom/pan, frontend image caching, standardized error/loading states, onboarding completion, and spacing persistence.

**Architecture:** Each chunk is independent and produces working, testable software. Backend changes (spacing column, image caching is frontend-only) are additive with migrations. Frontend state is introduced as Leptos signals/contexts without breaking existing IPC.

**Tech Stack:** Rust, Leptos 0.8 (CSR), Tauri 2, Tailwind, SQLite (rusqlite).

---

## Chunk A — Layout interaction state machine

### Task A1: Introduce `LayoutState` in `src/pages/layout.rs`

**Files:**
- Modify: `src/pages/layout.rs`

- [ ] **Step 1:** Add a `LayoutMode` enum (`Place`, `Move`, `Resize`) and a `LayoutState` struct holding `selected_bed: Option<String>`, `selected_plant: Option<String>`, `mode`, `drag_origin: Option<(f64,f64)>`.
- [ ] **Step 2:** Replace scattered `tool: RwSignal<Tool>` + `drag: RwSignal<Option<Drag>>` usage with a single `RwSignal<LayoutState>` while keeping `Tool` mapping for the toolbar.
- [ ] **Step 3:** Update `on_down`/`on_move`/`on_up` to read/write through `LayoutState`; preserve existing bed/plant drag behavior.
- [ ] **Step 4:** Run `cargo test --workspace` and `trunk build`; verify layout page still drags beds/plants.

### Task A2: Expose z-order assumption in `hit_bed`/`hit_plant`

**Files:**
- Modify: `garden-core/src/layout.rs` (doc comments only)

- [ ] **Step 1:** Add doc comments stating iteration is reverse (top-most wins) and callers must pass beds/plants in bottom-to-top order.
- [ ] **Step 2:** Add tests for empty inputs and boundary points.

---

## Chunk B — Canvas zoom & pan

### Task B1: Make scale a signal

**Files:**
- Modify: `src/canvas.rs`, `src/pages/layout.rs`

- [ ] **Step 1:** Change `SCALE` from a const to a parameter: `px(cm, scale)` and `draw_grid(ctx, w, h, scale)`.
- [ ] **Step 2:** In `LayoutPage`, add `scale: RwSignal<f64>` (default 2.0) and `pan: RwSignal<(f64,f64)>`; pass into redraw.
- [ ] **Step 3:** Add mouse-wheel handler on canvas adjusting `scale` (clamp 0.5–6.0), and middle-drag panning.
- [ ] **Step 4:** Tests: unit-test `px`/`snap_to_grid` still; manual smoke of zoom.

### Task B2: Zoom controls UI

**Files:**
- Modify: `src/pages/layout.rs`

- [ ] **Step 1:** Add `+`/`-`/reset buttons next to the toolbar; wire to `scale`.
- [ ] **Step 2:** Show current scale percentage in the status line.

---

## Chunk C — Frontend image caching

### Task C1: Cache resolved `PlantImage` in context

**Files:**
- Create: `src/image_cache.rs`
- Modify: `src/app.rs`, `src/components/plant_image.rs`, `src/pages/plant_kb.rs` (if exists)

- [ ] **Step 1:** Create a `PlantImageCache` struct (RwSignal<HashMap<String, PlantImage>>) provided via Leptos context.
- [ ] **Step 2:** In `PlantThumbnail`, check cache before IPC; store result after fetch.
- [ ] **Step 3:** Invalidate on override set/clear.
- [ ] **Step 4:** Run tests + build.

---

## Chunk D — Standardized error/loading states

### Task D1: Error toast component

**Files:**
- Create: `src/components/toast.rs`
- Modify: `src/components/mod.rs`, `src/app.rs`

- [ ] **Step 1:** Create a `Toast` component driven by a global `RwSignal<Option<String>>`.
- [ ] **Step 2:** Mount in `AppShell`; expose `show_toast(msg)` helper.
- [ ] **Step 3:** Replace `.ok()` swallow sites in `virtual_garden.rs` and `layout.rs` reload paths with toast on error.

### Task D2: Loading skeletons for image slots

**Files:**
- Modify: `src/components/plant_image.rs`

- [ ] **Step 1:** Replace the blank 48×48 div fallback with a pulsing skeleton (`animate-pulse bg-stone-800`).
- [ ] **Step 2:** Add soft fallback color/icon when all image candidates fail.

---

## Chunk E — Onboarding completion

### Task E1: Surface onboarding expectations

**Files:**
- Modify: `src/pages/onboarding.rs`

- [ ] **Step 1:** Add sections explaining offline models, image attribution (link to credits), and local-first data.
- [ ] **Step 2:** Add a "skip" confirmation and completion state.

---

## Chunk F — Spacing persistence

### Task F1: Add `spacing_cm` column to catalog

**Files:**
- Modify: `src-tauri/src/db/migrations.rs`, `src-tauri/src/db/catalog.rs`, `src-tauri/src/db/seed.rs`

- [ ] **Step 1:** Add migration adding `spacing_cm REAL` to `catalog`.
- [ ] **Step 2:** Update `INSERT` in seed.rs to include `spacing_cm`.
- [ ] **Step 3:** Update `get_plant_image_meta`/detail queries to select it.
- [ ] **Step 4:** Update layouts.rs `analyze_layout` to prefer `meta.spacing_cm` over `default_spacing_cm`.
- [ ] **Step 5:** Run full test suite.

---

## Self-Review

- Spec coverage: all six deferred items have chunks.
- Placeholders: none; each task references concrete files.
- Type consistency: `spacing_cm` naming consistent across seed.rs, catalog.rs, layout.rs.

## Execution Handoff

Plan complete and saved. Recommended execution: Subagent-Driven (one subagent per task) with two-stage review between tasks. Alternatively Inline Execution with checkpoints. Which approach?
