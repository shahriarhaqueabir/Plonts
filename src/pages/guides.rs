use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::ipc;
use garden_core::models::{DailyTip, Guide, GuideFilters, GuideProgress, GuideTopic, SkillLevel};

#[derive(serde::Serialize)]
struct ListGuidesArgs { filters: Option<GuideFilters> }

#[derive(serde::Serialize)]
struct GetGuideArgs { id: String }

#[derive(serde::Serialize)]
struct SaveProgressArgs { guide_id: String, current_step: i64, completed: bool }

#[derive(serde::Serialize)]
struct BookmarkArgs { guide_id: String, bookmarked: bool }

#[derive(serde::Serialize)]
struct DismissTipArgs { id: String, days: i64 }

#[derive(serde::Serialize)]
struct SnoozeTipArgs { id: String, hours: i64 }

fn topic_label(t: GuideTopic) -> &'static str {
    match t {
        GuideTopic::Soil => "Soil",
        GuideTopic::Composting => "Composting",
        GuideTopic::Pruning => "Pruning",
        GuideTopic::Propagation => "Propagation",
        GuideTopic::Pests => "Pests",
        GuideTopic::Seasonal => "Seasonal",
    }
}

fn skill_label(s: SkillLevel) -> &'static str {
    match s {
        SkillLevel::Beginner => "Beginner",
        SkillLevel::Intermediate => "Intermediate",
        SkillLevel::Advanced => "Advanced",
    }
}

