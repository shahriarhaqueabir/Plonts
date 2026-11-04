use leptos::prelude::*;

use garden_core::models::PlantSummary;

#[component]
pub fn PlantCard(plant: PlantSummary, on_select: Callback<String>) -> impl IntoView {
    let PlantSummary {
        id,
        name,
        scientific_name,
        categories,
        ..
    } = plant;
    let click_id = id.clone();

    view! {
        <button
            class="card w-full p-4 text-left transition-colors hover:border-garden-500/40"
            on:click=move |_| on_select.run(click_id.clone())
        >
            <div class="font-bold text-stone-100">{name}</div>
            <div class="text-xs italic text-stone-500">{scientific_name.unwrap_or_default()}</div>
            <div class="mt-2 flex flex-wrap gap-1">
                <For each=move || categories.clone() key=|c| c.clone() let:cat>
                    <span class="rounded border border-stone-800 bg-stone-900 px-1.5 py-0.5 text-[10px] text-stone-400">
                        {cat}
                    </span>
                </For>
            </div>
        </button>
    }
}
