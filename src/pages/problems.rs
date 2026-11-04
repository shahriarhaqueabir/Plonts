use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::ipc;
use garden_core::models::{Planting, Problem, ProblemCause, SearchResult, Symptom};

#[derive(serde::Serialize)]
struct DiagnoseArgs { symptom: String, plant_id: Option<String> }

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateProblemArgs {
    planting_id: Option<String>,
    symptom: String,
    affected_part: Option<String>,
    spread: Option<String>,
    cause: String,
    treatment: String,
    prevention: String,
    follow_up_at: Option<String>,
}

#[derive(serde::Serialize)]
struct ResolveArgs { id: String, resolution_note: Option<String> }

#[derive(serde::Serialize)]
struct IdArgs { id: String }

#[derive(serde::Serialize)]
struct SearchArgs { query: String, filters: Option<garden_core::models::SearchFilters>, limit: Option<usize> }

fn symptom_label(s: Symptom) -> &'static str {
    match s {
        Symptom::YellowLeaves => "Yellow leaves",
        Symptom::Spots => "Spots",
        Symptom::Holes => "Holes",
        Symptom::Wilting => "Wilting",
        Symptom::WhitePowder => "White powder",
    }
}

#[component]
pub fn ProblemsPage() -> impl IntoView {
    let symptom = RwSignal::new("yellow_leaves".to_string());
    let diag_plant = RwSignal::new(None::<String>);
    let affected_part = RwSignal::new(String::new());
    let spread = RwSignal::new(String::new());
    let causes = RwSignal::new(Vec::<ProblemCause>::new());
    let selected_cause = RwSignal::new(None::<ProblemCause>);

    let problems = RwSignal::new(Vec::<Problem>::new());
    let plants = RwSignal::new(Vec::<SearchResult>::new());
    let plantings = RwSignal::new(Vec::<Planting>::new());

    let log_planting = RwSignal::new(None::<String>);
    let follow_up = RwSignal::new(String::new());
    let resolve_id = RwSignal::new(None::<String>);
    let resolve_note = RwSignal::new(String::new());

    let reload_problems = move || {
        spawn_local(async move {
            if let Ok(p) = ipc::call0::<Vec<Problem>>("list_problems").await {
                problems.set(p);
            }
        });
    };

    let run_diagnosis = move || {
        let s = symptom.get();
        let pid = diag_plant.get();
        spawn_local(async move {
            let args = DiagnoseArgs { symptom: s, plant_id: pid };
            if let Ok(c) = ipc::call::<Vec<ProblemCause>, _>("diagnose_problem", &args).await {
                causes.set(c);
                selected_cause.set(None);
            }
        });
    };

    spawn_local(async move {
        if let Ok(ps) = ipc::call::<Vec<SearchResult>, _>("search_plants", &SearchArgs { query: String::new(), filters: None, limit: Some(300) }).await {
            plants.set(ps);
        }
        if let Ok(pl) = ipc::call0::<Vec<Planting>>("list_plantings").await {
            plantings.set(pl);
        }
        reload_problems();
        run_diagnosis();
    });

    let log_problem = move |_| {
        let Some(cause) = selected_cause.get() else { return; };
        let args = CreateProblemArgs {
            planting_id: log_planting.get(),
            symptom: symptom.get(),
            affected_part: if affected_part.get().trim().is_empty() { None } else { Some(affected_part.get()) },
            spread: if spread.get().trim().is_empty() { None } else { Some(spread.get()) },
            cause: cause.name.clone(),
            treatment: cause.treatment.clone(),
            prevention: cause.prevention.clone(),
            follow_up_at: if follow_up.get().trim().is_empty() { None } else { Some(follow_up.get()) },
        };
        spawn_local(async move {
            let _ = ipc::call::<Problem, _>("create_problem", &args).await;
            selected_cause.set(None);
            reload_problems();
        });
    };

    let resolve = move |id: String| {
        let note = resolve_note.get();
        let note = if note.trim().is_empty() { None } else { Some(note) };
        spawn_local(async move {
            let _ = ipc::call::<Problem, _>("resolve_problem", &ResolveArgs { id, resolution_note: note }).await;
            resolve_id.set(None);
            resolve_note.set(String::new());
            reload_problems();
        });
    };

    let delete = move |id: String| {
        spawn_local(async move {
            let _ = ipc::call::<serde_json::Value, _>("delete_problem", &IdArgs { id }).await;
            reload_problems();
        });
    };

    view! {
        <div class="p-6">
            <h1 class="mb-1 text-xl font-bold text-stone-100">"Problems"</h1>
            <p class="mb-4 text-sm text-stone-400">"Diagnose plant problems and track treatments."</p>

            <section class="card mb-4 p-4">
                <h2 class="mb-3 text-xs font-black uppercase tracking-widest text-stone-500">"Symptom picker"</h2>
                <div class="grid grid-cols-1 gap-3 md:grid-cols-2">
                    <select class="input" on:change=move |ev| { symptom.set(event_target_value(&ev)); run_diagnosis(); }>
                        <option value="yellow_leaves">"Yellow leaves"</option>
                        <option value="spots">"Spots"</option>
                        <option value="holes">"Holes"</option>
                        <option value="wilting">"Wilting"</option>
                        <option value="white_powder">"White powder"</option>
                    </select>
                    <select class="input" on:change=move |ev| {
                        let v = event_target_value(&ev);
                        diag_plant.set(if v.is_empty() { None } else { Some(v) });
                        run_diagnosis();
                    }>
                        <option value="">"Any plant"</option>
                        <For each=move || plants.get() key=|p| p.plant.id.clone() let:p>
                            <option value=p.plant.id.clone()>{p.plant.name.clone()}</option>
                        </For>
                    </select>
                    <input type="text" class="input" placeholder="Affected part (e.g. lower leaves)"
                        prop:value=move || affected_part.get()
                        on:input=move |ev| affected_part.set(event_target_value(&ev)) />
                    <select class="input" on:change=move |ev| spread.set(event_target_value(&ev))>
                        <option value="">"Spread (optional)"</option>
                        <option value="few_leaves">"A few leaves"</option>
                        <option value="one_branch">"One branch"</option>
                        <option value="whole_plant">"Whole plant"</option>
                        <option value="spreading">"Spreading to other plants"</option>
                    </select>
                </div>
            </section>

            <div class="space-y-3">
                <For each=move || causes.get() key=|c| c.id.clone() let:c>
                    {
                        let c_log = c.clone();
                        let c_name = c.name.clone();
                        let c_desc = c.description.clone();
                        let c_treat = c.treatment.clone();
                        let c_prev = c.prevention.clone();
                        let c_org = c.organic;
                        view! {
                            <div class="card p-4">
                                <div class="flex items-center justify-between gap-2">
                                    <div class="font-bold text-stone-100">
                                        {c_name}
                                        { if c_org { view! { <span class="ml-2 rounded border border-garden-500/20 bg-garden-900/10 px-1.5 py-0.5 text-[10px] text-garden-300">"organic"</span> }.into_any() } else { view! { <span></span> }.into_any() } }
                                    </div>
                                    <button class="btn-ghost text-[10px]" on:click=move |_| selected_cause.set(Some(c_log.clone()))>"Log this problem"</button>
                                </div>
                                <p class="mt-1 text-sm text-stone-400">{c_desc}</p>
                                <div class="mt-2 text-sm">
                                    <span class="font-bold text-garden-400">"Treatment: "</span><span class="text-stone-300">{c_treat}</span>
                                </div>
                                <div class="mt-1 text-sm">
                                    <span class="font-bold text-stone-500">"Prevention: "</span><span class="text-stone-400">{c_prev}</span>
                                </div>
                            </div>
                        }
                    }
                </For>
            </div>

            { move || selected_cause.get().map(|c| {
                let c_name = c.name.clone();
                view! {
                    <section class="card mt-4 border-garden-500/40 p-4">
                        <h2 class="mb-3 text-xs font-black uppercase tracking-widest text-stone-500">"Log issue"</h2>
                        <p class="mb-3 text-sm text-stone-300">"Cause: " {c_name}</p>
                        <div class="mb-3 flex gap-3">
                            <select class="input" on:change=move |ev| {
                                let v = event_target_value(&ev);
                                log_planting.set(if v.is_empty() { None } else { Some(v) });
                            }>
                                <option value="">"No planting"</option>
                                <For each=move || plantings.get() key=|p| p.id.clone() let:p>
                                    <option value=p.id.clone()>{format!("{} - {}", p.plant_name, p.bed_name)}</option>
                                </For>
                            </select>
                            <input type="date" class="input" placeholder="Follow-up date"
                                prop:value=move || follow_up.get()
                                on:input=move |ev| follow_up.set(event_target_value(&ev)) />
                        </div>
                        <div class="flex gap-2">
                            <button class="btn-primary" on:click=log_problem>"Save problem"</button>
                            <button class="btn-ghost" on:click=move |_| selected_cause.set(None)>"Cancel"</button>
                        </div>
                    </section>
                }.into_any()
            }) }

            <h2 class="mt-6 mb-3 text-xs font-black uppercase tracking-widest text-stone-500">"Logged problems"</h2>
            <div class="space-y-3">
                <For each=move || problems.get() key=|p| p.id.clone() let:p>
                    {
                        let p_id = p.id.clone();
                        let p_resolve_id = p.id.clone();
                        let p_delete_id = p.id.clone();
                        let p_planting = p.planting_name.clone();
                        let p_follow = p.follow_up_at.clone();
                        let p_symptom = p.symptom;
                        let p_cause = p.cause.clone();
                        let p_resolved = p.resolved;
                        let p_note = p.resolution_note.clone();
                        view! {
                            <div class=move || if p_resolved { "card p-4 opacity-60" } else { "card p-4" }>
                                <div class="flex items-start justify-between gap-3">
                                    <div>
                                        <div class="flex flex-wrap items-center gap-2">
                                            <span class="text-sm font-bold text-stone-100">{symptom_label(p_symptom)}</span>
                                            { if p_resolved { view! { <span class="rounded border border-garden-500/20 bg-garden-900/10 px-1.5 py-0.5 text-[10px] text-garden-300">"resolved"</span> }.into_any() } else { view! { <span class="rounded border border-amber-500/20 bg-amber-900/10 px-1.5 py-0.5 text-[10px] text-amber-300">"open"</span> }.into_any() } }
                                            { if !p_resolved && p_follow.is_some() { view! { <span class="rounded border border-stone-800 bg-stone-900 px-1.5 py-0.5 text-[10px] text-stone-400">{"follow-up "} {p_follow.clone().unwrap_or_default()}</span> }.into_any() } else { view! { <span></span> }.into_any() } }
                                        </div>
                                        <div class="mt-1 text-xs text-stone-400">{p_cause}</div>
                                        { if p_note.is_some() { view! { <div class="mt-1 text-[10px] text-stone-500">{"worked: "} {p_note.clone().unwrap_or_default()}</div> }.into_any() } else { view! { <span></span> }.into_any() } }
                                        { if p_planting.is_some() { view! { <div class="mt-1 text-[10px] text-stone-600">{"planting: "} {p_planting.clone().unwrap_or_default()}</div> }.into_any() } else { view! { <span></span> }.into_any() } }
                                    </div>
                                    <div class="flex shrink-0 gap-2">
                                        { if !p_resolved {
                                            view! { <button class="btn-ghost text-[10px]" on:click=move |_| { resolve_id.set(Some(p_resolve_id.clone())); resolve_note.set(String::new()); }>"Resolve"</button> }.into_any()
                                        } else {
                                            view! { <span></span> }.into_any()
                                        } }
                                        <button class="btn-ghost text-[10px]" on:click=move |_| delete(p_delete_id.clone())>"Delete"</button>
                                    </div>
                                </div>
                                { let p_id_check = p_id.clone(); move || if resolve_id.get().as_deref() == Some(p_id_check.as_str()) {
                                    let p_id_save = p_id_check.clone();
                                    view! {
                                        <div class="mt-3 flex gap-2">
                                            <input type="text" class="input flex-1" placeholder="What worked?"
                                                prop:value=move || resolve_note.get()
                                                on:input=move |ev| resolve_note.set(event_target_value(&ev)) />
                                            <button class="btn-primary" on:click=move |_| resolve(p_id_save.clone())>"Save"</button>
                                            <button class="btn-ghost" on:click=move |_| resolve_id.set(None)>"Cancel"</button>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! { <span></span> }.into_any()
                                } }
                            </div>
                        }
                    }
                </For>
                { move || if problems.get().is_empty() {
                    view! { <p class="text-sm text-stone-500">"No problems logged yet."</p> }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                } }
            </div>
        </div>
    }
}