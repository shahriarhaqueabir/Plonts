use leptos::prelude::*;
use crate::image_cache::use_image_cache;

use garden_core::models::PlantImage;

/// Renders a resolved plant image with a client-side fallback chain:
/// override -> species -> category -> generic. If `size_px` is set the image is
/// square; otherwise pass `class_name` (e.g. a full-width/height utility).
#[component]
pub fn PlantImage(
    image: PlantImage,
    #[prop(optional)] size_px: Option<u32>,
    #[prop(default = "shrink-0 rounded-md object-cover")] class_name: &'static str,
) -> impl IntoView {
    let candidates: Vec<String> = [
        image.override_url,
        image.species_url,
        image.category_url,
        Some(image.generic_url),
    ]
    .into_iter()
    .flatten()
    .collect();

    let idx = RwSignal::new(0usize);
    let n = candidates.len();
    let exhausted = RwSignal::new(n == 0);
    let src = Signal::derive(move || candidates.get(idx.get()).cloned().unwrap_or_default());
    let advance = move |_| {
        idx.update(|i| {
            if *i + 1 < n {
                *i += 1;
            } else {
                exhausted.set(true);
            }
        });
    };
    let style = move || match size_px {
        Some(s) => format!("width:{s}px;height:{s}px"),
        None => String::new(),
    };

    view! {
        <Show when=move || !exhausted.get() fallback=move || view! {
            <div class=class_name style=style>
                <span class="flex h-full w-full items-center justify-center text-stone-500">"🌿"</span>
            </div>
        }>
            <img class=class_name style=style src=move || src.get() on:error=advance />
        </Show>
    }
}

/// Fetches and renders a plant image for a catalog_id at a fixed square size.
/// Wraps the fetch in its own LocalResource + Suspense.
#[component]
pub fn PlantThumbnail(catalog_id: String, size_px: u32) -> impl IntoView {
    let image = LocalResource::new(move || {
        let id = catalog_id.clone();
        let cache = use_image_cache();
        async move {
            if let Some(cached) = cache.get(&id) {
                return Some(cached);
            }
            let res = crate::ipc::get_plant_image(&id).await.ok();
            if let Some(img) = &res {
                cache.set(id.clone(), img.clone());
            }
            res
        }
    });

    view! {
        <Suspense fallback=move || view! {
            <div class="shrink-0 animate-pulse rounded-md bg-stone-800" style=move || format!("width:{size_px}px;height:{size_px}px")></div>
        }>
            { move || image.get().flatten().map(|img| view! {
                <PlantImage image=img size_px=size_px />
            }) }
        </Suspense>
    }
}

/// Generic leaf illustration for empty states and onboarding.
#[component]
pub fn GenericLeaf(#[prop(default = 96)] size_px: u32) -> impl IntoView {
    let url = LocalResource::new(move || async move { crate::ipc::get_generic_image().await.ok() });

    view! {
        <Suspense fallback=move || view! {
            <div class="rounded-full bg-stone-800" style=move || format!("width:{size_px}px;height:{size_px}px")></div>
        }>
            { move || url.get().flatten().map(|u| view! {
                <img src=u class="rounded-full" style=move || format!("width:{size_px}px;height:{size_px}px") />
            }) }
        </Suspense>
    }
}
