# Gamified Virtual Garden — Vision & Roadmap

**Date:** 2026-11-19
**Status:** Vision (not yet implemented)
**Goal:** Transform LovelyGardening from a planning tool into a living, gamified virtual garden that beginners love.

---

## 1. The Vision

> "A garden you can see grow — not a spreadsheet of dots on a grid."

The user plants a seed. Over days and weeks, it sprouts, grows, flowers, and fruits — all visible in the app. The app tells them what to do next ("water today", "harvest in 3 days"), warns them if they miss a step, and lets them scrub time forward to see what their garden will look like in 6 months or back 3 months to review what happened.

This is **not** a game with points and leaderboards. It's a **living simulation** that makes gardening tangible, rewarding, and low-risk for beginners who are afraid of killing plants.

---

## 2. Core Pillars

### 2.1 Visual Garden (not a grid)
- Replace the dot-and-circle canvas with a **soil-textured garden bed** rendered with SVG/CSS.
- Plants are **illustrated icons** (SVG) that change appearance by growth stage (seed → sprout → seedling → mature → flowering → fruiting → harvest).
- Beds look like raised beds or soil plots with subtle texture, borders, and shadows.
- Weather/sky backdrop reflects season (optional, P2).

### 2.2 Growth Over Time
- Each planted item has a **growth timeline** driven by the plant's real-world days-to-maturity.
- Growth stages auto-advance based on elapsed real time (or simulated time).
- Visual changes: seed → sprout → leafy plant → flowering → fruiting → harvest-ready.
- Dead/wilted state if neglected (missed watering, wrong season).

### 2.3 Task & Care System
- **Daily to-do list**: "Water tomatoes", "Thin seedlings", "Harvest lettuce".
- **Weather-aware**: skip watering if rain forecast; delay planting if frost.
- **Reminders**: push notifications (desktop) or in-app badge.
- **Tips**: contextual tips based on what's planted and current stage ("Tomatoes need 6+ hrs sun — your bed gets 4. Consider moving.").
- **Diagnose**: "My leaves are yellow" → symptom → cause → treatment.

### 2.4 Time Simulation
- **Scrubber**: drag a timeline to see the garden at any date from −3 months to +6 months.
- **Fast-forward**: "Show me what this looks like in 6 months."
- **Rewind**: review past states, harvests, and notes.
- **What-if**: simulate planting a new crop and see projected harvest.

### 2.5 Gamification (light touch)
- **Streaks**: "7 days of watering streak" — gentle, not punishing.
- **Milestones**: "First harvest!", "First companion planting", "Survived a frost."
- **Progress visualization**: garden "health" indicator, yield estimates.
- **No leaderboards, no paywalls, no points-for-money.** Just satisfaction and learning.

---

## 3. Asset Strategy

