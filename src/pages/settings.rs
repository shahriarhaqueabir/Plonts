use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::ipc;
use garden_core::models::{ExperienceLevel, Hemisphere, Settings, Theme, Units};

#[derive(serde::Serialize)]
struct UpdateSettingsArgs { settings: Settings }

#[component]
pub fn SettingsPage() -> impl IntoView {
    let settings = RwSignal::new(None::<Settings>);
    let hemisphere = RwSignal::new(Hemisphere::North);
    let city = RwSignal::new(String::new());
    let experience = RwSignal::new(ExperienceLevel::Beginner);
    let units = RwSignal::new(Units::Metric);
    let theme = RwSignal::new(Theme::Dark);
    let saved = RwSignal::new(false);
    let credits = LocalResource::new(move || async move { crate::ipc::list_credits().await.ok() });
    spawn_local(async move {
        if let Ok(s) = ipc::call0::<Settings>("get_settings").await {
            hemisphere.set(s.hemisphere);
            city.set(s.city.clone().unwrap_or_default());
            experience.set(s.experience_level);
            units.set(s.units);
            theme.set(s.theme);
            settings.set(Some(s));
        }
    });

    let save = move |_| {
        let base = settings.get().unwrap_or_default();
        let updated = Settings {
            id: base.id,
            first_load_complete: base.first_load_complete,
            hemisphere: hemisphere.get(),
            city: if city.get().trim().is_empty() { None } else { Some(city.get()) },
            experience_level: experience.get(),
            units: units.get(),
            theme: theme.get(),
            data_version: base.data_version,
        };
        spawn_local(async move {
            let args = UpdateSettingsArgs { settings: updated };
            match ipc::call::<Settings, _>("update_settings", &args).await {
                Ok(s) => { settings.set(Some(s)); saved.set(true); }
                Err(_) => {}
            }
        });
    };

    let on_hemisphere = move |ev: leptos::ev::Event| {
        hemisphere.set(if event_target_value(&ev) == "south" { Hemisphere::South } else { Hemisphere::North });
    };
    let on_experience = move |ev: leptos::ev::Event| {
        experience.set(match event_target_value(&ev).as_str() {
            "learning" => ExperienceLevel::Learning,
            "confident" => ExperienceLevel::Confident,
            "expert" => ExperienceLevel::Expert,
            _ => ExperienceLevel::Beginner,
        });
    };
    let on_units = move |ev: leptos::ev::Event| {
        units.set(if event_target_value(&ev) == "imperial" { Units::Imperial } else { Units::Metric });
    };
    let on_theme = move |ev: leptos::ev::Event| {
        theme.set(if event_target_value(&ev) == "light" { Theme::Light } else { Theme::Dark });
    };

    view! {
        <div class="p-6">
            <h1 class="mb-6 text-xl font-bold text-stone-100">"Settings"</h1>
            <div class="max-w-md space-y-5">
                <div>
                    <label class="mb-1 block text-xs font-bold uppercase tracking-widest text-stone-400">"Theme"</label>
                    <select class="input" on:change=on_theme>
                        <option value="dark" selected=move || theme.get() == Theme::Dark>"Dark"</option>
                        <option value="light" selected=move || theme.get() == Theme::Light>"Light"</option>
                    </select>
                </div>
                <div>
                    <label class="mb-1 block text-xs font-bold uppercase tracking-widest text-stone-400">"Hemisphere"</label>
                    <select class="input" on:change=on_hemisphere>
                        <option value="north" selected=move || hemisphere.get() == Hemisphere::North>"Northern"</option>
                        <option value="south" selected=move || hemisphere.get() == Hemisphere::South>"Southern"</option>
                    </select>
                </div>
                <div>
                    <label class="mb-1 block text-xs font-bold uppercase tracking-widest text-stone-400">"Nearest city"</label>
                    <input type="text" class="input" placeholder="e.g. London"
                        prop:value=move || city.get()
                        on:input=move |ev| city.set(event_target_value(&ev)) />
                </div>
                <div>
                    <label class="mb-1 block text-xs font-bold uppercase tracking-widest text-stone-400">"Experience level"</label>
                    <select class="input" on:change=on_experience>
                        <option value="beginner" selected=move || experience.get() == ExperienceLevel::Beginner>"Beginner"</option>
                        <option value="learning" selected=move || experience.get() == ExperienceLevel::Learning>"Learning"</option>
                        <option value="confident" selected=move || experience.get() == ExperienceLevel::Confident>"Confident"</option>
                        <option value="expert" selected=move || experience.get() == ExperienceLevel::Expert>"Expert"</option>
                    </select>
                </div>
                <div>
                    <label class="mb-1 block text-xs font-bold uppercase tracking-widest text-stone-400">"Units"</label>
                    <select class="input" on:change=on_units>
                        <option value="metric" selected=move || units.get() == Units::Metric>"Metric"</option>
                        <option value="imperial" selected=move || units.get() == Units::Imperial>"Imperial"</option>
                    </select>
                </div>
                <div class="flex items-center gap-3">
                    <button class="btn-primary" on:click=save>"Save"</button>
                    { move || if saved.get() {
                        view! { <span class="text-xs font-bold text-garden-400">"Saved!"</span> }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    } }
                </div>
            </div>
                <section class="card mt-8 max-w-2xl p-4">
                    <h2 class="mb-3 text-xs font-black uppercase tracking-widest text-stone-500">"Credits & Licenses"</h2>
                    <Suspense fallback=move || view! { <p class="text-sm text-stone-500">"Loading..."</p> }>
                        { move || credits.get().flatten().map(|list| view! {
                            <ul class="space-y-2">
                                <For each=move || list.clone() key=|c| c.source.clone() let:c>
                                    <li class="text-sm text-stone-400">
                                        <span class="font-bold text-stone-200">{c.source.clone()}</span>
                                        <span class="text-stone-500">" — " {c.license.clone()}</span>
                                        { c.note.clone().map(|n| view! { <span class="text-stone-500">" (" {n} ")"</span> }) }
                                        { c.url.clone().map(|u| view! {
                                            <a class="ml-1 text-garden-400 hover:underline" href=u target="_blank">"source"</a>
                                        }) }
                                    </li>
                                </For>
                            </ul>
                        }) }
                    </Suspense>
                </section>
        </div>
    }
}
