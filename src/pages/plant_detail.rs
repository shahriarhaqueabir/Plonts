use leptos::prelude::*;

#[component]
pub fn PlantDetailView(plant_id: String, on_back: Callback<()>) -> impl IntoView {
    view! {
        <div class="p-6">
            <button class="btn-ghost mb-4" on:click=move |_| on_back.run(())>"Back"</button>
            <div>{plant_id}</div>
        </div>
    }
}
