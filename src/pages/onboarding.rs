use leptos::prelude::*;

#[component]
pub fn Onboarding(on_complete: impl Fn() + 'static) -> impl IntoView {
    view! { <div class="p-6">"Onboarding"</div> }
}