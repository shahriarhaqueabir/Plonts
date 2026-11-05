use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, MouseEvent};

use crate::{canvas, ipc};
use garden_core::layout::{default_spacing_cm, hit_bed, hit_plant, snap_to_grid};
use garden_core::models::{
    BedShape, Garden, Layout, LayoutDiff, LayoutPlant, LayoutWarning, RotationPlan, SearchResult,
    WarningKind,
};

const CANVAS_W: f64 = 1000.0;
const CANVAS_H: f64 = 700.0;
const PLANT_HIT_CM: f64 = 12.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tool {
    AddBed,
    AddPlant,
    Select,
}

#[derive(Clone)]
enum Drag {
    NewBed { start_x: f64, start_y: f64, cur_x: f64, cur_y: f64 },
    Bed { index: usize, offset_x: f64, offset_y: f64 },
    Plant { id: String, offset_x: f64, offset_y: f64 },
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ListLayoutsArgs { garden_id: String }

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateLayoutArgs { garden_id: String, name: String }

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SaveLayoutArgs { id: String, canvas_json: String }

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct IdArgs { id: String }

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct LayoutIdArgs { layout_id: String }

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct CompareArgs { a_id: String, b_id: String }

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct AddPlantArgs { layout_id: String, catalog_id: String, bed_id: Option<String>, x: f64, y: f64, quantity: i64 }

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdatePlantArgs { id: String, bed_id: Option<String>, x: f64, y: f64, quantity: i64 }

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct RotationArgs { garden_id: String, season: String, plant_family: String, notes: Option<String> }

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchArgs { query: String, filters: Option<garden_core::models::SearchFilters>, limit: Option<usize> }

fn tool_class(current: Tool, this: Tool) -> &'static str {
    if current == this {
        "rounded-lg bg-garden-500/20 px-3 py-1.5 font-bold text-garden-300"
    } else {
        "rounded-lg px-3 py-1.5 text-stone-400 hover:bg-stone-900"
    }
}

fn warning_class(kind: WarningKind) -> &'static str {
    match kind {
        WarningKind::Overcrowding => "mb-1 rounded-md bg-amber-500/10 px-2 py-1 text-xs text-amber-300",
        WarningKind::Companion => "mb-1 rounded-md bg-rose-500/10 px-2 py-1 text-xs text-rose-300",
        WarningKind::Shade => "mb-1 rounded-md bg-sky-500/10 px-2 py-1 text-xs text-sky-300",
    }
}

