use leptos::prelude::*;

/// Global toast message signal. Provide in AppShell.
#[component]
pub fn ToastHost() -> impl IntoView {
    let msg = use_context::<RwSignal<Option<String>>>().unwrap_or_else(|| RwSignal::new(None));
    view! {
        <Show when=move || msg.get().is_some()>
            <div class="fixed bottom-4 right-4 z-50 max-w-sm rounded-xl border border-rose-700 bg-rose-950/90 px-4 py-3 text-sm text-rose-100 shadow-lg">
                {move || msg.get().unwrap_or_default()}
                <button class="ml-3 text-rose-300 hover:text-white" on:click=move |_| msg.set(None)>"✕"</button>
            </div>
        </Show>
    }
}

pub fn show_toast(msg: RwSignal<Option<String>>, text: impl Into<String>) {
    msg.set(Some(text.into()));
}