#[component]
pub fn GuidesPage() -> impl IntoView {
    let guides = RwSignal::new(Vec::<Guide>::new());
    let detail = RwSignal::new(None::<Guide>);
    let tip = RwSignal::new(None::<DailyTip>);
    let topic = RwSignal::new(None::<GuideTopic>);
    let skill = RwSignal::new(None::<SkillLevel>);
    let time_max = RwSignal::new(None::<i64>);

    let current_step = Signal::derive(move || {
        detail.get().and_then(|g| g.progress.clone()).map(|p| p.current_step).unwrap_or(0)
    });
    let total_steps = Signal::derive(move || {
        detail.get().map(|g| g.steps.len() as i64).unwrap_or(0)
    });
    let is_complete = Signal::derive(move || {
        detail.get().and_then(|g| g.progress.clone()).map(|p| p.completed).unwrap_or(false)
    });
    let steps_indexed = Signal::derive(move || {
        detail.get()
            .map(|g| g.steps.iter().cloned().enumerate().map(|(i, s)| (i as i64, s)).collect::<Vec<_>>())
            .unwrap_or_default()
    });

    let reload = move || {
        let filters = GuideFilters {
            topic: topic.get(),
            skill_level: skill.get(),
            time_max_minutes: time_max.get(),
        };
        spawn_local(async move {
            let args = ListGuidesArgs { filters: Some(filters) };
            if let Ok(g) = ipc::call::<Vec<Guide>, _>("list_guides", &args).await {
                guides.set(g);
            }
        });
    };

    let reload_tip = move || {
        spawn_local(async move {
            if let Ok(t) = ipc::call0::<Option<DailyTip>>("get_daily_tip").await {
                tip.set(t);
            }
        });
    };

    let refresh_detail = move |id: String| {
        spawn_local(async move {
            let args = GetGuideArgs { id };
            if let Ok(g) = ipc::call::<Guide, _>("get_guide", &args).await {
                detail.set(Some(g));
            }
        });
    };

    spawn_local(async move {
        reload();
        reload_tip();
    });

    let toggle_bookmark = move |_| {
        if let Some(g) = detail.get() {
            let id = g.id.clone();
            let mark = !g.bookmarked;
            spawn_local(async move {
                let args = BookmarkArgs { guide_id: id.clone(), bookmarked: mark };
                if ipc::call::<Guide, _>("set_guide_bookmark", &args).await.is_ok() {
                    refresh_detail(id.clone());
                    reload();
                }
            });
        }
    };

    let mark_step_done = move |_| {
        if let Some(g) = detail.get() {
            let id = g.id.clone();
            let total = g.steps.len() as i64;
            let cur = g.progress.as_ref().map(|p| p.current_step).unwrap_or(0);
            let next = (cur + 1).min(total);
            let done = next >= total;
            spawn_local(async move {
                let args = SaveProgressArgs { guide_id: id.clone(), current_step: next, completed: done };
                if ipc::call::<GuideProgress, _>("save_guide_progress", &args).await.is_ok() {
                    refresh_detail(id);
                    reload();
                }
            });
        }
    };

    let reset_progress = move |_| {
        if let Some(g) = detail.get() {
            let id = g.id.clone();
            spawn_local(async move {
                let args = SaveProgressArgs { guide_id: id.clone(), current_step: 0, completed: false };
                if ipc::call::<GuideProgress, _>("save_guide_progress", &args).await.is_ok() {
                    refresh_detail(id);
                    reload();
                }
            });
        }
    };

    let dismiss_tip = move |_| {
        if let Some(t) = tip.get() {
            let id = t.id.clone();
            spawn_local(async move {
                let args = DismissTipArgs { id, days: 1 };
                let _ = ipc::call::<DailyTip, _>("dismiss_daily_tip", &args).await;
                reload_tip();
            });
        }
    };

    let snooze_tip = move |_| {
        if let Some(t) = tip.get() {
            let id = t.id.clone();
            spawn_local(async move {
                let args = SnoozeTipArgs { id, hours: 8 };
                let _ = ipc::call::<DailyTip, _>("snooze_daily_tip", &args).await;
                reload_tip();
            });
        }
    };

    let on_topic = move |ev: leptos::ev::Event| {
        let v = event_target_value(&ev);
        topic.set(if v.is_empty() { None } else {
            Some(match v.as_str() {
                "composting" => GuideTopic::Composting,
                "pruning" => GuideTopic::Pruning,
                "propagation" => GuideTopic::Propagation,
                "pests" => GuideTopic::Pests,
                "seasonal" => GuideTopic::Seasonal,
                _ => GuideTopic::Soil,
            })
        });
        reload();
    };
    let on_skill = move |ev: leptos::ev::Event| {
        let v = event_target_value(&ev);
        skill.set(if v.is_empty() { None } else {
            Some(match v.as_str() {
                "intermediate" => SkillLevel::Intermediate,
                "advanced" => SkillLevel::Advanced,
                _ => SkillLevel::Beginner,
            })
        });
        reload();
    };
    let on_time = move |ev: leptos::ev::Event| {
        let v = event_target_value(&ev);
        time_max.set(if v.is_empty() { None } else { v.parse::<i64>().ok() });
        reload();
    };

    view! {
        <div class="p-6">
            <h1 class="mb-1 text-xl font-bold text-stone-100">"Guides"</h1>
            <p class="mb-4 text-sm text-stone-400">"Step-by-step guides by topic, skill level and time."</p>

            { move || tip.get().map(|t| view! {
                <div class="card mb-6 flex items-start justify-between gap-4 border-amber-500/20 bg-amber-900/10 p-4">
                    <div>
                        <div class="text-xs font-black uppercase tracking-widest text-amber-400">"Daily tip"</div>
                        <div class="mt-1 font-bold text-stone-100">{t.title.clone()}</div>
                        <div class="text-sm text-stone-400">{t.body.clone()}</div>
                    </div>
                    <div class="flex shrink-0 gap-2">
                        <button class="btn-ghost" on:click=dismiss_tip>"Dismiss"</button>
                        <button class="btn-ghost" on:click=snooze_tip>"Snooze"</button>
                    </div>
                </div>
            }) }

            { move || match detail.get() {
                Some(g) => view! {
                    <div>
                        <button class="mb-4 text-xs font-bold text-stone-400 hover:text-stone-200"
                            on:click=move |_| detail.set(None)>"<- Back to guides"</button>

                        <div class="card mb-4 p-4">
                            <div class="flex items-start justify-between">
                                <div>
                                    <h2 class="text-lg font-bold text-stone-100">{g.title.clone()}</h2>
                                    <div class="mt-1 flex flex-wrap gap-1">
                                        <span class="rounded border border-stone-800 bg-stone-900 px-1.5 py-0.5 text-[10px] text-stone-400">{topic_label(g.topic)}</span>
                                        <span class="rounded border border-stone-800 bg-stone-900 px-1.5 py-0.5 text-[10px] text-stone-400">{skill_label(g.skill_level)}</span>
                                        <span class="rounded border border-stone-800 bg-stone-900 px-1.5 py-0.5 text-[10px] text-stone-400">{format!("{} min", g.time_minutes)}</span>
                                    </div>
                                </div>
                                <button class="btn-ghost" on:click=toggle_bookmark>
                                    { if g.bookmarked { "Unbookmark" } else { "Bookmark" } }
                                </button>
                            </div>
                            <div class="mt-3 text-xs font-bold text-stone-400">
                                { move || format!(
                                    "Progress: {} of {} steps{}",
                                    current_step.get().min(total_steps.get()),
                                    total_steps.get(),
                                    if is_complete.get() { " (complete)" } else { "" },
                                ) }
                            </div>
                        </div>

                        <div class="card mb-4 p-4">
                            <h3 class="mb-2 text-sm font-bold text-garden-400">"Tools & materials"</h3>
                            <ul class="flex flex-wrap gap-2">
                                <For each=move || g.materials.clone() key=|m| m.clone() let:m>
                                    <li class="rounded border border-stone-800 bg-stone-900 px-2 py-1 text-xs text-stone-300">{m}</li>
                                </For>
                            </ul>
                        </div>

                        <div class="space-y-3">
                            <For each=move || steps_indexed.get() key=|(_, s)| s.id.clone() let:item>
                                {
                                    let (index, s) = item;
                                    let done = move || index < current_step.get();
                                    view! {
                                        <div class=move || if done() { "card p-4 border-garden-500/40" } else { "card p-4" } >
                                            <div class="flex items-center gap-2">
                                                <span class="flex h-5 w-5 items-center justify-center rounded-full border border-stone-700 text-[10px] text-stone-400">
                                                    { move || if done() { "v".to_string() } else { index.to_string() } }
                                                </span>
                                                <span class="font-bold text-stone-100">{s.title.clone()}</span>
                                            </div>
                                            <p class="mt-1 text-sm text-stone-400">{s.body.clone()}</p>
                                            <ul class="mt-2 space-y-1">
                                                <For each=move || s.checklist.clone() key=|c| c.clone() let:c>
                                                    <li class="flex items-center gap-2 text-xs text-stone-400">
                                                        <span class="text-garden-500">"-"</span>{c}
                                                    </li>
                                                </For>
                                            </ul>
                                        </div>
                                    }
                                }
                            </For>
                        </div>

                        <div class="mt-4 flex gap-3">
                            <button class="btn-primary" on:click=mark_step_done>
                                { move || if current_step.get() >= total_steps.get() { "Done" } else { "Mark step done" } }
                            </button>
                            <button class="btn-ghost" on:click=reset_progress>"Reset"</button>
                        </div>
                    </div>
                }.into_any(),
                None => view! {
                    <div>
                        <div class="mb-4 flex gap-3">
                            <select class="input w-48" on:change=on_topic>
                                <option value="">"All topics"</option>
                                <option value="soil">"Soil"</option>
                                <option value="composting">"Composting"</option>
                                <option value="pruning">"Pruning"</option>
                                <option value="propagation">"Propagation"</option>
                                <option value="pests">"Pests"</option>
                                <option value="seasonal">"Seasonal"</option>
                            </select>
                            <select class="input w-48" on:change=on_skill>
                                <option value="">"Any skill"</option>
                                <option value="beginner">"Beginner"</option>
                                <option value="intermediate">"Intermediate"</option>
                                <option value="advanced">"Advanced"</option>
                            </select>
                            <select class="input w-48" on:change=on_time>
                                <option value="">"Any duration"</option>
                                <option value="15">"15 min or less"</option>
                                <option value="30">"30 min or less"</option>
                                <option value="60">"60 min or less"</option>
                            </select>
                        </div>
                        <div class="grid auto-rows-min grid-cols-1 gap-3 md:grid-cols-2 lg:grid-cols-3">
                            <For each=move || guides.get() key=|g| g.id.clone() let:g>
                                <button class="card w-full p-4 text-left hover:border-garden-500/40"
                                    on:click=move |_| refresh_detail(g.id.clone())>
                                    <div class="flex items-center justify-between">
                                        <div class="font-bold text-stone-100">{g.title.clone()}</div>
                                        { if g.bookmarked { view! { <span class="text-garden-400">"*"</span> }.into_any() } else { view! { <span></span> }.into_any() } }
                                    </div>
                                    <div class="mt-1 flex flex-wrap gap-1">
                                        <span class="rounded border border-stone-800 bg-stone-900 px-1.5 py-0.5 text-[10px] text-stone-400">{topic_label(g.topic)}</span>
                                        <span class="rounded border border-stone-800 bg-stone-900 px-1.5 py-0.5 text-[10px] text-stone-400">{skill_label(g.skill_level)}</span>
                                        <span class="rounded border border-stone-800 bg-stone-900 px-1.5 py-0.5 text-[10px] text-stone-400">{format!("{} min", g.time_minutes)}</span>
                                    </div>
                                </button>
                            </For>
                        </div>
                    </div>
                }.into_any(),
            } }
        </div>
    }
}