### 3.1 Plant Icons (SVG)
| Source | License | Notes |
|--------|---------|-------|
| [openfarmcc/open-crop-icons](https://github.com/openfarmcc/open-crop-icons) | Open | Commonly cultivated crops, CC-licensed |
| [days-later/svg-plant](https://github.com/days-later/svg-plant) | MIT | Procedurally generated SVG plants — could generate growth stages dynamically |
| [Icon Monks](https://www.iconmonks.co/topics/garden-icons/) | MIT | 15 garden icons, 6 weights |
| [Good Stuff No Nonsense](https://goodstuffnononsense.com/hand-drawn-icons/garden-icons/) | Commercial (paid) | 65 hand-drawn icons — warm, friendly style |
| OpenGameArt "Farming Game - Basics" | CC-BY 3.0 | 4 plants with growth states, soil tiles |

**Recommendation:** Start with openfarmcc icons (crop coverage) + days-later/svg-plant (procedural growth stages). Hand-drawn icons (Good Stuff) for premium feel if budget allows.

### 3.2 Soil/Bed/Environment
- SVG soil texture overlays (SVGRepo has soil icons)
- CSS gradients for soil color (brown → dark brown)
- Raised bed borders: simple rounded rectangles with wood-grain CSS
- Optional: sky gradient backdrop that changes with season

### 3.3 Growth Stage Art
Each plant needs 4–6 stage illustrations:
1. **Seed** (dot in soil)
2. **Sprout** (tiny green shoot)
3. **Seedling** (small plant with leaves)
4. **Mature** (full plant)
5. **Flowering** (flowers visible)
6. **Fruiting/Harvest** (fruits/vegetables visible)

Procedural generation (svg-plant) could create these dynamically from a single seed definition. Alternatively, use 4–6 static SVGs per plant category (leafy, root, fruiting, etc.).

---

## 4. Technical Architecture

### 4.1 Data Model Changes

```
PlantGrowthStage: Seed | Sprout | Seedling | Mature | Flowering | Fruiting | HarvestReady | Wilted | Dead

PlantedItem (extend existing LayoutPlant):
  - planted_at: DateTime
  - growth_stage: PlantGrowthStage (derived from elapsed time + days_to_maturity)
  - last_watered: Option<DateTime>
  - health: f64 (0.0–1.0, decays without care)
  - notes: Vec<Note>

GardenState:
  - simulated_date: DateTime (for time scrubbing)
  - real_date: DateTime
  - tasks: Vec<Task>
  - milestones: Vec<Milestone>
```

### 4.2 Growth Engine (garden-core)
- `growth_stage(plant, elapsed_days, days_to_maturity) -> PlantGrowthStage`
- `health_decay(last_watered, now, plant) -> f64`
- `tasks_for_date(garden, date) -> Vec<Task>`
- `simulate_forward(garden, days) -> GardenState`
- `simulate_backward(garden, days) -> GardenState`

### 4.3 Rendering (Leptos/WASM)
- Replace canvas 2D with **SVG-based garden** (Leptos `view!` with SVG elements).
- Each plant renders as `<svg>` or `<img>` based on growth stage.
- Soil bed: `<rect>` with CSS gradient + texture pattern.
- Time scrubber: range input bound to simulated_date.
- Growth animation: CSS transitions on plant SVG swap.

### 4.4 Time Simulation
- Real time: app uses system clock for growth.
- Simulated time: user drags scrubber; garden renders at that date.
- Constraints: −90 days (3 months back) to +180 days (6 months forward).
- Tasks and tips recompute for simulated date.

---

## 5. Phased Roadmap

### Phase 1: Visual Upgrade (2–3 weeks)
- [ ] Replace canvas grid with SVG soil bed
- [ ] Add plant SVG icons (openfarmcc) at fixed mature size
- [ ] Growth stage display (static: seed/sprout/mature based on planted_at)
- [ ] Remove grid lines; add soil texture CSS

### Phase 2: Growth Engine (2–3 weeks)
- [ ] Add growth_stage to PlantedItem (derived from elapsed time)
- [ ] Auto-advance stages on app load
- [ ] Wilted/dead state if neglected
- [ ] Health decay model

### Phase 3: Care System (2–3 weeks)
- [ ] Daily task list (water, thin, harvest)
- [ ] Weather-aware reminders (skip watering if rain)
- [ ] Contextual tips per plant/stage
- [ ] "Diagnose my plant" symptom → cause → treatment

### Phase 4: Time Simulation (2 weeks)
- [ ] Time scrubber (−3 months to +6 months)
- [ ] Fast-forward / rewind
- [ ] What-if planting simulation
- [ ] Past state review (harvests, notes)

### Phase 5: Gamification (1–2 weeks)
- [ ] Streaks (watering, care)
- [ ] Milestones (first harvest, etc.)
- [ ] Garden health indicator
- [ ] Yield estimates

---

## 6. What to Build First

**Start with Phase 1 + a slice of Phase 2:** Make the garden look like a garden (soil bed + plant icons + growth stages). This is the biggest visual leap and the foundation everything else builds on.

The time simulation and gamification are exciting but depend on the visual + growth engine being solid first.

---

## 7. Resolved Decisions (2026-11-19)

1. **Art style:** Clean vector (modern, minimal). Free icons only — openfarmcc + Icon Monks (MIT) + SVGRepo. No paid assets.
2. **Gamification:** Streaks/milestones only. No XP, levels, or leaderboards.
3. **Time simulation UX:** Scrubber on the garden page (inline, not a separate view).
4. **Platform:** Tauri desktop only. No mobile browser support needed.
5. **Weather:** Free weather API integration (e.g., Open-Meteo, no API key required).
6. **Auth/Accounts:** No sign-ins, no logins, no external account creation. All data stays local. In-app "Profile" section for user customization, preferences, and settings (hemisphere, city, units, theme, experience level — already partially in Settings page).

## 7a. Profile Section (New Requirement)

Add a dedicated Profile/Settings area where the user can:
- Set display name / avatar (local only, no cloud)
- Configure preferences (units, theme, hemisphere, city, experience level)
- Manage notification preferences (reminders on/off, weather alerts)
- View milestones and streaks
- Export/import garden data (backup)
- No authentication — profile is device-local

---

## Sources

- openfarmcc/open-crop-icons (GitHub) — open crop icons
- days-later/svg-plant (GitHub) — procedural SVG plants, MIT
- OpenGameArt "Farming Game - Basics" — growth states + soil tiles, CC-BY 3.0
- SVGRepo — soil icons
- Icon Monks — free garden icons, MIT
- Good Stuff No Nonsense — 65 hand-drawn garden icons (paid)
- UX research brief (2026-11-19-ux-human-centric-design-brief.md) — beginner needs, pain points, design principles