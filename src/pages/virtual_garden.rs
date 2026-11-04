use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::ipc;
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
struct UpdateStatusArgs { id: String, status: String, removal_reason: Option<String> }

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchArgs { query: String, filters: Option<garden_core::models::SearchFilters>, limit: Option<usize> }

#[component]
pub fn VirtualGardenPage() -> impl IntoView {
    let gardens = RwSignal::new(Vec::<Garden>::new());
    let selected_garden = RwSignal::new(None::<String>);
    let beds = RwSignal::new(Vec::<Bed>::new());
    let plantings = RwSignal::new(Vec::<Planting>::new());
    let plants = RwSignal::new(Vec::<SearchResult>::new());

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
                }).await.ok();
                if let Some(g) = g {
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
            let _ = ipc::call::<Planting, _>("add_planting", &args).await;
            reload_plantings();
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
            <h1 class="mb-6 text-xl font-bold text-stone-100">"Virtual Garden"</h1>

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
                        <div class="mb-2 flex items-center justify-between rounded-lg border border-stone-800 bg-stone-950 p-3">
                            <div>
                                <div class="text-sm font-bold text-stone-200">{p.plant_name.clone()}</div>
                                <div class="text-[11px] text-stone-500">
                                    {p.bed_name.clone()} " - " {p.quantity} " x " {p.source.clone()} " - " {p.status.clone()}
                                </div>
                            </div>
                            <button class="btn-ghost text-[10px]" on:click=move |_| remove(p.id.clone())>"Remove"</button>
                        </div>
                    </For>
                    { move || if plantings.get().is_empty() {
                        view! { <p class="text-sm text-stone-500">"Nothing planted yet."</p> }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    } }
                </section>
            </div>
        </div>
    }
}