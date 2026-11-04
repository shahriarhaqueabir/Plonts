use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::plant_card::PlantCard;
use crate::ipc;
use crate::pages::plant_detail::PlantDetailView;
use garden_core::models::{SearchFilters, SearchResult};

#[derive(serde::Serialize)]
struct SearchArgs {
    query: String,
    filters: Option<SearchFilters>,
    limit: Option<usize>,
}

#[component]
pub fn PlantKbPage() -> impl IntoView {
    let query = RwSignal::new(String::new());
    let category = RwSignal::new(None::<String>);
    let results = RwSignal::new(Vec::<SearchResult>::new());
    let categories = RwSignal::new(Vec::<String>::new());
    let selected = RwSignal::new(None::<String>);
    let detail_id = Signal::derive(move || selected.get().clone().unwrap_or_default());

    let run = move || {
        let q = query.get();
        let cat = category.get();
        let results = results;
        spawn_local(async move {
            let filters = cat.map(|c| SearchFilters {
                categories: Some(vec![c]),
                ..Default::default()
            });
            let args = SearchArgs { query: q, filters, limit: Some(200) };
            let r = ipc::call::<Vec<SearchResult>, _>("search_plants", &args)
                .await
                .unwrap_or_default();
            results.set(r);
        });
    };

    spawn_local(async move {
        if let Ok(cats) = ipc::call0::<Vec<String>>("list_categories").await {
            categories.set(cats);
        }
        run();
    });

    let on_input = move |ev: leptos::ev::Event| {
        query.set(event_target_value(&ev));
        run();
    };
    let on_category = move |ev: leptos::ev::Event| {
        let v = event_target_value(&ev);
        category.set(if v.is_empty() { None } else { Some(v) });
        run();
    };

    view! {
        <div class="flex h-full flex-col p-6">
            { move || match selected.get() {
                Some(_) => view! {
                    <PlantDetailView
                        plant_id=detail_id
                        on_back=Callback::new(move |_| selected.set(None))
                    />
                }.into_any(),
                None => view! {
                    <div class="mb-4">
                        <h1 class="mb-4 text-xl font-bold text-stone-100">"Plant Knowledge Base"</h1>
                        <div class="flex gap-3">
                            <input
                                type="text"
                                class="input flex-1"
                                placeholder="Search by name, trait, companion, pest..."
                                prop:value=move || query.get()
                                on:input=on_input
                            />
                            <select class="input w-48" on:change=on_category>
                                <option value="">"All categories"</option>
                                <For each=move || categories.get() key=|c| c.clone() let:cat>
                                    <option value=cat.clone()>{cat.clone()}</option>
                                </For>
                            </select>
                        </div>
                    </div>
                    <div class="grid flex-1 auto-rows-min grid-cols-1 gap-3 overflow-auto md:grid-cols-2 lg:grid-cols-3">
                        <For each=move || results.get() key=|r| r.plant.id.clone() let:item>
                            <PlantCard plant=item.plant.clone() on_select=Callback::new(move |id| selected.set(Some(id))) />
                        </For>
                    </div>
                }.into_any(),
            } }
        </div>
    }
}
