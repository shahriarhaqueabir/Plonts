use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::ipc;
use garden_core::models::{ExperienceLevel, Hemisphere, Settings, Units};

#[component]
pub fn Onboarding(on_complete: impl Fn() + 'static) -> impl IntoView {
    let hemisphere = RwSignal::new(Hemisphere::North);
    let city = RwSignal::new(String::new());
    let experience = RwSignal::new(ExperienceLevel::Beginner);
    let units = RwSignal::new(Units::Metric);
    let saving = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);

    let on_hemisphere = move |ev: leptos::ev::Event| {
        let v = event_target_value(&ev);
        hemisphere.set(if v == "south" { Hemisphere::South } else { Hemisphere::North });
    };
    let on_experience = move |ev: leptos::ev::Event| {
        let v = event_target_value(&ev);
        experience.set(match v.as_str() {
            "learning" => ExperienceLevel::Learning,
            "confident" => ExperienceLevel::Confident,
            "expert" => ExperienceLevel::Expert,
            _ => ExperienceLevel::Beginner,
        });
    };
    let on_units = move |ev: leptos::ev::Event| {
        let v = event_target_value(&ev);
        units.set(if v == "imperial" { Units::Imperial } else { Units::Metric });
    };

    let on_complete = std::rc::Rc::new(on_complete);
    let finish = move |_| {
        saving.set(true);
        error.set(None);
        let on_complete = on_complete.clone();
        spawn_local(async move {
            match ipc::call0::<Settings>("get_settings").await {
                Ok(mut s) => {
                    s.first_load_complete = true;
                    s.hemisphere = hemisphere.get();
                    s.city = if city.get().trim().is_empty() { None } else { Some(city.get()) };
                    s.experience_level = experience.get();
                    s.units = units.get();
                    match ipc::call::<Settings, _>("update_settings", &s).await {
                        Ok(_) => on_complete(),
                        Err(e) => { error.set(Some(e)); saving.set(false); }
                    }
                }
                Err(e) => { error.set(Some(e)); saving.set(false); }
            }
        });
    };

    view! {
        <div class="flex min-h-full items-center justify-center p-8">
            <div class="w-full max-w-md card p-8">
                <h1 class="mb-1 text-2xl font-black tracking-tight text-stone-100">"Welcome to LovelyGarden"</h1>
                <p class="mb-6 text-sm text-stone-400">
                    "A local-first gardening companion. Everything stays on your device."
                </p>

                <label class="mb-1 block text-xs font-bold uppercase tracking-widest text-stone-400">"Hemisphere"</label>
                <select class="input mb-4" on:change=on_hemisphere>
                    <option value="north">"Northern"</option>
                    <option value="south">"Southern"</option>
                </select>

                <label class="mb-1 block text-xs font-bold uppercase tracking-widest text-stone-400">"Nearest city (optional)"</label>
                <input
                    type="text"
                    class="input mb-4"
                    placeholder="e.g. London"
                    prop:value=move || city.get()
                    on:input=move |ev| city.set(event_target_value(&ev))
                />

                <label class="mb-1 block text-xs font-bold uppercase tracking-widest text-stone-400">"Experience level"</label>
                <select class="input mb-4" on:change=on_experience>
                    <option value="beginner">"Beginner"</option>
                    <option value="learning">"Learning"</option>
                    <option value="confident">"Confident"</option>
                    <option value="expert">"Expert"</option>
                </select>

                <label class="mb-1 block text-xs font-bold uppercase tracking-widest text-stone-400">"Units"</label>
                <select class="input mb-6" on:change=on_units>
                    <option value="metric">"Metric"</option>
                    <option value="imperial">"Imperial"</option>
                </select>

                { move || error.get().map(|e| view! {
                    <p class="mb-4 text-xs font-bold text-red-400">{e}</p>
                }) }

                <button class="btn-primary w-full" on:click=finish disabled=move || saving.get()>
                    { move || if saving.get() { "Preparing..." } else { "Enter Garden" } }
                </button>
            </div>
        </div>
    }
}