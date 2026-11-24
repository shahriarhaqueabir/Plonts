# UX Research Brief — Human-Centric Design for First-Time Gardeners

**Date:** 2026-11-19
**Goal:** Make LovelyGardening intuitive and confidence-building for first-time gardeners.
**Sources:** 12+ UX case studies, academic review analysis (PLOS One, 2024), user surveys (n=48–30), usability tests (6–30 participants).

---

## 1. Who We're Designing For

**Primary persona: "Novice Madison"** — first-time gardener, small space (balcony/patio/pots), 0–3 hrs/week, overwhelmed by information, wants a quick win, needs guidance not jargon. Self-identifies as "novice." Motivated by food quality, enjoyment, and the challenge, but low confidence.

**Secondary: "Intermediate Sam"** — has grown a few things, wants reminders and companion-planting guidance, takes photos, needs unit conversion and weather-aware scheduling.

**Key insight from research:** Beginners don't plan. They don't create documents or calendars. They search Google/YouTube/Facebook in the moment and forget. The app must replace that fragmented, in-the-moment searching with a single, trusted, location-aware companion.

---

## 2. Top Pain Points (Ranked by Frequency)

| # | Pain Point | Evidence |
|---|-----------|----------|
| 1 | **Information overload / complexity** | Beginners "fail to start" due to conflicting advice; "most time it just dies" (Vegetopia interviews) |
| 2 | **Low confidence** | "Just tell me what to do" / "Make it as easy as building Legos" (Vegetopia) |
| 3 | **Timeliness** | Miss a key event → plant dies; reminders welcomed by 4/5 (Theresa Fischbach, PLOS One) |
| 4 | **Curated, personalized info** | Internet is "aimed at experienced gardeners"; need location/space-specific guidance |
| 5 | **Planning is minimal** | 11/14 beginners do minimal planning; no calendars or diagrams (Vegetopia) |
| 6 | **Remembering tasks** | "I'm a keen gardener but have a lousy memory" (PLOS One reviews) |
| 7 | **In-garden access** | "You don't want to traipse inside in dirty clothes to look things up" (Fischbach) |
| 8 | **Data loss / slow apps** | Users furious when garden data disappears after updates |
| 9 | **Overwhelm from too much detail** | "Might work OK for veteran gardeners, but not much help for newbies" (PLOS One) |
| 10 | **Account/login pressure** | "NO way I'm giving you my birthdate just to look up plant info" (PLOS One) |

---

## 3. Design Principles (Human-Centric)

1. **One plant to start.** Don't ask for a whole garden. Offer "add your first plant" — quick win builds confidence.
2. **"What can I plant today?"** — A single, prominent action. No planning required. Location + date → suggestions.
3. **Ease-of-growing rating.** Show beginners which plants are easiest. "Ease of growing" score on plant cards.
4. **Step-by-step like Legos.** Bite-sized tasks with visual emphasis. "Just tell me what to do next."
5. **Reminders that guide, not nag.** Weather-aware, user-controllable, tick-off-able. "Make it feel like a hobby, not a chore."
6. **Progressive disclosure.** Short summaries first; details on demand. Categories, not walls of text.
7. **Plain language, no jargon.** AA contrast, large touch targets, plain speech (accessibility).
8. **Emotional support.** "Don't worry!" messages, encouragement, confidence-building. 80% of beginners think care = water + sunlight; education is needed.
9. **In-garden ready.** Mobile-first, readable in sunlight, works offline, no login wall.
10. **No data loss.** Local-first (already have this). Never lose garden data.
11. **Quick notes + photos.** Easy capture, editable, organized — 4/5 take photos already.
12. **Companion planting as "friends/enemies."** Simple, visual, actionable.

---

## 4. Recommended UX Changes (Prioritized)

