use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::image_cache::provide_image_cache;
use crate::ipc;
use crate::pages;
use garden_core::models::Settings;

#[derive(serde::Serialize)]
struct UpdateSettingsArgs { settings: Settings }

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum Tab {
    Today,
    Garden,
    Plants,
    Journal,
    Coach,
    // secondary
    Layout,
    SowingCalendar,
    Guides,
    Problems,
    SeedsHarvest,
    Settings,
}

#[component]
pub fn App() -> impl IntoView {
    provide_image_cache();
    let settings = RwSignal::new(None::<Settings>);
    let tab = RwSignal::new(Tab::Today);

    spawn_local(async move {
        let s = ipc::call0::<Settings>("get_settings").await.ok();
        settings.set(s);
    });

    view! {
        <main class=move || {
            let dark = settings.get().map(|s| s.theme == garden_core::models::Theme::Dark).unwrap_or(true);
            if dark { "h-screen w-screen bg-[#161A14] text-stone-100 dark" } else { "h-screen w-screen bg-[#F6F1E7] text-[#2B2A25]" }
        }>
            { move || match settings.get() {
                None => view! { <p class="p-8 text-stone-400">"Loading..."</p> }.into_any(),
                Some(s) if !s.first_load_complete => view! {
                    <pages::Onboarding on_complete=move || {
                        spawn_local(async move {
                            if let Ok(mut s) = ipc::call0::<Settings>("get_settings").await {
                                s.first_load_complete = true;
                                let _ = ipc::call::<Settings, _>("update_settings", &UpdateSettingsArgs { settings: s.clone() }).await;
                                settings.set(Some(s));
                            }
                        });
                    } />
                }.into_any(),
                Some(_) => view! { <AppShell tab /> }.into_any(),
            } }
        </main>
    }
}

#[component]
fn AppShell(tab: RwSignal<Tab>) -> impl IntoView {
    provide_context(RwSignal::new(None::<String>));
    let primary: Vec<(Tab, &'static str)> = vec![
        (Tab::Today, "Today"),
        (Tab::Garden, "Garden"),
        (Tab::Plants, "Plants"),
        (Tab::Journal, "Journal"),
        (Tab::Coach, "Coach"),
    ];
    let secondary: Vec<(Tab, &'static str)> = vec![
        (Tab::Layout, "Layout"),
        (Tab::SowingCalendar, "Sowing Calendar"),
        (Tab::Guides, "Guides"),
        (Tab::Problems, "Problems"),
        (Tab::SeedsHarvest, "Seeds & Harvest"),
    ];

    view! {
        <div class="flex h-full">
            <nav class="flex w-56 shrink-0 flex-col gap-1 border-r border-stone-300/40 bg-white/40 p-4 backdrop-blur dark:border-stone-800 dark:bg-[#1F261C]/60">
                <h1 class="mb-4 px-2 font-serif text-xl font-bold tracking-tight text-[#2F4A2A] dark:text-[#86efac]">
                    "GardenOS"
                </h1>
                <For each=move || primary.clone() key=|(t, _)| *t let:item>
                    {
                        let (t, label) = item;
                        let active = move || tab.get() == t;
                        view! {
                            <button
                                class=move || {
                                    if active() {
                                        "rounded-xl bg-garden-500/20 px-3 py-2 text-left text-sm font-bold text-garden-300"
                                    } else {
                                        "rounded-xl px-3 py-2 text-left text-sm font-bold text-stone-300 hover:bg-stone-900"
                                    }
                                }
                                on:click=move |_| tab.set(t)
                            >
                                {label}
                            </button>
                        }
                    }
                </For>
                <p class="mt-4 px-2 text-[10px] font-black uppercase tracking-widest text-stone-600">"More"</p>
                <For each=move || secondary.clone() key=|(t, _)| *t let:item>
                    {
                        let (t, label) = item;
                        let active = move || tab.get() == t;
                        view! {
                            <button
                                class=move || {
                                    if active() {
                                        "rounded-xl bg-stone-800 px-3 py-2 text-left text-xs font-bold text-stone-200"
                                    } else {
                                        "rounded-xl px-3 py-2 text-left text-xs font-bold text-stone-500 hover:bg-stone-900"
                                    }
                                }
                                on:click=move |_| tab.set(t)
                            >
                                {label}
                            </button>
                        }
                    }
                </For>
                <div class="mt-auto">
                    <button
                        class=move || {
                            if tab.get() == Tab::Settings {
                                "w-full rounded-xl bg-stone-800 px-3 py-2 text-left text-xs font-bold text-stone-200"
                            } else {
                                "w-full rounded-xl px-3 py-2 text-left text-xs font-bold text-stone-500 hover:bg-stone-900"
                            }
                        }
                        on:click=move |_| tab.set(Tab::Settings)
                    >
                        "Settings"
                    </button>
                </div>
            </nav>
            <section class="flex-1 overflow-auto bg-[#F6F1E7] dark:bg-[#161A14]">
                { move || match tab.get() {
                    Tab::Today => view! { <pages::TodayPage /> }.into_any(),
                    Tab::Garden => view! { <pages::VirtualGardenPage /> }.into_any(),
                    Tab::Plants => view! { <pages::PlantKbPage /> }.into_any(),
                    Tab::Layout => view! { <pages::LayoutPage /> }.into_any(),
                    Tab::SowingCalendar => view! { <pages::SowingCalendarPage /> }.into_any(),
                    Tab::Guides => view! { <pages::GuidesPage /> }.into_any(),
                    Tab::Journal => view! { <pages::JournalPage /> }.into_any(),
                    Tab::Coach => view! { <pages::CoachPage /> }.into_any(),
                    Tab::Problems => view! { <pages::ProblemsPage /> }.into_any(),
                    Tab::SeedsHarvest => view! { <pages::SeedsHarvestPage /> }.into_any(),
                    Tab::Settings => view! { <pages::SettingsPage /> }.into_any(),
                } }
            </section>
            <crate::components::toast::ToastHost />
        </div>
    }
}