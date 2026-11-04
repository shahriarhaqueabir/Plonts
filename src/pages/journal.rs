use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::ipc;
use garden_core::models::{
    Bed, Garden, JournalEntry, JournalFilters, Measurement, MeasurementKind, PlantSummary,
    PlantTimeline, SearchResult,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode { Entries, Timeline }

#[derive(serde::Serialize)]
struct ListEntriesArgs { filters: Option<JournalFilters> }

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateEntryArgs { plant_id: Option<String>, bed_id: Option<String>, text: String, tags: Vec<String> }

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateEntryArgs { id: String, plant_id: Option<String>, bed_id: Option<String>, text: String, tags: Vec<String> }

#[derive(serde::Serialize)]
struct IdArgs { id: String }

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct AddPhotoArgs { entry_id: String, source_path: String }

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct PhotoIdArgs { photo_id: String }

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportPhotoArgs { photo_id: String, dest_path: String }

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct AddMeasurementArgs { plant_id: String, kind: MeasurementKind, value: f64, unit: String }

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct PlantIdArgs { plant_id: String }

#[derive(serde::Serialize)]
struct SearchArgs { query: String, filters: Option<garden_core::models::SearchFilters>, limit: Option<usize> }

fn parse_tags(s: &str) -> Vec<String> {
    s.split(',').map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect()
}

#[component]
pub fn JournalPage() -> impl IntoView {
    let mode = RwSignal::new(Mode::Entries);
    let entries = RwSignal::new(Vec::<JournalEntry>::new());
    let query = RwSignal::new(String::new());
    let plant_filter = RwSignal::new(None::<String>);
    let plants = RwSignal::new(Vec::<PlantSummary>::new());
    let beds = RwSignal::new(Vec::<Bed>::new());

    let new_text = RwSignal::new(String::new());
    let new_plant = RwSignal::new(None::<String>);
    let new_bed = RwSignal::new(None::<String>);
    let new_tags = RwSignal::new(String::new());

    let editing_id = RwSignal::new(None::<String>);
    let edit_text = RwSignal::new(String::new());
    let edit_plant = RwSignal::new(None::<String>);
    let edit_bed = RwSignal::new(None::<String>);
    let edit_tags = RwSignal::new(String::new());

    let tl_plant = RwSignal::new(None::<String>);
    let timeline = RwSignal::new(None::<PlantTimeline>);
    let m_kind = RwSignal::new(MeasurementKind::Height);
    let m_value = RwSignal::new(String::new());
    let m_unit = RwSignal::new("cm".to_string());

    let reload_entries = move || {
        let filters = JournalFilters {
            query: if query.get().trim().is_empty() { None } else { Some(query.get()) },
            plant_id: plant_filter.get(),
            tag: None,
        };
        spawn_local(async move {
            let args = ListEntriesArgs { filters: Some(filters) };
            if let Ok(e) = ipc::call::<Vec<JournalEntry>, _>("list_journal_entries", &args).await {
                entries.set(e);
            }
        });
    };

    spawn_local(async move {
        // plant options
        let args = SearchArgs { query: String::new(), filters: None, limit: Some(300) };
        if let Ok(ps) = ipc::call::<Vec<SearchResult>, _>("search_plants", &args).await {
            plants.set(ps.into_iter().map(|r| r.plant).collect());
        }
        // bed options (flatten across gardens)
        if let Ok(gs) = ipc::call0::<Vec<Garden>>("list_gardens").await {
            let mut all = Vec::new();
            for g in gs {
                let bargs = serde_json::json!({ "gardenId": g.id });
                if let Ok(bs) = ipc::call::<Vec<Bed>, _>("list_beds", &bargs).await {
                    all.extend(bs);
                }
            }
            beds.set(all);
        }
        reload_entries();
    });

    let create_entry = move |_| {
        let text = new_text.get();
        if text.trim().is_empty() { return; }
        let args = CreateEntryArgs {
            plant_id: new_plant.get(),
            bed_id: new_bed.get(),
            text,
            tags: parse_tags(&new_tags.get()),
        };
        spawn_local(async move {
            let _ = ipc::call::<JournalEntry, _>("create_journal_entry", &args).await;
            new_text.set(String::new());
            new_tags.set(String::new());
            new_plant.set(None);
            new_bed.set(None);
            reload_entries();
        });
    };

    let delete_entry = move |id: String| {
        spawn_local(async move {
            let _ = ipc::call::<serde_json::Value, _>("delete_journal_entry", &IdArgs { id }).await;
            reload_entries();
        });
    };

    let start_edit = move |e: JournalEntry| {
        editing_id.set(Some(e.id));
        edit_text.set(e.text);
        edit_plant.set(e.plant_id);
        edit_bed.set(e.bed_id);
        edit_tags.set(e.tags.join(", "));
    };

    let save_edit = move |_| {
        let Some(id) = editing_id.get() else { return; };
        let args = UpdateEntryArgs {
            id,
            plant_id: edit_plant.get(),
            bed_id: edit_bed.get(),
            text: edit_text.get(),
            tags: parse_tags(&edit_tags.get()),
        };
        spawn_local(async move {
            let _ = ipc::call::<JournalEntry, _>("update_journal_entry", &args).await;
            editing_id.set(None);
            reload_entries();
        });
    };

    let add_photo = move |entry_id: String| {
        spawn_local(async move {
            if let Ok(Some(path)) = ipc::pick_file().await {
                let args = AddPhotoArgs { entry_id: entry_id.clone(), source_path: path };
                let _ = ipc::call::<serde_json::Value, _>("add_journal_photo", &args).await;
                reload_entries();
            }
        });
    };

    let delete_photo = move |photo_id: String| {
        spawn_local(async move {
            let _ = ipc::call::<serde_json::Value, _>("delete_journal_photo", &PhotoIdArgs { photo_id }).await;
            reload_entries();
        });
    };

    let export_photo = move |photo_id: String| {
        spawn_local(async move {
            if let Ok(Some(dest)) = ipc::pick_save_path().await {
                let args = ExportPhotoArgs { photo_id: photo_id.clone(), dest_path: dest };
                let _ = ipc::call::<serde_json::Value, _>("export_journal_photo", &args).await;
            }
        });
    };

    let load_timeline = move |plant_id: String| {
        spawn_local(async move {
            let args = PlantIdArgs { plant_id: plant_id.clone() };
            if let Ok(t) = ipc::call::<PlantTimeline, _>("get_plant_timeline", &args).await {
                tl_plant.set(Some(plant_id));
                timeline.set(Some(t));
            }
        });
    };

    let add_measurement = move |_| {
        let Some(plant_id) = tl_plant.get() else { return; };
        let Ok(value) = m_value.get().parse::<f64>() else { return; };
        let args = AddMeasurementArgs {
            plant_id: plant_id.clone(),
            kind: m_kind.get(),
            value,
            unit: m_unit.get(),
        };
        spawn_local(async move {
            let _ = ipc::call::<Measurement, _>("add_measurement", &args).await;
            m_value.set(String::new());
            load_timeline(plant_id);
        });
    };

    fn opt_val(ev: &leptos::ev::Event) -> Option<String> {
        let v = event_target_value(ev);
        if v.is_empty() { None } else { Some(v) }
    }

    fn kind_label(k: MeasurementKind) -> &'static str {
        match k {
            MeasurementKind::Height => "Height",
            MeasurementKind::Yield => "Yield",
            MeasurementKind::Flowers => "Flowers",
        }
    }

    view! {
        <div class="p-6">
            <h1 class="mb-1 text-xl font-bold text-stone-100">"Journal"</h1>
            <p class="mb-4 text-sm text-stone-400">"Notes, photos and measurements for your garden."</p>

            <div class="mb-4 flex gap-3">
                <button class=move || if mode.get() == Mode::Entries { "btn-primary" } else { "btn-ghost" }
                    on:click=move |_| mode.set(Mode::Entries)>"Entries"</button>
                <button class=move || if mode.get() == Mode::Timeline { "btn-primary" } else { "btn-ghost" }
                    on:click=move |_| mode.set(Mode::Timeline)>"Timeline"</button>
            </div>

            { move || if mode.get() == Mode::Entries {
                view! {
                    <div>
                        <div class="mb-3 flex gap-3">
                            <input type="text" class="input flex-1" placeholder="Search entries..."
                                prop:value=move || query.get()
                                on:input=move |ev| { query.set(event_target_value(&ev)); reload_entries(); } />
                            <select class="input w-48" on:change=move |ev| {
                                let v = event_target_value(&ev);
                                plant_filter.set(if v.is_empty() { None } else { Some(v) });
                                reload_entries();
                            }>
                                <option value="">"All plants"</option>
                                <For each=move || plants.get() key=|p| p.id.clone() let:p>
                                    <option value=p.id.clone()>{p.name.clone()}</option>
                                </For>
                            </select>
                        </div>

                        { move || if editing_id.get().is_some() {
                            view! {
                                <div class="card mb-4 p-4">
                                    <h2 class="mb-3 text-xs font-black uppercase tracking-widest text-stone-500">"Edit entry"</h2>
                                    <textarea class="input mb-3" rows="3"
                                        prop:value=move || edit_text.get()
                                        on:input=move |ev| edit_text.set(event_target_value(&ev)) />
                                    <div class="mb-3 flex gap-3">
                                        <select class="input" on:change=move |ev| edit_plant.set(opt_val(&ev))>
                                            <option value="">"No plant"</option>
                                            <For each=move || plants.get() key=|p| p.id.clone() let:p>
                                                <option value=p.id.clone() selected=move || edit_plant.get().as_deref() == Some(p.id.as_str())>{p.name.clone()}</option>
                                            </For>
                                        </select>
                                        <select class="input" on:change=move |ev| edit_bed.set(opt_val(&ev))>
                                            <option value="">"No bed"</option>
                                            <For each=move || beds.get() key=|b| b.id.clone() let:b>
                                                <option value=b.id.clone() selected=move || edit_bed.get().as_deref() == Some(b.id.as_str())>{b.name.clone()}</option>
                                            </For>
                                        </select>
                                    </div>
                                    <input type="text" class="input mb-3" placeholder="Tags (comma separated)"
                                        prop:value=move || edit_tags.get()
                                        on:input=move |ev| edit_tags.set(event_target_value(&ev)) />
                                    <div class="flex gap-2">
                                        <button class="btn-primary" on:click=save_edit>"Save"</button>
                                        <button class="btn-ghost" on:click=move |_| editing_id.set(None)>"Cancel"</button>
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="card mb-4 p-4">
                                    <h2 class="mb-3 text-xs font-black uppercase tracking-widest text-stone-500">"New entry"</h2>
                                    <textarea class="input mb-3" rows="3" placeholder="What happened?"
                                        prop:value=move || new_text.get()
                                        on:input=move |ev| new_text.set(event_target_value(&ev)) />
                                    <div class="mb-3 flex gap-3">
                                        <select class="input" on:change=move |ev| new_plant.set(opt_val(&ev))>
                                            <option value="">"No plant"</option>
                                            <For each=move || plants.get() key=|p| p.id.clone() let:p>
                                                <option value=p.id.clone()>{p.name.clone()}</option>
                                            </For>
                                        </select>
                                        <select class="input" on:change=move |ev| new_bed.set(opt_val(&ev))>
                                            <option value="">"No bed"</option>
                                            <For each=move || beds.get() key=|b| b.id.clone() let:b>
                                                <option value=b.id.clone()>{b.name.clone()}</option>
                                            </For>
                                        </select>
                                    </div>
                                    <input type="text" class="input mb-3" placeholder="Tags (comma separated)"
                                        prop:value=move || new_tags.get()
                                        on:input=move |ev| new_tags.set(event_target_value(&ev)) />
                                    <button class="btn-primary" on:click=create_entry>"Add entry"</button>
                                </div>
                            }.into_any()
                        } }

                        <div class="space-y-3">
                            <For each=move || entries.get() key=|e| e.id.clone() let:e>
                                {
                                    let e_edit = e.clone();
                                    let e_delete = e.clone();
                                    let e_tags = e.clone();
                                    let e_photos = e.clone();
                                    let e_add = e.clone();
                                    let e_text = e.text.clone();
                                    let e_plant_name = e.plant_name.clone();
                                    let e_bed_name = e.bed_name.clone();
                                    view! {
                                <div class="card p-4">
                                    <div class="flex items-start justify-between gap-3">
                                        <p class="text-sm text-stone-200">{e_text}</p>
                                        <div class="flex shrink-0 gap-2">
                                            <button class="btn-ghost text-[10px]" on:click=move |_| start_edit(e_edit.clone())>"Edit"</button>
                                            <button class="btn-ghost text-[10px]" on:click=move |_| delete_entry(e_delete.id.clone())>"Delete"</button>
                                        </div>
                                    </div>
                                    <div class="mt-2 flex flex-wrap gap-1">
                                        { if e_plant_name.is_some() { view! { <span class="rounded border border-garden-500/20 bg-garden-900/10 px-1.5 py-0.5 text-[10px] text-garden-300">{e_plant_name.clone().unwrap_or_default()}</span> }.into_any() } else { view! { <span></span> }.into_any() } }
                                        { if e_bed_name.is_some() { view! { <span class="rounded border border-stone-800 bg-stone-900 px-1.5 py-0.5 text-[10px] text-stone-400">{e_bed_name.clone().unwrap_or_default()}</span> }.into_any() } else { view! { <span></span> }.into_any() } }
                                        <For each=move || e_tags.tags.clone() key=|t| t.clone() let:t>
                                            <span class="rounded border border-stone-800 bg-stone-900 px-1.5 py-0.5 text-[10px] text-stone-400">{"#"} {t}</span>
                                        </For>
                                    </div>
                                    <div class="mt-3 flex flex-wrap gap-2">
                                        <For each=move || e_photos.photos.clone() key=|p| p.id.clone() let:p>
                                            {
                                                let p_export = p.clone();
                                                let p_delete = p.clone();
                                                let p_url = p.url.clone();
                                                view! {
                                            <div class="flex flex-col gap-1">
                                                <img src=p_url class="h-20 w-20 rounded-lg object-cover" />
                                                <div class="flex gap-1">
                                                    <button class="btn-ghost text-[9px]" on:click=move |_| export_photo(p_export.id.clone())>"Export"</button>
                                                    <button class="btn-ghost text-[9px]" on:click=move |_| delete_photo(p_delete.id.clone())>"x"</button>
                                                </div>
                                            </div>
                                                }
                                            }
                                        </For>
                                        <button class="btn-ghost text-[10px]" on:click=move |_| add_photo(e_add.id.clone())>"Add photo"</button>
                                    </div>
                                </div>
                                    }
                                }
                            </For>
                            { move || if entries.get().is_empty() {
                                view! { <p class="text-sm text-stone-500">"No entries yet. Add your first note above."</p> }.into_any()
                            } else {
                                view! { <span></span> }.into_any()
                            } }
                        </div>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
                        <section class="card p-4">
                            <h2 class="mb-3 text-xs font-black uppercase tracking-widest text-stone-500">"Plant timeline"</h2>
                            <select class="input mb-3" on:change=move |ev| {
                                let v = event_target_value(&ev);
                                if v.is_empty() {
                                    tl_plant.set(None);
                                    timeline.set(None);
                                } else {
                                    load_timeline(v);
                                }
                            }>
                                <option value="">"Choose a plant"</option>
                                <For each=move || plants.get() key=|p| p.id.clone() let:p>
                                    <option value=p.id.clone()>{p.name.clone()}</option>
                                </For>
                            </select>

                            { move || timeline.get().map(|t| {
                                let plant_name = t.plant_name.clone();
                                let measurements = t.measurements.clone();
                                let entries = t.entries.clone();
                                let meas_empty = measurements.is_empty();
                                let entries_empty = entries.is_empty();
                                view! {
                                <div>
                                    <h3 class="mb-2 font-bold text-stone-100">{plant_name}</h3>
                                    <h4 class="mb-1 text-xs font-bold text-stone-500">"Measurements"</h4>
                                    <div class="mb-3 space-y-1">
                                        <For each=move || measurements.clone() key=|m| m.id.clone() let:m>
                                            <div class="flex items-center justify-between text-sm text-stone-300">
                                                <span>{format!("{}: {} {}", kind_label(m.kind), m.value, m.unit)}</span>
                                                <span class="text-[10px] text-stone-600">{m.recorded_at.clone()}</span>
                                            </div>
                                        </For>
                                        { if meas_empty {
                                            view! { <p class="text-xs text-stone-600">"No measurements yet."</p> }.into_any()
                                        } else {
                                            view! { <span></span> }.into_any()
                                        } }
                                    </div>
                                    <h4 class="mb-1 text-xs font-bold text-stone-500">"Entries"</h4>
                                    <div class="space-y-1">
                                        <For each=move || entries.clone() key=|e| e.id.clone() let:e>
                                            <div class="text-sm text-stone-300">
                                                <span class="text-[10px] text-stone-600">{e.created_at.clone()}</span> " " {e.text.clone()}
                                            </div>
                                        </For>
                                        { if entries_empty {
                                            view! { <p class="text-xs text-stone-600">"No entries yet."</p> }.into_any()
                                        } else {
                                            view! { <span></span> }.into_any()
                                        } }
                                    </div>
                                </div>
                            } }) }
                        </section>

                        <section class="card p-4">
                            <h2 class="mb-3 text-xs font-black uppercase tracking-widest text-stone-500">"Log measurement"</h2>
                            <select class="input mb-3" on:change=move |ev| m_kind.set(match event_target_value(&ev).as_str() {
                                "yield" => MeasurementKind::Yield,
                                "flowers" => MeasurementKind::Flowers,
                                _ => MeasurementKind::Height,
                            })>
                                <option value="height">"Height"</option>
                                <option value="yield">"Yield"</option>
                                <option value="flowers">"Flowers"</option>
                            </select>
                            <div class="mb-3 flex gap-3">
                                <input type="number" step="0.1" class="input" placeholder="Value"
                                    prop:value=move || m_value.get()
                                    on:input=move |ev| m_value.set(event_target_value(&ev)) />
                                <input type="text" class="input w-24" placeholder="unit"
                                    prop:value=move || m_unit.get()
                                    on:input=move |ev| m_unit.set(event_target_value(&ev)) />
                            </div>
                            <button class="btn-primary" on:click=add_measurement disabled=move || tl_plant.get().is_none()>"Save measurement"</button>
                            { move || if tl_plant.get().is_none() {
                                view! { <p class="mt-2 text-xs text-stone-600">"Choose a plant in the timeline panel first."</p> }.into_any()
                            } else {
                                view! { <span></span> }.into_any()
                            } }
                        </section>
                    </div>
                }.into_any()
            } }
        </div>
    }
}