### P0 — High Impact, Low Effort
- [ ] **"Start here" empty state.** Replace blank "Nothing planted yet" with a guided CTA: "Add your first plant — we'll help you pick." (Garden Buddy: empty state looked like loading.)
- [ ] **Ease-of-growing badge** on plant cards (Easy / Moderate / Advanced).
- [ ] **"Plant now" tag** on plant cards that are in season for the user's hemisphere.
- [ ] **Progress bar in onboarding** (Planta critique: no feedback on progress → feels longer).
- [ ] **Checkboxes not radio buttons** when multiple selections allowed (Planta critique).
- [ ] **Remove login wall** — app already local-first; ensure no account prompt gates core features.

### P1 — High Impact, Medium Effort
- [ ] **"What can I plant today?" quick action** on the home/virtual garden page — location + date → top 3 suggestions.
- [ ] **Task to-do list with weather-aware reminders** — daily bite-sized tasks per plant.
- [ ] **Progressive disclosure on plant detail** — collapsible sections (About / Ideal conditions / Pests / Recipes).
- [ ] **Companion planting "friends/enemies"** visual on layout canvas (already have warnings; make them friendlier).
- [ ] **Photo + note capture** with edit support and organization.
- [ ] **Unit conversion** (metric/imperial toggle already exists; ensure all displays respect it).

### P2 — Medium Impact, Higher Effort
- [ ] **Location-based suggestions** — use hemisphere + city (already captured in onboarding) to filter plants and timing.
- [ ] **"Diagnose my plant" feature** — symptom → cause → treatment (problems page exists; make it beginner-friendly with photos).
- [ ] **Visual garden layout with drag-and-drop** (layout page exists; refine for beginners).
- [ ] **Hamburger menu audit** — ensure it's functional or replace with bottom nav (Rooted critique).
- [ ] **Button contrast / touch target audit** (Rooted: 8.4/10 but button contrast was a complaint).

### P3 — Polish
- [ ] **Encouragement messages** ("Don't worry!", "Great start!", success animations).
- [ ] **Accessibility audit** — AA contrast, plain speech, large touch targets (Apple HIG).
- [ ] **Empty-state illustrations** (GenericLeaf exists; use more).
- [ ] **Onboarding back button** (Adelia: users want to go back if they change their mind).

---

## 5. What LovelyGardening Already Does Well

- ✅ Local-first, no account required
- ✅ Onboarding captures hemisphere, city, experience level, units
- ✅ Plant catalog with search, filters, companion/antagonist warnings
- ✅ Layout canvas with drag-and-drop beds/plants, spacing circles, warnings
- ✅ Toast notifications for errors
- ✅ Image caching with skeleton loading
- ✅ Skip confirmation in onboarding
- ✅ Multi-select + delete on layout canvas

---

## 6. Next Steps

1. **Prototype the "What can I plant today?" flow** — highest leverage for beginner confidence.
2. **Add ease-of-growing badges** to plant cards — quick win, low effort.
3. **Implement task to-do list** with weather-aware reminders — addresses the #1 unmet need (timeliness).
4. **Run a usability test** with 5–6 first-time gardeners on the current build to validate assumptions.

---

## Sources

- PLOS One (2024): "Urban gardening education: User reflections on mobile application designs" — 2,000+ app reviews analyzed
- Vegetopia UX case study (James Lung) — interviews with beginners/intermediates
- Budding Gardener (Theresa Fischbach) — 4 interviews, surveys
- Seedless (Calli Ryan) — guerilla usability testing, 12 participants
- Rooted (Carly Chan) — novice plant owner interviews, 5-user usability test
- Sprout (Kyle Purcell) — University of Toronto study on knowledge acquisition time
- Fresh Futures (Natalie) — 9 users, 100% task success, low-tech focus
- Garden Buddy (UX Academy) — survey + 2 personas (Sarah/Betty)
- Planta design critique (Pratt IXD) — progressive disclosure, onboarding feedback
- GrowIt — empathetic onboarding ("Don't worry!" messages)
- Saku, Aloe, HOTA case studies — beginner-focused flows
- Grounded (Adelia Designs) — 84.9% have <3 hrs/week for gardening