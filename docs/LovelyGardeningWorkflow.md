# LovelyGardening Workflows

This document describes the end-to-end workflows supported by LovelyGarden-rs
(Leptos CSR → WASM, shared `garden-core`, Tauri v2 desktop shell, SQLite).

## 1. First run / onboarding

1. App loads settings; if `first_load_complete` is false, the Onboarding page is shown.
2. User picks hemisphere, city, experience level, units.
3. Settings are saved; `first_load_complete` becomes true; the main shell appears.
4. The DB is seeded on first launch (catalog, KB, relations, FTS, tips, guides).

## 2. Virtual Garden (inventory)

- Create/select a garden and bed (a default garden + bed are created automatically).
- Select a plant from the searchable catalog (up to 300 results).
- Add a planting with quantity, source (seed / transplant / cutting), and optional notes.
- Remove a planting.
- View the inventory list with plant thumbnails, bed, quantity, source, status.

> Note: this page is an inventory view. Spatial planning happens in **Layout**.

## 3. Layout planning (canvas grid)

1. Select a garden and layout (a default "Untitled layout" is created if none exists).
2. **Add bed**: drag on the canvas to draw a bed rectangle; set bed sun exposure.
3. **Add plant**: choose a plant from the palette, click on the canvas (snaps to 10 cm grid).
4. **Select / move**: drag beds or plants; positions snap to the 10 cm grid.
5. **Save** persists bed shapes (version bump).
6. Warnings panel updates on analyze: overcrowding, poor companions (antagonists), shade conflicts.
7. Duplicate a layout, compare two layouts (added/removed/moved), export as PNG.
8. Crop rotation plans can be added per garden/season/family.

## 4. Plant knowledge base

- Search plants by name, trait, companion, or pest (FTS).
- Filter by category, sunlight, life cycle.
- Open plant detail: description, stages, seasonality, requirements, companions, antagonists, sources.
- Set/clear a per-plant image override.

## 5. Sowing calendar

- Choose month and hemisphere.
- See eligible/ineligible plants with reasons.

## 6. Guides

- Browse guides by topic, skill level, time.
- Track progress step by step with checklists.

## 7. Journal

- Create/edit/delete entries with plant/bed links, tags, photos.
- Search and filter entries.
- Record measurements (height, yield, flowers) per plant.
- View a plant timeline of measurements + entries.

## 8. Coach (local LLM)

1. Import a GGUF model (text or vision), load/unload it.
2. Create conversations; toggle context flags (Garden, Zone, Tasks).
3. Send messages with optional photo attachments.
4. Generate tasks/journal entries/layouts from an assistant reply via the action buttons.
5. Flag a message with a note.
6. Clear conversation or all conversations.

## 9. Problems

- Log a problem against a planting with a symptom.
- Get candidate causes, treatments, prevention.
- Mark resolved with a note; set follow-up date.

## 10. Seeds & harvest

- Track seed inventory with viability status.
- Record harvests (weight/count) linked to plantings.
- View yield summaries.
- Manage supplies (shopping list, tools with maintenance due dates).

## 11. Settings

- Hemisphere, city, experience level, units, theme.
- Data version is stamped; re-seeding happens when the bundled version is newer.

## Developer workflows

- Install Trunk: `cargo install trunk`
- Build CSS: `pnpm css:build`
- Dev (web): `trunk serve`
- Build (release): `pnpm build`
- Desktop: `cd src-tauri && cargo tauri dev`
- Tests: `cargo test --workspace`
- Lint: `cargo clippy --workspace`
- CI: `.github/workflows/build.yml` runs tests, trunk release build, and `cargo tauri build --no-bundle`.
