use leptos::prelude::*;
use leptos::task::spawn_local;
use js_sys::Date;
use crate::components::toast::show_toast;

use crate::ipc;
use crate::components::plant_image::{GenericLeaf, PlantThumbnail};
use garden_core::models::{Bed, Garden, Planting, SearchResult};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateGardenArgs { name: String, garden_type: String }

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateBedArgs {
    garden_id: String,
    name: String,
    soil_type: Option<String>,
    sun_exposure: Option<String>,
    width_cm: Option<i64>,
    height_cm: Option<i64>,
    is_container: bool,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ListBedsArgs { garden_id: String }

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct AddPlantingArgs {
    bed_id: String,
    catalog_id: String,
    quantity: i64,
    source: String,
    notes: Option<String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchArgs { query: String, filters: Option<garden_core::models::SearchFilters>, limit: Option<usize> }

/// Compute days between two date strings (SQLite "YYYY-MM-DD HH:MM:SS" or "YYYY-MM-DD").
fn days_between(from: &str, to: &str) -> i64 {
    let parse = |s: &str| -> f64 {
        let d = Date::new(&wasm_bindgen::JsValue::from_str(s));
        d.get_time()
    };
    let ms = parse(to) - parse(from);
    (ms / 86_400_000.0).round() as i64
}

/// Growth stage for a planting given days elapsed since planting.
fn growth_stage(days: i64) -> (&'static str, &'static str, f64) {
    // (emoji, label, progress 0..1)
    if days < 0 {
        ("🌱", "Not yet planted", 0.0)
    } else if days < 7 {
        ("🌱", "Seed", 0.1)
    } else if days < 21 {
        ("🌿", "Sprout", 0.25)
    } else if days < 45 {
        ("🪴", "Seedling", 0.45)
    } else if days < 75 {
        ("🌾", "Growing", 0.7)
    } else if days < 100 {
        ("🌸", "Flowering", 0.85)
    } else {
        ("🍅", "Harvest ready", 1.0)
    }
}

#[component]
pub fn VirtualGardenPage() -> impl IntoView {
    let gardens = RwSignal::new(Vec::<Garden>::new());
    let toast = use_context::<RwSignal<Option<String>>>().unwrap_or_else(|| RwSignal::new(None));
    let selected_garden = RwSignal::new(None::<String>);
    let beds = RwSignal::new(Vec::<Bed>::new());
    let plantings = RwSignal::new(Vec::<Planting>::new());
    let plants = RwSignal::new(Vec::<SearchResult>::new());

    // Time simulation: offset in days from today (-90 to +180).
    let time_offset = RwSignal::new(0_i64);

    let selected_bed = RwSignal::new(None::<String>);
    let selected_plant = RwSignal::new(None::<String>);
    let quantity = RwSignal::new(1_i64);
    let source = RwSignal::new("seed".to_string());
    let notes = RwSignal::new(String::new());

    let reload_gardens = move || {
        spawn_local(async move {
            if let Ok(gs) = ipc::call0::<Vec<Garden>>("list_gardens").await {
                gardens.set(gs.clone());
                if let Some(first) = gs.first() {
                    if selected_garden.get().is_none() {
                        selected_garden.set(Some(first.id.clone()));
                    }
                }
            }
        });
    };
    let reload_beds = move || {
        let gid = selected_garden.get();
        spawn_local(async move {
            let Some(gid) = gid else { beds.set(vec![]); return; };
            let args = ListBedsArgs { garden_id: gid };
            if let Ok(bs) = ipc::call::<Vec<Bed>, _>("list_beds", &args).await {
                beds.set(bs);
            }
        });
    };
    let reload_plantings = move || {
        spawn_local(async move {
            if let Ok(ps) = ipc::call0::<Vec<Planting>>("list_plantings").await {
                plantings.set(ps);
            }
        });
    };

    // bootstrap: ensure a default garden/bed exists
    spawn_local(async move {
        match ipc::call0::<Vec<Garden>>("list_gardens").await {
            Ok(gs) if gs.is_empty() => {
                let g = ipc::call::<Garden, _>("create_garden", &CreateGardenArgs {
                    name: "My Garden".into(),
                    garden_type: "raised_bed".into(),
                }).await;
                match &g {
                    Err(e) => show_toast(toast, format!("Failed to create default garden: {e}")),
                    _ => {}
                }
                if let Some(g) = g.ok() {
                    let _ = ipc::call::<Bed, _>("create_bed", &CreateBedArgs {
                        garden_id: g.id,
                        name: "Main bed".into(),
                        soil_type: None,
                        sun_exposure: None,
                        width_cm: None,
                        height_cm: None,
                        is_container: false,
                    }).await;
                }
                reload_gardens();
                reload_beds();
            }
            _ => {
                reload_gardens();
                reload_beds();
            }
        }
        reload_plantings();
        if let Ok(ps) = ipc::call::<Vec<SearchResult>, _>("search_plants", &SearchArgs {
            query: String::new(),
            filters: None,
            limit: Some(300),
        }).await {
            plants.set(ps);
        }
    });

    let add = move |_| {
        let Some(bed_id) = selected_bed.get() else { return; };
        let Some(catalog_id) = selected_plant.get() else { return; };
        let qty = quantity.get();
        let src = source.get();
        let n = notes.get();
        let n = if n.trim().is_empty() { None } else { Some(n) };
        spawn_local(async move {
            let args = AddPlantingArgs {
                bed_id,
                catalog_id,
                quantity: qty,
                source: src,
                notes: n,
            };
            let toast = use_context::<RwSignal<Option<String>>>().unwrap_or_else(|| RwSignal::new(None));
            match ipc::call::<Planting, _>("add_planting", &args).await {
                Ok(_) => reload_plantings(),
                Err(e) => show_toast(toast, format!("Failed to add planting: {e}")),
            }
        });
    };

    let remove = move |id: String| {
        spawn_local(async move {
            let _ = ipc::call::<serde_json::Value, _>("remove_planting", &serde_json::json!({ "id": id })).await;
            reload_plantings();
        });
    };

    view! {
        <div class="p-6">
            <h1 class="mb-4 text-xl font-bold text-stone-100">"Virtual Garden"</h1>

            // Time simulation scrubber.
            <div class="mb-6 rounded-xl border border-stone-800 bg-stone-950 p-4">
                <div class="mb-2 flex items-center justify-between text-xs">
                    <span class="font-bold text-stone-300">"Time view"</span>
                    <span class="text-stone-500">
                        {move || {
                            let off = time_offset.get();
                            if off == 0 { "Today".to_string() }
                            else if off > 0 { format!("+{} days", off) }
                            else { format!("{} days", off) }
                        }}
                    </span>
                </div>
                <input
                    type="range" min="-90" max="180" step="1"
                    class="w-full accent-garden-500"
                    prop:value=move || time_offset.get()
                    on:input=move |ev| {
                        time_offset.set(event_target_value(&ev).parse().unwrap_or(0));
                    }
                />
                <div class="mt-1 flex justify-between text-[10px] text-stone-600">
                    <span>"-3 months"</span>
                    <button class="text-garden-400 hover:text-garden-300" on:click=move |_| time_offset.set(0)>"Today"</button>
                    <span>"+6 months"</span>
                </div>
            </div>

            <div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
                <section class="card p-4">
                    <h2 class="mb-3 text-xs font-black uppercase tracking-widest text-stone-500">"Add planting"</h2>
                    <select class="input mb-3" on:change=move |ev| { selected_garden.set(Some(event_target_value(&ev))); reload_beds(); }>
                        <For each=move || gardens.get() key=|g| g.id.clone() let:g>
                            <option value=g.id.clone() selected=move || selected_garden.get().as_deref() == Some(g.id.as_str())>
                                {g.name.clone()}
                            </option>
                        </For>
                    </select>
                    <select class="input mb-3" on:change=move |ev| selected_bed.set(Some(event_target_value(&ev)))>
                        <option value="">"Select bed"</option>
                        <For each=move || beds.get() key=|b| b.id.clone() let:b>
                            <option value=b.id.clone()>{b.name.clone()}</option>
                        </For>
                    </select>
                    <select class="input mb-3" on:change=move |ev| selected_plant.set(Some(event_target_value(&ev)))>
                        <option value="">"Select plant"</option>
                        <For each=move || plants.get() key=|p| p.plant.id.clone() let:p>
                            <option value=p.plant.id.clone()>{p.plant.name.clone()}</option>
                        </For>
                    </select>
                    <div class="mb-3 flex gap-3">
                        <input type="number" min="1" class="input w-24" prop:value=move || quantity.get()
                            on:input=move |ev| quantity.set(event_target_value(&ev).parse().unwrap_or(1)) />
                        <select class="input" on:change=move |ev| source.set(event_target_value(&ev))>
                            <option value="seed">"Seed"</option>
                            <option value="transplant">"Transplant"</option>
                            <option value="cutting">"Cutting"</option>
                        </select>
                    </div>
                    <input type="text" class="input mb-3" placeholder="Notes (optional)"
                        prop:value=move || notes.get()
                        on:input=move |ev| notes.set(event_target_value(&ev)) />
                    <button class="btn-primary w-full" on:click=add>"Add to garden"</button>
                </section>

                <section class="card p-4">
                    <h2 class="mb-3 text-xs font-black uppercase tracking-widest text-stone-500">"Inventory"</h2>
                    <For each=move || plantings.get() key=|p| p.id.clone() let:p>
                        <div class="mb-2 flex items-center gap-3 rounded-lg border border-stone-800 bg-stone-950 p-3">
                            <PlantThumbnail catalog_id=p.catalog_id.clone() size_px=32 />
                            <div class="min-w-0 flex-1">
                                <div class="flex items-center gap-2">
                                    <div class="text-sm font-bold text-stone-200">{p.plant_name.clone()}</div>
                                </div>
                                <div class="text-[11px] text-stone-500">
                                    {p.bed_name.clone()} " · " {p.quantity} "x " {p.source.clone()}
                                </div>
                                <div class="mt-1 text-[10px] text-garden-400">
                                    {move || {
                                        let off = time_offset.get();
                                        let view_date = {
                                            let d = Date::new_0();
                                            d.set_date((d.get_date() as i32 + off as i32) as u32);
                                            format!("{:04}-{:02}-{:02}", d.get_full_year(), d.get_month() + 1, d.get_date())
                                        };
                                        let pa = p.planted_at.clone();
                                        let days = days_between(&pa[..10], &view_date);
                                        let (emoji, label, progress) = growth_stage(days);
                                        format!("{} {} · Day {} · {:.0}%", emoji, label, days.max(0), progress * 100.0)
                                    }}
                                </div>
                            </div>
                            <button class="btn-ghost text-[10px]" on:click=move |_| remove(p.id.clone())>"Remove"</button>
                        </div>
                    </For>
                    { move || if plantings.get().is_empty() {
                        view! {
                            <div class="flex flex-col items-center gap-3 py-8 text-center">
                                <GenericLeaf size_px=96 />
                                <p class="text-sm text-stone-500">"Nothing planted yet. Add your first planting on the left."</p>
                            </div>
                        }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    } }
                </section>
            </div>
        </div>
    }
}