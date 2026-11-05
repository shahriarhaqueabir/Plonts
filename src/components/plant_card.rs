use leptos::prelude::*;

use crate::components::plant_image::PlantImage;
use garden_core::models::PlantSummary;

#[component]
pub fn PlantCard(plant: PlantSummary, on_select: Callback<String>) -> impl IntoView {
    let PlantSummary { id, name, scientific_name, categories, .. } = plant;
    let click_id = id.clone();
    let image = LocalResource::new(move || {
        let id = id.clone();
        async move { crate::ipc::get_plant_image(&id).await.ok() }
    });

    view! {
        <button
            class="card flex w-full items-start gap-3 p-4 text-left transition-colors hover:border-garden-500/40"
            on:click=move |_| on_select.run(click_id.clone())
        >
            <Suspense fallback=move || view! {
                <div class="shrink-0 rounded-md bg-stone-800" style="width:48px;height:48px"></div>
            }>
                { move || image.get().flatten().map(|img| view! {
                    <PlantImage image=img size_px=48 />
                }) }
            </Suspense>
            <div class="min-w-0 flex-1">
                <div class="font-bold text-stone-100">{name}</div>
                <div class="text-xs italic text-stone-500">{scientific_name.unwrap_or_default()}</div>
                <div class="mt-2 flex flex-wrap gap-1">
                    <For each=move || categories.clone() key=|c| c.clone() let:cat>
                        <span class="rounded border border-stone-800 bg-stone-900 px-1.5 py-0.5 text-[10px] text-stone-400">
                            {cat}
                        </span>
                    </For>
                </div>
            </div>
        </button>
    }
}
