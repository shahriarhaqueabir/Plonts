use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::ipc;
use crate::pages;
use garden_core::models::Settings;

#[derive(serde::Serialize)]
struct UpdateSettingsArgs { settings: Settings }

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum Tab {
    VirtualGarden,
    Layout,
    PlantKb,
    SowingCalendar,
    Guides,
    Journal,
    Problems,
    SeedsHarvest,
    Settings,
}

#[component]
pub fn App() -> impl IntoView {
    let settings = RwSignal::new(None::<Settings>);
    let tab = RwSignal::new(Tab::PlantKb);

    spawn_local(async move {
        let s = ipc::call0::<Settings>("get_settings").await.ok();
        settings.set(s);
    });

    view! {
        <main class="h-screen w-screen bg-stone-950 text-stone-100">
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
    let tabs: Vec<(Tab, &'static str)> = vec![
        (Tab::VirtualGarden, "Virtual Garden"),
        (Tab::Layout, "Layout"),
        (Tab::PlantKb, "Plant KB"),
        (Tab::SowingCalendar, "Sowing Calendar"),
        (Tab::Guides, "Guides"),
        (Tab::Journal, "Journal"),
        (Tab::Problems, "Problems"),
        (Tab::SeedsHarvest, "Seeds & Harvest"),
        (Tab::Settings, "Settings"),
    ];

    view! {
        <div class="flex h-full">
            <nav class="flex w-56 shrink-0 flex-col gap-1 border-r border-stone-800 p-4">
                <h1 class="mb-4 px-2 text-sm font-black uppercase tracking-widest text-garden-400">
                    "LovelyGarden"
                </h1>
                <For each=move || tabs.clone() key=|(t, _)| *t let:item>
                    {
                        let (t, label) = item;
                        let active = move || tab.get() == t;
                        view! {
                            <button
                                class=move || {
                                    if active() {
                                        "rounded-xl bg-garden-500/20 px-3 py-2 text-left text-xs font-bold text-garden-300"
                                    } else {
                                        "rounded-xl px-3 py-2 text-left text-xs font-bold text-stone-400 hover:bg-stone-900"
                                    }
                                }
                                on:click=move |_| tab.set(t)
                            >
                                {label}
                            </button>
                        }
                    }
                </For>
            </nav>
            <section class="flex-1 overflow-auto">
                { move || match tab.get() {
                    Tab::VirtualGarden => view! { <pages::VirtualGardenPage /> }.into_any(),
                    Tab::Layout => view! { <pages::LayoutPage /> }.into_any(),
                    Tab::PlantKb => view! { <pages::PlantKbPage /> }.into_any(),
                    Tab::SowingCalendar => view! { <pages::SowingCalendarPage /> }.into_any(),
                    Tab::Guides => view! { <pages::GuidesPage /> }.into_any(),
                    Tab::Journal => view! { <pages::JournalPage /> }.into_any(),
                    Tab::Problems => view! { <pages::ProblemsPage /> }.into_any(),
                    Tab::SeedsHarvest => view! { <pages::SeedsHarvestPage /> }.into_any(),
                    Tab::Settings => view! { <pages::SettingsPage /> }.into_any(),
                } }
            </section>
        </div>
    }
}