use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::ipc;
use garden_core::models::{
    Harvest, Planting, SearchResult, Seed, ShoppingItem, Tool, Viability, YieldRow,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode { Seeds, Harvests, Supplies }

#[derive(serde::Serialize)]
struct SearchArgs { query: String, filters: Option<garden_core::models::SearchFilters>, limit: Option<usize> }

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct AddSeedArgs { catalog_id: Option<String>, variety: String, source: String, year: i64, quantity: i64, germination_notes: Option<String> }

#[derive(serde::Serialize)]
struct IdArgs { id: String }

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct AddHarvestArgs { planting_id: Option<String>, catalog_id: Option<String>, date: String, weight_g: Option<f64>, count: Option<i64>, notes: Option<String> }

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct YieldArgs { group_by: String }

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct AddShoppingArgs { name: String, category: String, quantity: Option<String> }

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct AddToolArgs { name: String, next_maintenance: Option<String>, notes: Option<String> }

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct MaintainArgs { id: String, next_maintenance: Option<String> }

fn viability_label(v: Viability) -> &'static str {
    match v {
        Viability::Fresh => "Fresh",
        Viability::Aging => "Aging",
        Viability::Expired => "Expired",
    }
}

#[component]
pub fn SeedsHarvestPage() -> impl IntoView {
    let mode = RwSignal::new(Mode::Seeds);

    // seeds
    let seeds = RwSignal::new(Vec::<Seed>::new());
    let plants = RwSignal::new(Vec::<SearchResult>::new());
    let s_catalog = RwSignal::new(None::<String>);
    let s_variety = RwSignal::new(String::new());
    let s_source = RwSignal::new(String::new());
    let s_year = RwSignal::new(2026_i64);
    let s_qty = RwSignal::new(1_i64);
    let s_notes = RwSignal::new(String::new());

    // harvests
    let harvests = RwSignal::new(Vec::<Harvest>::new());
    let plantings = RwSignal::new(Vec::<Planting>::new());
    let h_planting = RwSignal::new(None::<String>);
    let h_catalog = RwSignal::new(None::<String>);
    let h_date = RwSignal::new(String::new());
    let h_weight = RwSignal::new(String::new());
    let h_count = RwSignal::new(String::new());
    let h_notes = RwSignal::new(String::new());
    let yield_group = RwSignal::new("plant".to_string());
    let yields = RwSignal::new(Vec::<YieldRow>::new());

    // supplies
    let shopping = RwSignal::new(Vec::<ShoppingItem>::new());
    let tools = RwSignal::new(Vec::<Tool>::new());
    let shop_name = RwSignal::new(String::new());
    let shop_category = RwSignal::new("seeds".to_string());
    let shop_qty = RwSignal::new(String::new());
    let tool_name = RwSignal::new(String::new());
    let tool_next = RwSignal::new(String::new());
    let tool_notes = RwSignal::new(String::new());
    let maintain_id = RwSignal::new(None::<String>);
    let maintain_next = RwSignal::new(String::new());

    let reload_seeds = move || { spawn_local(async move { if let Ok(s) = ipc::call0::<Vec<Seed>>("list_seeds").await { seeds.set(s); } }); };
    let reload_harvests = move || { spawn_local(async move { if let Ok(h) = ipc::call0::<Vec<Harvest>>("list_harvests").await { harvests.set(h); } }); };
    let reload_yield = move || {
        let g = yield_group.get();
        spawn_local(async move { if let Ok(y) = ipc::call::<Vec<YieldRow>, _>("yield_summary", &YieldArgs { group_by: g }).await { yields.set(y); } });
    };
    let reload_shopping = move || { spawn_local(async move { if let Ok(s) = ipc::call0::<Vec<ShoppingItem>>("list_shopping_items").await { shopping.set(s); } }); };
    let reload_tools = move || { spawn_local(async move { if let Ok(t) = ipc::call0::<Vec<Tool>>("list_tools").await { tools.set(t); } }); };

    spawn_local(async move {
        if let Ok(ps) = ipc::call::<Vec<SearchResult>, _>("search_plants", &SearchArgs { query: String::new(), filters: None, limit: Some(300) }).await {
            plants.set(ps);
        }
        if let Ok(pl) = ipc::call0::<Vec<Planting>>("list_plantings").await {
            plantings.set(pl);
        }
        reload_seeds();
        reload_harvests();
        reload_yield();
        reload_shopping();
        reload_tools();
    });

    let add_seed = move |_| {
        if s_variety.get().trim().is_empty() { return; }
        let args = AddSeedArgs {
            catalog_id: s_catalog.get(),
            variety: s_variety.get(),
            source: s_source.get(),
            year: s_year.get(),
            quantity: s_qty.get(),
            germination_notes: if s_notes.get().trim().is_empty() { None } else { Some(s_notes.get()) },
        };
        spawn_local(async move {
            let _ = ipc::call::<Seed, _>("add_seed", &args).await;
            s_variety.set(String::new());
            s_notes.set(String::new());
            reload_seeds();
        });
    };
    let delete_seed = move |id: String| { spawn_local(async move { let _ = ipc::call::<serde_json::Value, _>("delete_seed", &IdArgs { id }).await; reload_seeds(); }); };
    let ack_seed = move |id: String| { spawn_local(async move { let _ = ipc::call::<Seed, _>("acknowledge_seed_viability", &IdArgs { id }).await; reload_seeds(); }); };

    let add_harvest = move |_| {
        if h_date.get().trim().is_empty() { return; }
        let args = AddHarvestArgs {
            planting_id: h_planting.get(),
            catalog_id: h_catalog.get(),
            date: h_date.get(),
            weight_g: h_weight.get().trim().parse::<f64>().ok(),
            count: h_count.get().trim().parse::<i64>().ok(),
            notes: if h_notes.get().trim().is_empty() { None } else { Some(h_notes.get()) },
        };
        spawn_local(async move {
            let _ = ipc::call::<Harvest, _>("add_harvest", &args).await;
            h_date.set(String::new());
            h_weight.set(String::new());
            h_count.set(String::new());
            h_notes.set(String::new());
            reload_harvests();
            reload_yield();
        });
    };
    let delete_harvest = move |id: String| { spawn_local(async move { let _ = ipc::call::<serde_json::Value, _>("delete_harvest", &IdArgs { id }).await; reload_harvests(); reload_yield(); }); };

    let add_shopping = move |_| {
        if shop_name.get().trim().is_empty() { return; }
        let args = AddShoppingArgs { name: shop_name.get(), category: shop_category.get(), quantity: if shop_qty.get().trim().is_empty() { None } else { Some(shop_qty.get()) } };
        spawn_local(async move {
            let _ = ipc::call::<ShoppingItem, _>("add_shopping_item", &args).await;
            shop_name.set(String::new());
            shop_qty.set(String::new());
            reload_shopping();
        });
    };
    let toggle_shopping = move |id: String| { spawn_local(async move { let _ = ipc::call::<ShoppingItem, _>("toggle_shopping_item", &IdArgs { id }).await; reload_shopping(); }); };
    let delete_shopping = move |id: String| { spawn_local(async move { let _ = ipc::call::<serde_json::Value, _>("delete_shopping_item", &IdArgs { id }).await; reload_shopping(); }); };

    let add_tool = move |_| {
        if tool_name.get().trim().is_empty() { return; }
        let args = AddToolArgs { name: tool_name.get(), next_maintenance: if tool_next.get().trim().is_empty() { None } else { Some(tool_next.get()) }, notes: if tool_notes.get().trim().is_empty() { None } else { Some(tool_notes.get()) } };
        spawn_local(async move {
            let _ = ipc::call::<Tool, _>("add_tool", &args).await;
            tool_name.set(String::new());
            tool_next.set(String::new());
            tool_notes.set(String::new());
            reload_tools();
        });
    };
    let maintain_tool = move |id: String| {
        let next = maintain_next.get();
        let next = if next.trim().is_empty() { None } else { Some(next) };
        spawn_local(async move {
            let _ = ipc::call::<Tool, _>("mark_tool_maintained", &MaintainArgs { id, next_maintenance: next }).await;
            maintain_id.set(None);
            maintain_next.set(String::new());
            reload_tools();
        });
    };
    let delete_tool = move |id: String| { spawn_local(async move { let _ = ipc::call::<serde_json::Value, _>("delete_tool", &IdArgs { id }).await; reload_tools(); }); };

    view! {
        <div class="p-6">
            <h1 class="mb-1 text-xl font-bold text-stone-100">"Seeds & Harvest"</h1>
            <p class="mb-4 text-sm text-stone-400">"Track seeds, harvests, supplies and tools."</p>

            <div class="mb-4 flex gap-3">
                <button class=move || if mode.get() == Mode::Seeds { "btn-primary" } else { "btn-ghost" }
                    on:click=move |_| mode.set(Mode::Seeds)>"Seeds"</button>
                <button class=move || if mode.get() == Mode::Harvests { "btn-primary" } else { "btn-ghost" }
                    on:click=move |_| mode.set(Mode::Harvests)>"Harvests"</button>
                <button class=move || if mode.get() == Mode::Supplies { "btn-primary" } else { "btn-ghost" }
                    on:click=move |_| mode.set(Mode::Supplies)>"Supplies"</button>
            </div>

            { move || match mode.get() {
                Mode::Seeds => view! {
                    <div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
                        <section class="card p-4">
                            <h2 class="mb-3 text-xs font-black uppercase tracking-widest text-stone-500">"Add seeds"</h2>
                            <select class="input mb-3" on:change=move |ev| {
                                let v = event_target_value(&ev);
                                s_catalog.set(if v.is_empty() { None } else { Some(v) });
                            }>
                                <option value="">"No plant"</option>
                                <For each=move || plants.get() key=|p| p.plant.id.clone() let:p>
                                    <option value=p.plant.id.clone()>{p.plant.name.clone()}</option>
                                </For>
                            </select>
                            <input type="text" class="input mb-3" placeholder="Variety"
                                prop:value=move || s_variety.get()
                                on:input=move |ev| s_variety.set(event_target_value(&ev)) />
                            <input type="text" class="input mb-3" placeholder="Source (shop / saved)"
                                prop:value=move || s_source.get()
                                on:input=move |ev| s_source.set(event_target_value(&ev)) />
                            <div class="mb-3 flex gap-3">
                                <input type="number" min="1900" class="input w-28" placeholder="Year"
                                    prop:value=move || s_year.get()
                                    on:input=move |ev| s_year.set(event_target_value(&ev).parse().unwrap_or(2026)) />
                                <input type="number" min="0" class="input w-28" placeholder="Qty"
                                    prop:value=move || s_qty.get()
                                    on:input=move |ev| s_qty.set(event_target_value(&ev).parse().unwrap_or(0)) />
                            </div>
                            <input type="text" class="input mb-3" placeholder="Germination notes"
                                prop:value=move || s_notes.get()
                                on:input=move |ev| s_notes.set(event_target_value(&ev)) />
                            <button class="btn-primary w-full" on:click=add_seed>"Add seeds"</button>
                        </section>

                        <section class="card p-4">
                            <h2 class="mb-3 text-xs font-black uppercase tracking-widest text-stone-500">"Inventory"</h2>
                            <For each=move || seeds.get() key=|s| s.id.clone() let:s>
                                {
                                    let s_delete = s.id.clone();
                                    let s_ack = s.id.clone();
                                    let s_plant = s.plant_name.clone();
                                    let s_variety = s.variety.clone();
                                    let s_year = s.year;
                                    let s_qty = s.quantity;
                                    let s_source = s.source.clone();
                                    let s_notes = s.germination_notes.clone();
                                    let s_warned = s.viability_warned;
                                    let s_viability = s.viability;
                                    view! {
                                        <div class="mb-2 rounded-lg border border-stone-800 bg-stone-950 p-3">
                                            <div class="flex items-center justify-between gap-2">
                                                <div class="font-bold text-stone-200">{s_variety}</div>
                                                <span class="text-[10px] text-stone-500">{viability_label(s_viability)}</span>
                                            </div>
                                            <div class="text-[11px] text-stone-500">{s_year} " - " {s_qty} " pkts - " {s_source}</div>
                                            { if s_plant.is_some() { view! { <div class="text-[11px] text-stone-500">{s_plant.clone().unwrap_or_default()}</div> }.into_any() } else { view! { <span></span> }.into_any() } }
                                            { if s_notes.is_some() { view! { <div class="text-[11px] text-stone-600">{s_notes.clone().unwrap_or_default()}</div> }.into_any() } else { view! { <span></span> }.into_any() } }
                                            <div class="mt-2 flex gap-2">
                                                { if s_viability != Viability::Fresh && !s_warned {
                                                    view! { <button class="btn-ghost text-[10px]" on:click=move |_| ack_seed(s_ack.clone())>"Acknowledge warning"</button> }.into_any()
                                                } else {
                                                    view! { <span></span> }.into_any()
                                                } }
                                                <button class="btn-ghost text-[10px]" on:click=move |_| delete_seed(s_delete.clone())>"Delete"</button>
                                            </div>
                                        </div>
                                    }
                                }
                            </For>
                            { move || if seeds.get().is_empty() { view! { <p class="text-sm text-stone-500">"No seeds yet."</p> }.into_any() } else { view! { <span></span> }.into_any() } }
                        </section>
                    </div>
                }.into_any(),

                Mode::Harvests => view! {
                    <div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
                        <section class="card p-4">
                            <h2 class="mb-3 text-xs font-black uppercase tracking-widest text-stone-500">"Log harvest"</h2>
                            <select class="input mb-3" on:change=move |ev| {
                                let v = event_target_value(&ev);
                                h_planting.set(if v.is_empty() { None } else { Some(v) });
                            }>
                                <option value="">"No planting"</option>
                                <For each=move || plantings.get() key=|p| p.id.clone() let:p>
                                    <option value=p.id.clone()>{format!("{} - {}", p.plant_name, p.bed_name)}</option>
                                </For>
                            </select>
                            <select class="input mb-3" on:change=move |ev| {
                                let v = event_target_value(&ev);
                                h_catalog.set(if v.is_empty() { None } else { Some(v) });
                            }>
                                <option value="">"Plant (optional)"</option>
                                <For each=move || plants.get() key=|p| p.plant.id.clone() let:p>
                                    <option value=p.plant.id.clone()>{p.plant.name.clone()}</option>
                                </For>
                            </select>
                            <input type="date" class="input mb-3"
                                prop:value=move || h_date.get()
                                on:input=move |ev| h_date.set(event_target_value(&ev)) />
                            <div class="mb-3 flex gap-3">
                                <input type="number" step="0.1" class="input w-28" placeholder="Weight g"
                                    prop:value=move || h_weight.get()
                                    on:input=move |ev| h_weight.set(event_target_value(&ev)) />
                                <input type="number" class="input w-28" placeholder="Count"
                                    prop:value=move || h_count.get()
                                    on:input=move |ev| h_count.set(event_target_value(&ev)) />
                            </div>
                            <input type="text" class="input mb-3" placeholder="Notes"
                                prop:value=move || h_notes.get()
                                on:input=move |ev| h_notes.set(event_target_value(&ev)) />
                            <button class="btn-primary w-full" on:click=add_harvest>"Log harvest"</button>
                        </section>

                        <div class="space-y-6">
                            <section class="card p-4">
                                <div class="mb-3 flex items-center justify-between">
                                    <h2 class="text-xs font-black uppercase tracking-widest text-stone-500">"Yield summary"</h2>
                                    <select class="input w-36" on:change=move |ev| { yield_group.set(event_target_value(&ev)); reload_yield(); }>
                                        <option value="plant" selected=move || yield_group.get() == "plant">"By plant"</option>
                                        <option value="bed" selected=move || yield_group.get() == "bed">"By bed"</option>
                                        <option value="season" selected=move || yield_group.get() == "season">"By season"</option>
                                    </select>
                                </div>
                                <For each=move || yields.get() key=|y| y.key.clone() let:y>
                                    <div class="flex items-center justify-between border-b border-stone-800 py-1 text-sm">
                                        <span class="font-bold text-stone-200">{y.label.clone()}</span>
                                        <span class="text-stone-400">{format!("{}", y.harvest_count)} " harvests - " {format!("{} g", y.total_weight_g.map(|w| w as i64).unwrap_or(0))}</span>
                                    </div>
                                </For>
                                { move || if yields.get().is_empty() { view! { <p class="text-sm text-stone-500">"No harvests yet."</p> }.into_any() } else { view! { <span></span> }.into_any() } }
                            </section>

                            <section class="card p-4">
                                <h2 class="mb-3 text-xs font-black uppercase tracking-widest text-stone-500">"Harvest log"</h2>
                                <For each=move || harvests.get() key=|h| h.id.clone() let:h>
                                    {
                                        let h_delete = h.id.clone();
                                        let h_plant = h.plant_name.clone();
                                        let h_date = h.date.clone();
                                        let h_weight = h.weight_g;
                                        let h_count = h.count;
                                        let h_notes = h.notes.clone();
                                        view! {
                                            <div class="mb-2 flex items-start justify-between gap-2 rounded-lg border border-stone-800 bg-stone-950 p-3">
                                                <div>
                                                    <div class="text-sm font-bold text-stone-200">{h_plant.clone().unwrap_or_else(|| "Harvest".to_string())}</div>
                                                    <div class="text-[11px] text-stone-500">{h_date}
                                                        { if h_weight.is_some() { view! { <span>{" - "} {format!("{} g", h_weight.unwrap_or(0.0) as i64)}</span> }.into_any() } else { view! { <span></span> }.into_any() } }
                                                        { if h_count.is_some() { view! { <span>{" - "} {h_count.unwrap_or(0)} " items"</span> }.into_any() } else { view! { <span></span> }.into_any() } }
                                                    </div>
                                                    { if h_notes.is_some() { view! { <div class="text-[11px] text-stone-600">{h_notes.clone().unwrap_or_default()}</div> }.into_any() } else { view! { <span></span> }.into_any() } }
                                                </div>
                                                <button class="btn-ghost text-[10px]" on:click=move |_| delete_harvest(h_delete.clone())>"Delete"</button>
                                            </div>
                                        }
                                    }
                                </For>
                                { move || if harvests.get().is_empty() { view! { <p class="text-sm text-stone-500">"No harvests logged yet."</p> }.into_any() } else { view! { <span></span> }.into_any() } }
                            </section>
                        </div>
                    </div>
                }.into_any(),

                Mode::Supplies => view! {
                    <div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
                        <section class="card p-4">
                            <h2 class="mb-3 text-xs font-black uppercase tracking-widest text-stone-500">"Shopping list"</h2>
                            <div class="mb-3 flex gap-2">
                                <input type="text" class="input flex-1" placeholder="Item"
                                    prop:value=move || shop_name.get()
                                    on:input=move |ev| shop_name.set(event_target_value(&ev)) />
                                <select class="input w-28" on:change=move |ev| shop_category.set(event_target_value(&ev))>
                                    <option value="seeds">"Seeds"</option>
                                    <option value="soil">"Soil"</option>
                                    <option value="tools">"Tools"</option>
                                    <option value="other">"Other"</option>
                                </select>
                                <input type="text" class="input w-28" placeholder="Qty"
                                    prop:value=move || shop_qty.get()
                                    on:input=move |ev| shop_qty.set(event_target_value(&ev)) />
                            </div>
                            <button class="btn-primary w-full" on:click=add_shopping>"Add item"</button>
                            <div class="mt-3 space-y-1">
                                <For each=move || shopping.get() key=|i| i.id.clone() let:i>
                                    {
                                        let i_toggle = i.id.clone();
                                        let i_delete = i.id.clone();
                                        let i_name = i.name.clone();
                                        let i_qty = i.quantity.clone();
                                        let i_checked = i.checked;
                                        view! {
                                            <div class="flex items-center justify-between rounded-lg border border-stone-800 bg-stone-950 px-3 py-2">
                                                <label class="flex items-center gap-2 text-sm">
                                                    <input type="checkbox" prop:checked=move || i_checked on:change=move |_| toggle_shopping(i_toggle.clone()) />
                                                    <span class=move || if i_checked { "text-stone-500 line-through" } else { "text-stone-200" }>{i_name}</span>
                                                    { if i_qty.is_some() { view! { <span class="text-[10px] text-stone-500">{i_qty.clone().unwrap_or_default()}</span> }.into_any() } else { view! { <span></span> }.into_any() } }
                                                </label>
                                                <button class="btn-ghost text-[10px]" on:click=move |_| delete_shopping(i_delete.clone())>"x"</button>
                                            </div>
                                        }
                                    }
                                </For>
                            </div>
                        </section>

                        <section class="card p-4">
                            <h2 class="mb-3 text-xs font-black uppercase tracking-widest text-stone-500">"Tools"</h2>
                            <div class="mb-3 flex gap-2">
                                <input type="text" class="input flex-1" placeholder="Tool name"
                                    prop:value=move || tool_name.get()
                                    on:input=move |ev| tool_name.set(event_target_value(&ev)) />
                                <input type="date" class="input" placeholder="Next maintenance"
                                    prop:value=move || tool_next.get()
                                    on:input=move |ev| tool_next.set(event_target_value(&ev)) />
                            </div>
                            <input type="text" class="input mb-3" placeholder="Notes"
                                prop:value=move || tool_notes.get()
                                on:input=move |ev| tool_notes.set(event_target_value(&ev)) />
                            <button class="btn-primary w-full" on:click=add_tool>"Add tool"</button>
                            <div class="mt-3 space-y-2">
                                <For each=move || tools.get() key=|t| t.id.clone() let:t>
                                    {
                                        let t_id = t.id.clone();
                                        let t_maint_id = t.id.clone();
                                        let t_delete = t.id.clone();
                                        let t_name = t.name.clone();
                                        let t_next = t.next_maintenance.clone();
                                        let t_due = t.maintenance_due;
                                        let t_notes = t.notes.clone();
                                        view! {
                                            <div class=move || if t_due { "rounded-lg border border-amber-500/30 bg-amber-900/10 p-3" } else { "rounded-lg border border-stone-800 bg-stone-950 p-3" } >
                                                <div class="flex items-center justify-between gap-2">
                                                    <div>
                                                        <div class="text-sm font-bold text-stone-200">{t_name}</div>
                                                        <div class="text-[11px] text-stone-500">
                                                            { if t_next.is_some() { view! { <span>{"next: "} {t_next.clone().unwrap_or_default()}</span> }.into_any() } else { view! { <span>"no schedule"</span> }.into_any() } }
                                                        </div>
                                                        { if t_notes.is_some() { view! { <div class="text-[11px] text-stone-600">{t_notes.clone().unwrap_or_default()}</div> }.into_any() } else { view! { <span></span> }.into_any() } }
                                                        { if t_due { view! { <div class="text-[10px] text-amber-300">"maintenance due"</div> }.into_any() } else { view! { <span></span> }.into_any() } }
                                                    </div>
                                                    <div class="flex shrink-0 gap-2">
                                                        <button class="btn-ghost text-[10px]" on:click=move |_| { maintain_id.set(Some(t_maint_id.clone())); maintain_next.set(String::new()); }>"Maintain"</button>
                                                        <button class="btn-ghost text-[10px]" on:click=move |_| delete_tool(t_delete.clone())>"Delete"</button>
                                                    </div>
                                                </div>
                                                { let t_id_check = t_id.clone(); move || if maintain_id.get().as_deref() == Some(t_id_check.as_str()) {
                                                    let t_id_save = t_id_check.clone();
                                                    view! {
                                                        <div class="mt-2 flex gap-2">
                                                            <input type="date" class="input flex-1" placeholder="Next maintenance"
                                                                prop:value=move || maintain_next.get()
                                                                on:input=move |ev| maintain_next.set(event_target_value(&ev)) />
                                                            <button class="btn-primary" on:click=move |_| maintain_tool(t_id_save.clone())>"Save"</button>
                                                            <button class="btn-ghost" on:click=move |_| maintain_id.set(None)>"Cancel"</button>
                                                        </div>
                                                    }.into_any()
                                                } else {
                                                    view! { <span></span> }.into_any()
                                                } }
                                            </div>
                                        }
                                    }
                                </For>
                            </div>
                        </section>
                    </div>
                }.into_any(),
            } }
        </div>
    }
}
