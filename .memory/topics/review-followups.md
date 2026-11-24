# Review Follow-ups Audit (2026-11-19)

Plan: docs/superpowers/plans/2026-11-14-review-followups.md

## Chunk A — Layout interaction state
- A1 LayoutState struct: PARTIAL — struct introduced but `_layout_state` unused; `tool`/`drag` still separate signals.
- A2 hit_bed/hit_plant doc comments (top-most, caller order): DONE.

## Chunk B — Canvas zoom & pan
- B1 scale as param to px()/draw_*: NOT DONE — `BASE_SCALE` const still used; scale is CSS-transform only.
- B1 zoom/pan signals + wheel handler: DONE.
- B2 zoom controls UI (+/-/reset + %): DONE.
- Hit-test refinement (canvas_pos): DONE this session (was dividing by zoom incorrectly).

## Chunk C — Frontend image caching
- C1 PlantImageCache struct + provide/use: DONE.
- C2 PlantThumbnail checks cache before IPC: DONE.
- C3 invalidate on override set/clear: N/A — no frontend override management yet.
- C4 tests+build: DONE (trunk build clean).

## Chunk D — Standardized error/loading
- D1 Toast component + mount + show_toast helper: DONE.
- D1 replace .ok() swallows in virtual_garden + layout reload: PARTIAL — persist_beds + garden bootstrap now toast; other sites remain.
- D2 skeleton for image slots: DONE (Suspense fallback animate-pulse).
- D2 soft fallback when all candidates fail: NOT DONE — img src becomes "".

## Chunk E — Onboarding completion
- E1 sections (offline models, attribution, local-first): DONE.
- E1 skip confirmation: DONE this session.
- E1 completion state: NOT DONE.

## Chunk F — Spacing persistence
- F1 migration spacing_cm column: DONE.
- F2 seed.rs includes spacing_cm: DONE.
- F3 catalog/detail queries select it: DONE (plant_meta).
- F4 analyze_layout prefers meta.spacing_cm: DONE.
- F5 full test suite: DONE (165 passed).

## Summary
Done: A2, B (zoom UI + wheel), C (cache), D (toast infra + skeleton), E (info sections + skip confirm), F (all).
Partial: A1 (struct only), B1 (px not parameterized), D1 (toast not on all sites), E (no completion state).
Not done: D2 soft fallback, onboarding completion state, LayoutState wiring, multi-select state machine (feature).