#[component]
pub fn LayoutPage() -> impl IntoView {
    let gardens = RwSignal::new(Vec::<Garden>::new());
    let garden_id = RwSignal::new(None::<String>);
    let layouts = RwSignal::new(Vec::<Layout>::new());
    let layout_id = RwSignal::new(None::<String>);
    let layout = RwSignal::new(None::<Layout>);
    let beds = RwSignal::new(Vec::<BedShape>::new());
    let plants = RwSignal::new(Vec::<LayoutPlant>::new());
    let warnings = RwSignal::new(Vec::<LayoutWarning>::new());
    let palette = RwSignal::new(Vec::<SearchResult>::new());
    let selected_plant = RwSignal::new(None::<String>);
    let tool = RwSignal::new(Tool::AddBed);
    let drag = RwSignal::new(None::<Drag>);
    let cursor = RwSignal::new((0.0_f64, 0.0_f64));
    let new_bed_sun = RwSignal::new(None::<String>);
    let next_shape_id = RwSignal::new(0_u32);
    let rotations = RwSignal::new(Vec::<RotationPlan>::new());
    let rot_season = RwSignal::new("Spring".to_string());
    let rot_family = RwSignal::new(String::new());
    let rot_notes = RwSignal::new(String::new());
    let compare_a = RwSignal::new(None::<String>);
    let compare_b = RwSignal::new(None::<String>);
    let diff = RwSignal::new(None::<LayoutDiff>);

    let ctx = RwSignal::new(None::<CanvasRenderingContext2d>);
    let canvas_ref = NodeRef::<leptos::html::Canvas>::new();

    let spacing_for = move |catalog_id: &str| -> f64 {
        palette
            .get()
            .iter()
            .find(|r| r.plant.id == catalog_id)
            .and_then(|r| r.plant.categories.first().cloned())
            .map(|c| default_spacing_cm(&c))
            .unwrap_or(40.0)
    };

    let redraw = move || {
        let Some(ctx) = ctx.get() else { return; };
        canvas::clear(&ctx, CANVAS_W, CANVAS_H);
        canvas::draw_grid(&ctx, CANVAS_W, CANVAS_H);
        for b in beds.get().iter() {
            canvas::draw_bed(&ctx, b);
        }
        for p in plants.get().iter() {
            let radius = spacing_for(&p.catalog_id) / 2.0;
            canvas::draw_plant(&ctx, p, radius);
        }
        if let Some(d) = drag.get() {
            match d {
                Drag::NewBed { start_x, start_y, cur_x, cur_y } => {
                    canvas::draw_bed_rect(&ctx, start_x, start_y, cur_x, cur_y);
                }
                Drag::Bed { index, .. } => {
                    if let Some(b) = beds.get().get(index) {
                        canvas::draw_bed_highlight(&ctx, b);
                    }
                }
                Drag::Plant { .. } => {}
            }
        }
        if tool.get() == Tool::AddPlant {
            if let Some(pid) = selected_plant.get() {
                let radius = spacing_for(&pid) / 2.0;
                let (cx, cy) = cursor.get();
                canvas::draw_preview_circle(&ctx, cx, cy, radius);
            }
        }
    };

    let persist_beds = move || {
        let Some(lid) = layout_id.get() else { return; };
        let json = serde_json::to_string(&beds.get()).unwrap_or_else(|_| "[]".to_string());
        spawn_local(async move {
            let _ = ipc::call::<Layout, _>("save_layout", &SaveLayoutArgs { id: lid, canvas_json: json }).await;
        });
    };

    let refresh_warnings = move || {
        let Some(lid) = layout_id.get() else { return; };
        spawn_local(async move {
            if let Ok(ws) = ipc::call::<Vec<LayoutWarning>, _>("analyze_layout", &LayoutIdArgs { layout_id: lid }).await {
                warnings.set(ws);
            }
        });
    };

    let reload = move || {
        spawn_local(async move {
            if let Ok(gs) = ipc::call0::<Vec<Garden>>("list_gardens").await {
                gardens.set(gs);
            }
            if garden_id.get().is_none() {
                if let Some(g) = gardens.get().first() {
                    garden_id.set(Some(g.id.clone()));
                }
            }
            if let Ok(ps) = ipc::call::<Vec<SearchResult>, _>("search_plants", &SearchArgs {
                query: String::new(), filters: None, limit: Some(300),
            }).await {
                palette.set(ps);
            }
            let Some(gid) = garden_id.get() else { redraw(); return; };
            let largs = ListLayoutsArgs { garden_id: gid.clone() };
            if let Ok(ls) = ipc::call::<Vec<Layout>, _>("list_layouts", &largs).await {
                layouts.set(ls);
            }
            if layout_id.get().is_none() {
                if let Some(l) = layouts.get().first() {
                    layout_id.set(Some(l.id.clone()));
                } else if let Ok(c) = ipc::call::<Layout, _>("create_layout", &CreateLayoutArgs {
                    garden_id: gid.clone(), name: "Untitled layout".into(),
                }).await {
                    layout_id.set(Some(c.id));
                    if let Ok(ls) = ipc::call::<Vec<Layout>, _>("list_layouts", &largs).await {
                        layouts.set(ls);
                    }
                }
            }
            if let Some(lid) = layout_id.get() {
                if let Ok(l) = ipc::call::<Layout, _>("get_layout", &IdArgs { id: lid.clone() }).await {
                    let bs: Vec<BedShape> = serde_json::from_str(&l.canvas_json).unwrap_or_default();
                    beds.set(bs);
                    layout.set(Some(l));
                }
                if let Ok(ps) = ipc::call::<Vec<LayoutPlant>, _>("list_layout_plants", &LayoutIdArgs { layout_id: lid.clone() }).await {
                    plants.set(ps);
                }
                if let Ok(ws) = ipc::call::<Vec<LayoutWarning>, _>("analyze_layout", &LayoutIdArgs { layout_id: lid.clone() }).await {
                    warnings.set(ws);
                }
                if let Ok(rs) = ipc::call::<Vec<RotationPlan>, _>("list_rotation_plans", &ListLayoutsArgs { garden_id: gid.clone() }).await {
                    rotations.set(rs);
                }
            }
            redraw();
        });
    };

    let canvas_pos = move |ev: &MouseEvent| -> (f64, f64) {
        (
            snap_to_grid(ev.offset_x() as f64 / canvas::SCALE),
            snap_to_grid(ev.offset_y() as f64 / canvas::SCALE),
        )
    };

    let on_down = move |ev: MouseEvent| {
        let (x, y) = canvas_pos(&ev);
        match tool.get() {
            Tool::AddBed => {
                drag.set(Some(Drag::NewBed { start_x: x, start_y: y, cur_x: x, cur_y: y }));
            }
            Tool::AddPlant => {
                let Some(pid) = selected_plant.get() else { return; };
                let Some(lid) = layout_id.get() else { return; };
                let bed_id = hit_bed(&beds.get(), x, y).map(|i| beds.get()[i].id.clone());
                spawn_local(async move {
                    let _ = ipc::call::<LayoutPlant, _>("add_layout_plant", &AddPlantArgs {
                        layout_id: lid.clone(), catalog_id: pid, bed_id, x, y, quantity: 1,
                    }).await;
                    if let Ok(ps) = ipc::call::<Vec<LayoutPlant>, _>("list_layout_plants", &LayoutIdArgs { layout_id: lid.clone() }).await {
                        plants.set(ps);
                    }
                    if let Ok(ws) = ipc::call::<Vec<LayoutWarning>, _>("analyze_layout", &LayoutIdArgs { layout_id: lid }).await {
                        warnings.set(ws);
                    }
                    redraw();
                });
            }
            Tool::Select => {
                if let Some(i) = hit_plant(&plants.get(), x, y, PLANT_HIT_CM) {
                    let p = plants.get()[i].clone();
                    drag.set(Some(Drag::Plant { id: p.id, offset_x: x - p.x, offset_y: y - p.y }));
                } else if let Some(i) = hit_bed(&beds.get(), x, y) {
                    let b = beds.get()[i].clone();
                    drag.set(Some(Drag::Bed { index: i, offset_x: x - b.x, offset_y: y - b.y }));
                }
            }
        }
    };

    let on_move = move |ev: MouseEvent| {
        let (x, y) = canvas_pos(&ev);
        cursor.set((x, y));
        if let Some(d) = drag.get() {
            match d {
                Drag::NewBed { start_x, start_y, .. } => {
                    drag.set(Some(Drag::NewBed { start_x, start_y, cur_x: x, cur_y: y }));
                }
                Drag::Bed { index, offset_x, offset_y } => {
                    beds.update(|bs| {
                        if let Some(b) = bs.get_mut(index) {
                            b.x = snap_to_grid(x - offset_x);
                            b.y = snap_to_grid(y - offset_y);
                        }
                    });
                }
                Drag::Plant { id, offset_x, offset_y } => {
                    plants.update(|ps| {
                        if let Some(p) = ps.iter_mut().find(|p| p.id == id) {
                            p.x = snap_to_grid(x - offset_x);
                            p.y = snap_to_grid(y - offset_y);
                        }
                    });
                }
            }
        }
        redraw();
    };

    let on_up = move |_ev: MouseEvent| {
        if let Some(d) = drag.get() {
            match d {
                Drag::NewBed { start_x, start_y, cur_x, cur_y } => {
                    let x = start_x.min(cur_x);
                    let y = start_y.min(cur_y);
                    let w = (cur_x - start_x).abs();
                    let h = (cur_y - start_y).abs();
                    if w >= garden_core::layout::GRID_CM && h >= garden_core::layout::GRID_CM {
                        let id = {
                            let n = next_shape_id.get();
                            next_shape_id.set(n + 1);
                            format!("shape_{}", n)
                        };
                        beds.update(|bs| bs.push(BedShape {
                            id,
                            name: format!("Bed {}", bs.len() + 1),
                            x, y, width_cm: w, height_cm: h,
                            sun_exposure: new_bed_sun.get(),
                        }));
                        persist_beds();
                        refresh_warnings();
                    }
                }
                Drag::Bed { .. } => {
                    persist_beds();
                }
                Drag::Plant { id, .. } => {
                    // capture plant data before spawning
                    let snapshot = plants.get().iter().find(|p| p.id == id).cloned();
                    if let Some(p) = snapshot {
                        let bed_id = hit_bed(&beds.get(), p.x, p.y).map(|i| beds.get()[i].id.clone());
                        let pid = p.id.clone();
                        let px = p.x;
                        let py = p.y;
                        let qty = p.quantity;
                        spawn_local(async move {
                            let _ = ipc::call::<LayoutPlant, _>("update_layout_plant", &UpdatePlantArgs {
                                id: pid, bed_id, x: px, y: py, quantity: qty,
                            }).await;
                            refresh_warnings();
                        });
                    }
                }
            }
        }
        drag.set(None);
        redraw();
    };

    let add_rotation = move |_| {
        let Some(gid) = garden_id.get() else { return; };
        let season = rot_season.get();
        let family = rot_family.get();
        let notes = rot_notes.get();
        let notes = if notes.trim().is_empty() { None } else { Some(notes) };
        spawn_local(async move {
            let _ = ipc::call::<RotationPlan, _>("create_rotation_plan", &RotationArgs {
                garden_id: gid.clone(), season, plant_family: family, notes,
            }).await;
            if let Ok(rs) = ipc::call::<Vec<RotationPlan>, _>("list_rotation_plans", &ListLayoutsArgs { garden_id: gid }).await {
                rotations.set(rs);
            }
        });
    };

    let remove_rotation = move |id: String| {
        spawn_local(async move {
            let _ = ipc::call::<serde_json::Value, _>("delete_rotation_plan", &IdArgs { id }).await;
            if let Some(gid) = garden_id.get() {
                if let Ok(rs) = ipc::call::<Vec<RotationPlan>, _>("list_rotation_plans", &ListLayoutsArgs { garden_id: gid }).await {
                    rotations.set(rs);
                }
            }
        });
    };

    let duplicate = move |_| {
        let Some(lid) = layout_id.get() else { return; };
        spawn_local(async move {
            if let Ok(dup) = ipc::call::<Layout, _>("duplicate_layout", &IdArgs { id: lid }).await {
                layout_id.set(Some(dup.id));
                reload();
            }
        });
    };

    let compare = move |_| {
        let (Some(a), Some(b)) = (compare_a.get(), compare_b.get()) else { return; };
        spawn_local(async move {
            if let Ok(d) = ipc::call::<LayoutDiff, _>("compare_layouts", &CompareArgs { a_id: a, b_id: b }).await {
                diff.set(Some(d));
            }
        });
    };

    let export_png = move |_| {
        let Some(ctx) = ctx.get() else { return; };
        let Some(canvas_el) = ctx.canvas() else { return; };
        let data_url = canvas_el.to_data_url_with_type("image/png").unwrap_or_default();
        let base64 = data_url.trim_start_matches("data:image/png;base64,").to_string();
        spawn_local(async move {
            let Some(path) = ipc::pick_png_save_path().await.ok().flatten() else { return; };
            let _ = ipc::save_export(&path, &base64).await;
        });
    };

    canvas_ref.on_load(move |canvas: HtmlCanvasElement| {
        canvas.set_width(CANVAS_W as u32);
        canvas.set_height(CANVAS_H as u32);
        let c = canvas
            .get_context("2d")
            .ok()
            .flatten()
            .and_then(|o| o.dyn_into::<CanvasRenderingContext2d>().ok());
        ctx.set(c);
        reload();
    });

    view! {
        <div class="flex h-full">
            <div class="flex-1 p-4">
                <div class="mb-3 flex items-center gap-2">
                    <h1 class="text-xl font-bold text-stone-100">"Layout Planning"</h1>
                    <select class="input w-48" on:change=move |ev| { garden_id.set(Some(event_target_value(&ev))); layout_id.set(None); reload(); }>
                        <For each=move || gardens.get() key=|g| g.id.clone() let:g>
                            <option value=g.id.clone() selected=move || garden_id.get().as_deref() == Some(g.id.as_str())>
                                {g.name.clone()}
                            </option>
                        </For>
                    </select>
                    <select class="input w-48" on:change=move |ev| { layout_id.set(Some(event_target_value(&ev))); reload(); }>
                        <For each=move || layouts.get() key=|l| l.id.clone() let:l>
                            <option value=l.id.clone() selected=move || layout_id.get().as_deref() == Some(l.id.as_str())>
                                {format!("{} (v{})", l.name, l.version)}
                            </option>
                        </For>
                    </select>
                    <button class="btn-primary" on:click=move |_| persist_beds()>"Save"</button>
                </div>

                <div class="mb-3 flex items-center gap-2 text-xs">
                    <button class=move || tool_class(tool.get(), Tool::AddBed) on:click=move |_| tool.set(Tool::AddBed)>"Add bed"</button>
                    <button class=move || tool_class(tool.get(), Tool::AddPlant) on:click=move |_| tool.set(Tool::AddPlant)>"Add plant"</button>
                    <button class=move || tool_class(tool.get(), Tool::Select) on:click=move |_| tool.set(Tool::Select)>"Select / move"</button>
                    <label class="ml-4 text-stone-400">"Bed sun:"</label>
                    <select class="input w-40" on:change=move |ev| {
                        let v = event_target_value(&ev);
                        new_bed_sun.set(if v.is_empty() { None } else { Some(v) });
                    }>
                        <option value="">"—"</option>
                        <option value="full_sun">"Full sun"</option>
                        <option value="partial_shade">"Partial shade"</option>
                        <option value="shade">"Shade"</option>
                    </select>
                </div>

                <canvas
                    node_ref=canvas_ref
                    class="rounded-xl border border-stone-800 bg-stone-950"
                    style="width:1000px;height:700px"
                    on:mousedown=move |ev| on_down(ev)
                    on:mousemove=move |ev| on_move(ev)
                    on:mouseup=move |ev| on_up(ev)
                    on:mouseleave=move |_| { drag.set(None); redraw(); }
                ></canvas>
            </div>

            <aside class="w-80 shrink-0 space-y-4 overflow-auto border-l border-stone-800 p-4">
                <Show when=move || tool.get() == Tool::AddPlant fallback=|| view! { <span></span> }>
                    <section class="card p-3">
                        <h2 class="mb-2 text-xs font-black uppercase tracking-widest text-stone-500">"Plant palette"</h2>
                        <div class="max-h-64 overflow-auto">
                            <For each=move || palette.get() key=|r| r.plant.id.clone() let:r>
                                {
                                    let r_class = r.clone();
                                    let r_click = r.clone();
                                    let name = r.plant.name.clone();
                                    view! {
                                        <button
                                            class=move || {
                                                if selected_plant.get().as_deref() == Some(r_class.plant.id.as_str()) {
                                                    "mb-1 block w-full rounded-lg bg-garden-500/20 px-2 py-1 text-left text-xs font-bold text-garden-300"
                                                } else {
                                                    "mb-1 block w-full rounded-lg px-2 py-1 text-left text-xs text-stone-300 hover:bg-stone-900"
                                                }
                                            }
                                            on:click=move |_| selected_plant.set(Some(r_click.plant.id.clone()))
                                        >
                                            {name}
                                        </button>
                                    }
                                }
                            </For>
                        </div>
                    </section>
                </Show>

                <section class="card p-3">
                    <h2 class="mb-2 text-xs font-black uppercase tracking-widest text-stone-500">"Warnings"</h2>
                    <For each=move || warnings.get() key=|w| format!("{:?}-{}-{:?}", w.kind, w.plant_a, w.plant_b) let:w>
                        <div class=move || warning_class(w.kind)>{w.message.clone()}</div>
                    </For>
                    { move || if warnings.get().is_empty() {
                        view! { <p class="text-xs text-stone-500">"No warnings."</p> }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    } }
                </section>

                <section class="card p-3">
                    <h2 class="mb-2 text-xs font-black uppercase tracking-widest text-stone-500">"Crop rotation"</h2>
                    <select class="input mb-2" prop:value=move || rot_season.get() on:change=move |ev| rot_season.set(event_target_value(&ev))>
                        <option value="Spring">"Spring"</option>
                        <option value="Summer">"Summer"</option>
                        <option value="Autumn">"Autumn"</option>
                        <option value="Winter">"Winter"</option>
                    </select>
                    <input type="text" class="input mb-2" placeholder="Plant family (e.g. Solanaceae)"
                        prop:value=move || rot_family.get() on:input=move |ev| rot_family.set(event_target_value(&ev)) />
                    <input type="text" class="input mb-2" placeholder="Notes (optional)"
                        prop:value=move || rot_notes.get() on:input=move |ev| rot_notes.set(event_target_value(&ev)) />
                    <button class="btn-primary w-full" on:click=add_rotation>"Add rotation plan"</button>
                    <div class="mt-2">
                        <For each=move || rotations.get() key=|r| r.id.clone() let:r>
                            <div class="mb-1 flex items-center gap-2 rounded-lg border border-stone-800 p-2">
                                <div class="flex-1 text-xs">
                                    <span class="font-bold text-stone-200">{r.season.clone()}</span>
                                    " - " {r.plant_family.clone()}
                                </div>
                                <button class="btn-ghost text-[10px]" on:click=move |_| remove_rotation(r.id.clone())>"Delete"</button>
                            </div>
                        </For>
                    </div>
                </section>

                <section class="card p-3">
                    <h2 class="mb-2 text-xs font-black uppercase tracking-widest text-stone-500">"Versions"</h2>
                    <button class="btn-primary mb-2 w-full" on:click=duplicate>"Duplicate layout"</button>
                    <div class="mb-2 flex gap-2">
                        <select class="input flex-1" on:change=move |ev| compare_a.set(Some(event_target_value(&ev)))>
                            <option value="">"— base"</option>
                            <For each=move || layouts.get() key=|l| l.id.clone() let:l>
                                <option value=l.id.clone()>{l.name.clone()}</option>
                            </For>
                        </select>
                        <select class="input flex-1" on:change=move |ev| compare_b.set(Some(event_target_value(&ev)))>
                            <option value="">"— other"</option>
                            <For each=move || layouts.get() key=|l| l.id.clone() let:l>
                                <option value=l.id.clone()>{l.name.clone()}</option>
                            </For>
                        </select>
                    </div>
                    <button class="btn-primary mb-2 w-full" on:click=compare>"Compare"</button>
                    { move || diff.get().map(|d| view! {
                        <div class="space-y-1 text-xs">
                            <div class="text-green-400">"Added: " {d.added.len()}</div>
                            <div class="text-rose-400">"Removed: " {d.removed.len()}</div>
                            <div class="text-amber-400">"Moved: " {d.moved.len()}</div>
                        </div>
                    }) }
                </section>

                <section class="card p-3">
                    <h2 class="mb-2 text-xs font-black uppercase tracking-widest text-stone-500">"Export"</h2>
                    <button class="btn-primary w-full" on:click=export_png>"Export as PNG"</button>
                </section>
            </aside>
        </div>
    }
}
