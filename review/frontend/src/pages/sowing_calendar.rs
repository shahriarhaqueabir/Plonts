use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::ipc;
use crate::components::plant_image::PlantThumbnail;
use garden_core::models::{CalendarEntry, Hemisphere, Settings};

#[derive(serde::Serialize)]
struct CalendarArgs { month: i32, hemisphere: Hemisphere }

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun",
    "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

#[component]
pub fn SowingCalendarPage() -> impl IntoView {
    let month = RwSignal::new(0_i32);
    let hemisphere = RwSignal::new(Hemisphere::North);
    let entries = RwSignal::new(Vec::<CalendarEntry>::new());

    let reload = move || {
        let m = month.get();
        let h = hemisphere.get();
        spawn_local(async move {
            let args = CalendarArgs { month: m, hemisphere: h };
            if let Ok(e) = ipc::call::<Vec<CalendarEntry>, _>("get_calendar", &args).await {
                entries.set(e);
            }
        });
    };

    spawn_local(async move {
        if let Ok(s) = ipc::call0::<Settings>("get_settings").await {
            hemisphere.set(s.hemisphere);
        }
        reload();
    });

    let eligible = move || entries.get().into_iter().filter(|e| e.eligible).collect::<Vec<_>>();
    let ineligible = move || entries.get().into_iter().filter(|e| !e.eligible).collect::<Vec<_>>();

    view! {
        <div class="p-6">
            <h1 class="mb-1 text-xl font-bold text-stone-100">"Sowing Calendar"</h1>
            <p class="mb-6 text-sm text-stone-400">"What to plant now for your hemisphere."</p>

            <div class="card mb-6 p-4">
                <label class="mb-2 block text-xs font-bold uppercase tracking-widest text-stone-400">
                    { move || format!("Month: {} (hemisphere: {})", MONTHS[month.get() as usize], if hemisphere.get() == Hemisphere::North { "North" } else { "South" }) }
                </label>
                <input
                    type="range" min="0" max="11"
                    prop:value=move || month.get()
                    on:input=move |ev| { month.set(event_target_value(&ev).parse().unwrap_or(0)); reload(); }
                    class="w-full accent-garden-500"
                />
                <div class="mt-2 flex justify-between text-[10px] text-stone-600">
                    <For each=move || MONTHS.to_vec() key=|m| m.to_string() let:m>
                        <span>{m}</span>
                    </For>
                </div>
            </div>

            <div class="grid grid-cols-1 gap-6 md:grid-cols-2">
                <section class="card p-4">
                    <h2 class="mb-3 text-sm font-bold text-garden-400">
                        { move || format!("Plant now ({})", eligible().len()) }
                    </h2>
                    <div class="space-y-2">
                        <For each=eligible key=|e| e.plant.id.clone() let:e>
                            <div class="flex items-center gap-2 rounded-lg border border-garden-500/20 bg-garden-900/10 p-3">
                                <PlantThumbnail catalog_id=e.plant.id.clone() size_px=24 />
                                <div class="min-w-0">
                                    <div class="text-sm font-bold text-stone-100">{e.plant.name.clone()}</div>
                                    <div class="text-xs italic text-stone-500">{e.plant.scientific_name.clone().unwrap_or_default()}</div>
                                </div>
                            </div>
                        </For>
                    </div>
                </section>

                <section class="card p-4">
                    <h2 class="mb-3 text-sm font-bold text-amber-400">
                        { move || format!("Out of window ({})", ineligible().len()) }
                    </h2>
                    <div class="space-y-2">
                        <For each=ineligible key=|e| e.plant.id.clone() let:e>
                            <div class="flex items-center gap-2 rounded-lg border border-amber-500/10 bg-amber-900/5 p-3">
                                <PlantThumbnail catalog_id=e.plant.id.clone() size_px=24 />
                                <div class="min-w-0">
                                    <div class="text-sm font-bold text-stone-100">{e.plant.name.clone()}</div>
                                    <div class="text-xs text-stone-500">{e.reason.clone()}</div>
                                </div>
                            </div>
                        </For>
                    </div>
                </section>
            </div>
        </div>
    }
}
