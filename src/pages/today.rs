use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::ipc;
use garden_core::llm::Task;
use garden_core::models::{Harvest, Problem, Settings};
use garden_core::seasonal::{MONTH_NAMES, season_for_month0, season_name};

#[derive(serde::Serialize)]
struct CalendarArgs { month: i32, hemisphere: garden_core::models::Hemisphere }

#[component]
pub fn TodayPage() -> impl IntoView {
    let settings = RwSignal::new(None::<Settings>);
    let tasks = RwSignal::new(Vec::<Task>::new());
    let problems = RwSignal::new(Vec::<Problem>::new());
    let harvests = RwSignal::new(Vec::<Harvest>::new());
    let sow_now = RwSignal::new(Vec::<garden_core::models::CalendarEntry>::new());

    // load everything
    spawn_local(async move {
        if let Ok(s) = ipc::call0::<Settings>("get_settings").await {
            let hemi = s.hemisphere;
            let now = js_sys::Date::new_0();
            let month0 = now.get_month() as i32;
            let month = month0 + 1;
            settings.set(Some(s));

            // tasks
            if let Ok(t) = ipc::call0::<Vec<Task>>("list_tasks").await {
                tasks.set(t);
            }
            // problems
            if let Ok(p) = ipc::call0::<Vec<Problem>>("list_problems").await {
                problems.set(p);
            }
            // harvests
            if let Ok(h) = ipc::call0::<Vec<Harvest>>("list_harvests").await {
                harvests.set(h);
            }
            // sow now
            let args = CalendarArgs { month, hemisphere: hemi };
            if let Ok(entries) = ipc::call::<Vec<garden_core::models::CalendarEntry>, _>("get_calendar", &args).await {
                let eligible: Vec<_> = entries.into_iter().filter(|e| e.eligible).take(6).collect();
                sow_now.set(eligible);
            }
        }
    });

    let greeting = move || {
        let hour = js_sys::Date::new_0().get_hours();
        if hour < 12 { "Good morning" } else if hour < 18 { "Good afternoon" } else { "Good evening" }
    };

    let season_label = move || {
        let now = js_sys::Date::new_0();
        let month0 = now.get_month() as i32;
        let hemi = settings.get().map(|s| s.hemisphere).unwrap_or_default();
        let adjusted = if hemi == garden_core::models::Hemisphere::South { (month0 + 6) % 12 } else { month0 };
        let season = season_for_month0(adjusted);
        season_name(season)
    };

    view! {
        <div class="p-6 space-y-6">
            <header class="flex items-end justify-between">
                <div>
                    <p class="text-sm text-stone-400">{move || format!("{}", MONTH_NAMES[js_sys::Date::new_0().get_month() as usize])}</p>
                    <h1 class="text-2xl font-bold text-stone-100">{move || format!("{} — {}", greeting(), season_label())}</h1>
                    <p class="text-sm text-stone-500">"Here's what your garden needs today."</p>
                </div>
            </header>

            // Sow now
            <section class="card p-4">
                <h2 class="mb-3 text-xs font-black uppercase tracking-widest text-stone-500">"Sow now"</h2>
                { move || {
                    let items = sow_now.get();
                    if items.is_empty() {
                        view! { <p class="text-sm text-stone-500">"No sowing recommendations for this month yet."</p> }.into_any()
                    } else {
                        view! {
                            <div class="grid grid-cols-2 gap-3 sm:grid-cols-3">
                                <For each=move || sow_now.get() key=|e| e.plant.id.clone() let:e>
                                    <div class="rounded-xl border border-stone-800 bg-stone-900 p-3">
                                        <p class="text-sm font-bold text-stone-100">{e.plant.name.clone()}</p>
                                        <p class="text-[10px] text-stone-500">{e.reason.clone()}</p>
                                    </div>
                                </For>
                            </div>
                        }.into_any()
                    }
                } }
            </section>

            // Needs attention
            <section class="card p-4">
                <h2 class="mb-3 text-xs font-black uppercase tracking-widest text-stone-500">"Needs attention"</h2>
                <div class="space-y-2">
                    <For each=move || { tasks.get().into_iter().filter(|t| t.status == garden_core::llm::TaskStatus::Open).collect::<Vec<_>>() } key=|t| t.id.clone() let:t>
                        <div class="flex items-center justify-between rounded-lg border border-stone-800 bg-stone-900 px-3 py-2">
                            <span class="text-sm text-stone-200">{t.title.clone()}</span>
                            <span class="text-[10px] text-stone-500">"task"</span>
                        </div>
                    </For>
                    <For each=move || { problems.get().into_iter().filter(|p| !p.resolved).collect::<Vec<_>>() } key=|p| p.id.clone() let:p>
                        <div class="flex items-center justify-between rounded-lg border border-stone-800 bg-stone-900 px-3 py-2">
                            <span class="text-sm text-stone-200">{format!("{} — {}", p.planting_name.clone().unwrap_or_default(), p.cause)}</span>
                            <span class="text-[10px] text-amber-400">"problem"</span>
                        </div>
                    </For>
                </div>
                { move || {
                    let empty_tasks = tasks.get().iter().filter(|t| t.status == garden_core::llm::TaskStatus::Open).count() == 0;
                    let empty_probs = problems.get().iter().filter(|p| !p.resolved).count() == 0;
                    if empty_tasks && empty_probs {
                        view! { <p class="mt-2 text-sm text-stone-500">"Nothing needs attention right now. Nice work."</p> }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }
                } }
            </section>

            // Ready to harvest
            <section class="card p-4">
                <h2 class="mb-3 text-xs font-black uppercase tracking-widest text-stone-500">"Ready to harvest"</h2>
                { move || {
                    let hs = harvests.get();
                    if hs.is_empty() {
                        view! { <p class="text-sm text-stone-500">"No recent harvests logged."</p> }.into_any()
                    } else {
                        view! {
                            <div class="space-y-2">
                                <For each=move || harvests.get() key=|h| h.id.clone() let:h>
                                    <div class="flex items-center justify-between rounded-lg border border-stone-800 bg-stone-900 px-3 py-2">
                                        <span class="text-sm text-stone-200">{h.plant_name.clone().unwrap_or_default()}</span>
                                        <span class="text-[10px] text-stone-500">{h.date.clone()}</span>
                                    </div>
                                </For>
                            </div>
                        }.into_any()
                    }
                } }
            </section>
        </div>
    }
}
