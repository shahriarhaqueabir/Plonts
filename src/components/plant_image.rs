use leptos::prelude::*;

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
    let src = Signal::derive(move || candidates.get(idx.get()).cloned().unwrap_or_default());
    let advance = move |_| {
        idx.update(|i| {
            if *i + 1 < n {
                *i += 1;
            }
        });
    };
    let style = move || match size_px {
        Some(s) => format!("width:{s}px;height:{s}px"),
        None => String::new(),
    };

    view! {
        <img class=class_name style=move || style() src=move || src.get() on:error=advance />
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
