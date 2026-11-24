# Project State — LovelyGardening

Last updated: 2026-11-19

## Goal
Leptos/Tauri garden planning app; dev run + review follow-ups in progress.

## Completed
- Dev image loading fix (resource_dir fallback to CARGO_MANIFEST_DIR + asset scope).
- Review root-cause report delivered.
- forReview folder created (15 files).
- Follow-up chunks: spacing persistence (F), image cache (C), toast + skeleton (D partial), zoom/pan (B), LayoutState (A), onboarding info+skip (E partial) — all substantially implemented.
- All tests pass (165), cargo build clean, clippy clean (no warnings).
- Fixed unused import in src/app.rs.
- Fixed clippy unwrap_or_default in src/image_cache.rs.
- Onboarding skip now requires confirmation before skipping.
- Layout save errors now surface via toast.
- Canvas hit-testing now uses base scale only (zoom/pan are CSS-transform only, so dividing by zoom was double-applying and misplacing hits).
- Virtual Garden page: growth stage display (seed→sprout→seedling→growing→flowering→harvest) with time scrubber (-3mo to +6mo) and progress percentage.
- Virtual garden bootstrap surfaces garden-creation errors via toast.
- Multi-select interaction state machine: shift-click add, plain click replace, empty-canvas clear; selected highlight; "Delete selected" button.
- LayoutState wiring: mode + drag unified into single RwSignal<LayoutState>; dead `_layout_state` removed.
- Onboarding completion state: "All set — welcome to your garden!" message on success.
- Soft fallback for failed plant images: leaf placeholder when all candidates fail.
- Gamified virtual garden vision spec written. Decisions locked: clean vector/free icons, streaks-only, inline scrubber, desktop-only, Open-Meteo weather, no auth + in-app profile section.

## Next
- Phase 1: Visual Upgrade — replace canvas grid with SVG soil bed + plant icons + growth stages.
- Profile section: in-app customization/preferences/settings (local only, no auth).
- Full error wiring across all remaining pages (toast on IPC failures) — partially done.
- Canvas zoom/pan rendering: parameterize px()/draw_* with scale (currently CSS-transform only).
- Trunk build smoke — verified clean (1m 34s).

## Notes
- Working tree has uncommitted WIP across many files; do not overwrite without care.
- cargo build --workspace and cargo test --workspace both pass (0 failures).