# Gamified Virtual Garden Vision (2026-11-19)

Full spec: docs/superpowers/specs/2026-11-19-gamified-virtual-garden-vision.md

## Vision
Replace the dot-and-circle grid canvas with a living, gamified virtual garden: soil-textured beds, illustrated plant SVGs that grow through stages, weather-aware care tasks, time simulation (-3 months to +6 months), light gamification (streaks, milestones).

## Resolved decisions (2026-11-19)
- Art style: clean vector (modern, free) — openfarmcc + Icon Monks + SVGRepo. No paid assets.
- Gamification: streaks/milestones only.
- Time simulation: scrubber on garden page (inline).
- Platform: Tauri desktop only.
- Weather: free API (Open-Meteo, no key needed).
- Auth: none. Profile section in-app for customization/preferences/settings (local only).

## Asset sources (free only)
- openfarmcc/open-crop-icons (open) — crop icons
- days-later/svg-plant (MIT) — procedural SVG plants
- OpenGameArt "Farming Game - Basics" (CC-BY 3.0) — growth states + soil tiles
- Icon Monks (MIT) — garden icons
- SVGRepo — soil/environment icons

## Roadmap phases
1. Visual Upgrade (soil bed + plant icons + growth stages)
2. Growth Engine (auto-advance stages, health decay)
3. Care System (daily tasks, weather reminders, tips, diagnose)
4. Time Simulation (scrubber -3mo to +6mo, what-if)
5. Gamification (streaks, milestones, health indicator, yield estimates)

## Profile section (new requirement)
- Display name/avatar (local only)
- Preferences (units, theme, hemisphere, city, experience level)
- Notification preferences (reminders on/off, weather alerts)
- View milestones and streaks
- Export/import garden data (backup)
- No authentication — device-local only