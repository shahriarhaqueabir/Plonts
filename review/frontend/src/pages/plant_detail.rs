use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::plant_image::PlantImage;
use crate::ipc;
use garden_core::models::PlantDetail;

#[derive(serde::Serialize)]
struct DetailArgs {
    id: String,
}

#[component]
pub fn PlantDetailView(plant_id: Signal<String>, on_back: Callback<()>) -> impl IntoView {
    let detail = LocalResource::new(move || {
        let id = plant_id.get();
        async move {
            let args = DetailArgs { id };
            ipc::call::<PlantDetail, _>("get_plant_detail", &args).await.ok()
        }
    });

    view! {
        <div class="p-6">
            <button class="btn-ghost mb-4" on:click=move |_| on_back.run(())>"Back"</button>
            <Suspense fallback=move || view! { <p class="text-stone-400">"Loading..."</p> }>
                { move || detail.get().flatten().map(|d| view! { <DetailContent detail=d /> }) }
            </Suspense>
        </div>
    }
}

#[component]
fn DetailContent(detail: PlantDetail) -> impl IntoView {
    let companions = detail.companions.clone();
    let antagonists = detail.antagonists.clone();
    let pests = detail.common_pests.clone();
    let diseases = detail.common_diseases.clone();
    let sources = detail.sources.clone();
    let stages = detail.stages.clone();

    let image = LocalResource::new({
        let plant_id = detail.id.clone();
        move || {
            let id = plant_id.clone();
            async move { crate::ipc::get_plant_image(&id).await.ok() }
        }
    });

    let choose_photo = {
        let plant_id = detail.id.clone();
        move |_| {
            let id = plant_id.clone();
            let image = image;
            spawn_local(async move {
                if let Ok(Some(path)) = crate::ipc::pick_file().await {
                    let _ = crate::ipc::set_plant_image_override(&id, &path).await;
                    image.refetch();
                }
            });
        }
    };

    let clear_photo = {
        let plant_id = detail.id.clone();
        move |_| {
            let id = plant_id.clone();
            let image = image;
            spawn_local(async move {
                let _ = crate::ipc::clear_plant_image_override(&id).await;
                image.refetch();
            });
        }
    };

    view! {
        <div class="mx-auto max-w-3xl">
            <header class="mb-6">
                <h1 class="text-3xl font-black tracking-tight text-stone-100">{detail.name.clone()}</h1>
                <p class="text-sm italic text-stone-500">{detail.scientific_name.clone().unwrap_or_default()}</p>
                <div class="mt-3 flex flex-wrap gap-1">
                    <For each=move || detail.categories.clone() key=|c| c.clone() let:cat>
                        <span class="rounded border border-garden-700/20 bg-garden-900/20 px-2 py-0.5 text-[11px] text-garden-400">{cat}</span>
                    </For>
                    { detail.family.clone().map(|f| view! {
                        <span class="rounded border border-stone-800 bg-stone-900 px-2 py-0.5 text-[11px] text-stone-400">{f}</span>
                    }) }
                </div>
            </header>

            <Suspense fallback=move || view! {
                <div class="mb-6 h-[240px] w-full rounded-xl bg-stone-800/60"></div>
            }>
                { move || image.get().flatten().map(|img| {
                    let attribution = img.attribution.clone();
                    let has_override = img.override_url.is_some();
                    view! {
                        <figure class="mb-6">
                            <PlantImage image=img.clone() class_name="h-[240px] w-full rounded-xl object-cover" />
                            <figcaption class="mt-2 flex items-center justify-between gap-2 text-xs text-stone-500">
                                <span>
                                    { attribution.map(|a| format!("{} — {} license", a.attribution, a.license))
                                        .unwrap_or_else(|| "No image".to_string()) }
                                </span>
                                <span class="flex shrink-0 gap-2">
                                    <button class="btn-ghost px-2 py-1 text-[10px]" on:click=choose_photo.clone()>"Choose photo"</button>
                                    { if has_override {
                                        view! { <button class="btn-ghost px-2 py-1 text-[10px]" on:click=clear_photo.clone()>"Reset"</button> }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    } }
                                </span>
                            </figcaption>
                        </figure>
                    }
                }) }
            </Suspense>

            { detail.description.clone().map(|d| view! {
                <p class="mb-6 text-sm leading-6 text-stone-300">{d}</p>
            }) }

            <section class="card mb-6 p-4">
                <h2 class="mb-3 text-xs font-black uppercase tracking-widest text-stone-500">"Quick facts"</h2>
                <div class="grid grid-cols-2 gap-3 md:grid-cols-4">
                    <Fact label="Sunlight" value=detail.sunlight.clone() />
                    <Fact label="Water" value=detail.water_requirements.clone() />
                    <Fact label="Soil pH" value=detail.preferred_ph.clone() />
                    <Fact label="Lifecycle" value=detail.life_cycle.clone() />
                </div>
            </section>

            <section class="card mb-6 p-4">
                <h2 class="mb-3 text-xs font-black uppercase tracking-widest text-stone-500">"Companions"</h2>
                <ChipList items=companions.iter().map(|r| r.related_name.clone()).collect() />
            </section>

            <section class="card mb-6 p-4">
                <h2 class="mb-3 text-xs font-black uppercase tracking-widest text-stone-500">"Avoid planting near"</h2>
                <ChipList items=antagonists.iter().map(|r| r.related_name.clone()).collect() />
            </section>

            <section class="card mb-6 p-4">
                <h2 class="mb-3 text-xs font-black uppercase tracking-widest text-stone-500">"Pests & diseases"</h2>
                <ChipList items=pests.clone() />
                <ChipList items=diseases.clone() />
            </section>

            <section class="card mb-6 p-4">
                <h2 class="mb-3 text-xs font-black uppercase tracking-widest text-stone-500">"Toxicity"</h2>
                {
                    if detail.toxic_parts.is_empty() {
                        view! { <p class="text-sm text-stone-400">"No toxic parts on record."</p> }.into_any()
                    } else {
                        view! { <ChipList items=detail.toxic_parts.clone() /> }.into_any()
                    }
                }
            </section>

            <section class="card mb-6 p-4">
                <h2 class="mb-3 text-xs font-black uppercase tracking-widest text-stone-500">"Sources"</h2>
                <For each=move || sources.clone() key=|s| s.id.clone() let:src>
                    <div class="mb-1 flex items-center gap-2 text-xs text-stone-400">
                        <span>{src.name.clone()}</span>
                        { src.credibility_tier.clone().map(|t| view! {
                            <span class="rounded bg-stone-800 px-1.5 py-0.5 text-[10px] uppercase text-stone-400">{t}</span>
                        }) }
                    </div>
                </For>
            </section>

            <section class="card p-4">
                <h2 class="mb-3 text-xs font-black uppercase tracking-widest text-stone-500">"Growth stages"</h2>
                <div class="flex flex-wrap gap-2">
                    <For each=move || stages.clone() key=|s| s.id.clone() let:stage>
                        <div class="rounded-lg border border-stone-800 bg-stone-950 px-3 py-2">
                            <div class="text-xs font-bold text-stone-200">{stage.name.clone()}</div>
                            <div class="text-[10px] text-stone-500">{stage.duration_days} "d"</div>
                        </div>
                    </For>
                </div>
            </section>
        </div>
    }
}

#[component]
fn Fact(label: &'static str, value: Option<String>) -> impl IntoView {
    view! {
        <div>
            <div class="text-[10px] font-bold uppercase tracking-widest text-stone-600">{label}</div>
            <div class="text-sm text-stone-200">{value.unwrap_or_else(|| "-".to_string())}</div>
        </div>
    }
}

#[component]
fn ChipList(items: Vec<String>) -> impl IntoView {
    view! {
        <div class="flex flex-wrap gap-1">
            <For each=move || items.clone() key=|i| i.clone() let:item>
                <span class="rounded border border-stone-800 bg-stone-900 px-2 py-0.5 text-xs text-stone-300">{item}</span>
            </For>
        </div>
    }
}