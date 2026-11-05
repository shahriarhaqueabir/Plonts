use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::ipc;
use garden_core::llm::{
    ChatConversation, ChatMessage, ContextFlags, MessageRole, ModelInfo, ModelKind,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Chat,
    Models,
}

fn human_size(bytes: i64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

fn mime_from_path(p: &str) -> Option<String> {
    let lower = p.to_lowercase();
    if lower.ends_with(".png") {
        Some("image/png".to_string())
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        Some("image/jpeg".to_string())
    } else if lower.ends_with(".webp") {
        Some("image/webp".to_string())
    } else if lower.ends_with(".gif") {
        Some("image/gif".to_string())
    } else {
        None
    }
}

fn prompt_note() -> Option<String> {
    let val = js_sys::eval("window.prompt('Flag note (optional):')").ok()?;
    if val.is_null() || val.is_undefined() {
        return None;
    }
    let s = val.as_string()?;
    if s.trim().is_empty() {
        None
    } else {
        Some(s)
    }
}

#[component]
pub fn CoachPage() -> impl IntoView {
    let mode = RwSignal::new(Mode::Chat);

    // Chat state
    let conversations = RwSignal::new(Vec::<ChatConversation>::new());
    let selected_conv = RwSignal::new(None::<String>);
    let messages = RwSignal::new(Vec::<ChatMessage>::new());
    let flags = RwSignal::new(ContextFlags::default());
    let input = RwSignal::new(String::new());
    let attachment_path = RwSignal::new(None::<String>);
    let attachment_mime = RwSignal::new(None::<String>);
    let sending = RwSignal::new(false);
    let last_speed = RwSignal::new(None::<f64>);
    let chat_error = RwSignal::new(None::<String>);

    // Models state
    let models = RwSignal::new(Vec::<ModelInfo>::new());
    let model_status = RwSignal::new(None::<ModelInfo>);
    let import_name = RwSignal::new(String::new());
    let import_kind = RwSignal::new(ModelKind::Text);
    let import_path = RwSignal::new(None::<String>);
    let models_error = RwSignal::new(None::<String>);

    // Reload helpers
    let reload_conversations = move || {
        spawn_local(async move {
            if let Ok(cs) = ipc::list_conversations().await {
                conversations.set(cs);
            }
        });
    };
    let reload_messages = move |conv_id: String| {
        spawn_local(async move {
            if let Ok(ms) = ipc::list_messages(&conv_id).await {
                messages.set(ms);
            }
        });
    };
    let reload_models = move || {
        spawn_local(async move {
            if let Ok(ms) = ipc::list_models().await {
                models.set(ms);
            }
        });
    };
    let reload_status = move || {
        spawn_local(async move {
            if let Ok(s) = ipc::get_model_status().await {
                model_status.set(s);
            }
        });
    };

    // Initial load
    spawn_local(async move {
        if let Ok(cs) = ipc::list_conversations().await {
            conversations.set(cs);
        }
        if let Ok(ms) = ipc::list_models().await {
            models.set(ms);
        }
        if let Ok(s) = ipc::get_model_status().await {
            model_status.set(s);
        }
    });

    let select_conv = move |id: String| {
        selected_conv.set(Some(id.clone()));
        // set flags from conversation list
        if let Some(c) = conversations.get().into_iter().find(|c| c.id == id) {
            flags.set(c.context_flags);
        }
        reload_messages(id);
    };

    let new_conv = move |_| {
        spawn_local(async move {
            let c = ipc::create_conversation("New chat", &ContextFlags::default()).await;
            if let Ok(c) = c {
                conversations.update(|v| v.insert(0, c.clone()));
                selected_conv.set(Some(c.id.clone()));
                flags.set(c.context_flags);
                messages.set(Vec::new());
            }
        });
    };

    let delete_conv = move |id: String| {
        spawn_local(async move {
            let _ = ipc::delete_conversation(&id).await;
            if selected_conv.get().as_deref() == Some(id.as_str()) {
                selected_conv.set(None);
                messages.set(Vec::new());
            }
            reload_conversations();
        });
    };

    let clear_conv = move |id: String| {
        spawn_local(async move {
            let _ = ipc::clear_conversation(&id).await;
            if selected_conv.get().as_deref() == Some(id.as_str()) {
                messages.set(Vec::new());
            }
        });
    };

    let clear_all = move |_| {
        spawn_local(async move {
            let _ = ipc::clear_all_conversations().await;
            selected_conv.set(None);
            messages.set(Vec::new());
            reload_conversations();
        });
    };

    let toggle_flag = move |which: &'static str, val: bool| {
        let mut f = flags.get();
        match which {
            "garden" => f.garden = val,
            "zone" => f.zone = val,
            "tasks" => f.tasks = val,
            _ => {}
        }
        flags.set(f.clone());
        if let Some(id) = selected_conv.get() {
            let f2 = f;
            spawn_local(async move {
                let _ = ipc::update_conversation_flags(&id, &f2).await;
            });
        }
    };

    let attach_photo = move |_| {
        spawn_local(async move {
            if let Ok(Some(path)) = ipc::pick_file().await {
                let m = mime_from_path(&path);
                attachment_path.set(Some(path));
                attachment_mime.set(m);
            }
        });
    };

    let send = move |_| {
        let content = input.get();
        if content.trim().is_empty() {
            return;
        }
        let flags_val = flags.get();
        let conv_opt = selected_conv.get();
        let attach = attachment_path.get();
        let mime = attachment_mime.get();
        spawn_local(async move {
            let mut conv_id = conv_opt;
            if conv_id.is_none() {
                let title: String = content.chars().take(40).collect();
                let title = if title.trim().is_empty() {
                    "New chat".to_string()
                } else {
                    title
                };
                match ipc::create_conversation(&title, &flags_val).await {
                    Ok(c) => {
                        conv_id = Some(c.id.clone());
                        selected_conv.set(Some(c.id.clone()));
                        flags.set(c.context_flags.clone());
                        conversations.update(|v| v.insert(0, c));
                    }
                    Err(e) => {
                        chat_error.set(Some(e));
                        return;
                    }
                }
            }
            let Some(conv_id) = conv_id else { return; };
            sending.set(true);
            chat_error.set(None);
            match ipc::generate_chat(&conv_id, &content, &flags_val, attach.as_deref(), mime.as_deref())
                .await
            {
                Ok(resp) => {
                    input.set(String::new());
                    attachment_path.set(None);
                    attachment_mime.set(None);
                    last_speed.set(Some(resp.compl_tok_per_sec));
                    if let Ok(ms) = ipc::list_messages(&conv_id).await {
                        messages.set(ms);
                    }
                    if let Ok(cs) = ipc::list_conversations().await {
                        conversations.set(cs);
                    }
                    let _ = resp;
                }
                Err(e) => {
                    chat_error.set(Some(e));
                }
            }
            sending.set(false);
        });
    };

    let flag_msg = move |id: String| {
        spawn_local(async move {
            let note = prompt_note();
            let _ = ipc::flag_message(&id, note.as_deref()).await;
            if let Some(conv) = selected_conv.get() {
                reload_messages(conv);
            }
        });
    };

    let tasks_from = move |text: String| {
        spawn_local(async move {
            let _ = ipc::create_tasks_from_text(&text).await;
        });
    };
    let journal_from = move |text: String| {
        spawn_local(async move {
            let _ = ipc::create_journal_from_text(&text).await;
        });
    };
    let layout_from = move |text: String| {
        spawn_local(async move {
            let _ = ipc::create_layout_from_text(&text).await;
        });
    };

    // Models actions
    let do_import = move |_| {
        let name = import_name.get();
        let kind = import_kind.get();
        let path = import_path.get();
        let Some(path) = path else {
            models_error.set(Some("Choose a GGUF file first".to_string()));
            return;
        };
        let name = if name.trim().is_empty() {
            path.split(['/', '\\']).last().unwrap_or("model").to_string()
        } else {
            name
        };
        spawn_local(async move {
            match ipc::import_model(&path, &name, kind).await {
                Ok(_) => {
                    import_name.set(String::new());
                    import_path.set(None);
                    reload_models();
                }
                Err(e) => models_error.set(Some(e)),
            }
        });
    };

    let do_choose_gguf = move |_| {
        spawn_local(async move {
            if let Ok(Some(p)) = ipc::pick_gguf().await {
                import_path.set(Some(p));
            }
        });
    };

    let do_load = move |id: String| {
        spawn_local(async move {
            match ipc::load_model(&id).await {
                Ok(_) => {
                    reload_status();
                    reload_models();
                }
                Err(e) => models_error.set(Some(e)),
            }
        });
    };
    let do_unload = move |_| {
        spawn_local(async move {
            let _ = ipc::unload_model().await;
            reload_status();
        });
    };
    let do_delete_model = move |id: String| {
        spawn_local(async move {
            match ipc::delete_model(&id).await {
                Ok(_) => {
                    reload_models();
                    reload_status();
                }
                Err(e) => models_error.set(Some(e)),
            }
        });
    };

    view! {
        <div class="p-6">
            <h1 class="mb-1 text-xl font-bold text-stone-100">"Coach"</h1>
            <p class="mb-4 text-sm text-stone-400">"Local LLM assistant grounded in your library."</p>

            <div class="mb-4 flex gap-3">
                <button class=move || if mode.get() == Mode::Chat { "btn-primary" } else { "btn-ghost" }
                    on:click=move |_| mode.set(Mode::Chat)>"Chat"</button>
                <button class=move || if mode.get() == Mode::Models { "btn-primary" } else { "btn-ghost" }
                    on:click=move |_| mode.set(Mode::Models)>"Models"</button>
            </div>

            { move || if mode.get() == Mode::Chat {
                view! {
                    <div class="grid grid-cols-1 gap-4 lg:grid-cols-[260px_1fr]">
                        // Left: conversations
                        <aside class="card p-3">
                            <div class="mb-3 flex items-center justify-between">
                                <h2 class="text-xs font-black uppercase tracking-widest text-stone-500">"Conversations"</h2>
                                <button class="btn-primary text-[10px]" on:click=new_conv>"New"</button>
                            </div>
                            <div class="mb-3 space-y-1 max-h-[60vh] overflow-auto">
                                <For each=move || conversations.get() key=|c| c.id.clone() let:c>
                                    {
                                        let c_sel = c.id.clone();
                                        let c_del = c.id.clone();
                                        let c_clr = c.id.clone();
                                        let is_active = move || selected_conv.get().as_deref() == Some(c.id.as_str());
                                        view! {
                                            <div class=move || if is_active() { "rounded-lg border border-garden-500/40 bg-garden-900/20 p-2" } else { "rounded-lg border border-stone-800 p-2" }>
                                                <button class="block w-full text-left text-sm text-stone-200 truncate"
                                                    on:click=move |_| select_conv(c_sel.clone())>
                                                    {c.title.clone()}
                                                </button>
                                                <div class="mt-1 flex gap-2">
                                                    <button class="btn-ghost text-[9px]" on:click=move |_| clear_conv(c_clr.clone())>"Clear"</button>
                                                    <button class="btn-ghost text-[9px]" on:click=move |_| delete_conv(c_del.clone())>"Delete"</button>
                                                </div>
                                            </div>
                                        }
                                    }
                                </For>
                                { move || if conversations.get().is_empty() {
                                    view! { <p class="text-xs text-stone-500">"No conversations yet."</p> }.into_any()
                                } else { view! { <span></span> }.into_any() } }
                            </div>
                            <button class="btn-ghost w-full text-[10px]" on:click=clear_all>"Clear all"</button>
                        </aside>

                        // Right: messages + composer
                        <section class="flex flex-col gap-3">
                            <div class="card p-4 max-h-[55vh] overflow-auto space-y-3">
                                <For each=move || messages.get() key=|m| m.id.clone() let:m>
                                    {
                                        let is_user = m.role == MessageRole::User;
                                        let msg_content = m.content.clone();
                                        let msg_sources = m.sources.clone();
                                        let msg_grounded = m.grounded;
                                        let msg_id = m.id.clone();
                                        let msg_text = m.content.clone();
                                        let attachments = m.attachments.clone();
                                        view! {
                                            <div class=move || if is_user { "ml-auto max-w-[80%] rounded-xl bg-garden-900/30 p-3" } else { "max-w-[85%] rounded-xl border border-stone-800 bg-stone-900 p-3" }>
                                                <div class="mb-1 flex items-center gap-2">
                                                    { if is_user {
                                                        view! { <span class="text-[10px] font-bold text-garden-300">"You"</span> }.into_any()
                                                    } else if msg_grounded {
                                                        view! { <span class="rounded bg-garden-500/20 px-1.5 py-0.5 text-[10px] font-bold text-garden-300">"from library"</span> }.into_any()
                                                    } else {
                                                        view! { <span class="rounded bg-stone-700 px-1.5 py-0.5 text-[10px] font-bold text-stone-300">"model-generated"</span> }.into_any()
                                                    } }
                                                </div>
                                                <p class="whitespace-pre-wrap text-sm text-stone-200">{msg_content}</p>
                                                { if !attachments.is_empty() {
                                                    view! {
                                                        <div class="mt-2 flex flex-wrap gap-2">
                                                            <For each=move || attachments.clone() key=|a| a.id.clone() let:a>
                                                                <img src=a.url.clone() class="h-20 w-20 rounded-lg object-cover" />
                                                            </For>
                                                        </div>
                                                    }.into_any()
                                                } else { view! { <span></span> }.into_any() } }
                                                { if !is_user && !msg_sources.is_empty() {
                                                    view! {
                                                        <div class="mt-2 space-y-1">
                                                            <p class="text-[10px] font-bold uppercase tracking-widest text-stone-500">"Sources"</p>
                                                            <For each=move || msg_sources.clone() key=|s| s.id.clone() let:s>
                                                                <div class="rounded border border-stone-800 bg-stone-950 px-2 py-1 text-[11px] text-stone-300">
                                                                    <span class="font-bold text-stone-200">{s.title.clone()}</span>
                                                                    <span class="ml-2 text-stone-500">{s.kind.clone()}</span>
                                                                </div>
                                                            </For>
                                                        </div>
                                                    }.into_any()
                                                } else { view! { <span></span> }.into_any() } }
                                                { if !is_user {
                                                    let t1 = msg_text.clone();
                                                    let t2 = msg_text.clone();
                                                    let t3 = msg_text.clone();
                                                    let fid = msg_id.clone();
                                                    view! {
                                                        <div class="mt-2 flex flex-wrap gap-2">
                                                            <button class="btn-ghost text-[9px]" on:click=move |_| tasks_from(t1.clone())>"Tasks"</button>
                                                            <button class="btn-ghost text-[9px]" on:click=move |_| journal_from(t2.clone())>"Journal"</button>
                                                            <button class="btn-ghost text-[9px]" on:click=move |_| layout_from(t3.clone())>"Layout"</button>
                                                            <button class="btn-ghost text-[9px]" on:click=move |_| flag_msg(fid.clone())>"Flag"</button>
                                                        </div>
                                                    }.into_any()
                                                } else { view! { <span></span> }.into_any() } }
                                            </div>
                                        }
                                    }
                                </For>
                                { move || if messages.get().is_empty() {
                                    view! { <p class="text-sm text-stone-500">"No messages yet. Send a question below."</p> }.into_any()
                                } else { view! { <span></span> }.into_any() } }
                            </div>

                            // Composer
                            <div class="card p-4 space-y-3">
                                <div class="flex flex-wrap items-center gap-3">
                                    <label class="flex items-center gap-1 text-xs text-stone-300">
                                        <input type="checkbox" prop:checked=move || flags.get().garden
                                            on:change=move |ev| toggle_flag("garden", event_target_checked(&ev)) />
                                        "Garden"
                                    </label>
                                    <label class="flex items-center gap-1 text-xs text-stone-300">
                                        <input type="checkbox" prop:checked=move || flags.get().zone
                                            on:change=move |ev| toggle_flag("zone", event_target_checked(&ev)) />
                                        "Zone"
                                    </label>
                                    <label class="flex items-center gap-1 text-xs text-stone-300">
                                        <input type="checkbox" prop:checked=move || flags.get().tasks
                                            on:change=move |ev| toggle_flag("tasks", event_target_checked(&ev)) />
                                        "Tasks"
                                    </label>
                                    { move || model_status.get().map(|m| view! {
                                        <span class="ml-auto text-[10px] text-stone-500">{format!("Loaded: {}", m.name)}</span>
                                    }) }
                                    { move || if model_status.get().is_none() {
                                        view! { <span class="ml-auto text-[10px] text-amber-400">"No model loaded"</span> }.into_any()
                                    } else { view! { <span></span> }.into_any() } }
                                </div>
                                { move || attachment_path.get().map(|p| view! {
                                    <div class="flex items-center gap-2 text-xs text-stone-400">
                                        <span class="truncate">{p}</span>
                                        <button class="btn-ghost text-[9px]" on:click=move |_| { attachment_path.set(None); attachment_mime.set(None); }>"x"</button>
                                    </div>
                                }) }
                                { move || last_speed.get().map(|s| view! {
                                    <p class="text-[10px] text-stone-500">{format!("~{:.1} tok/s", s)}</p>
                                }) }
                                { move || chat_error.get().map(|e| view! {
                                    <p class="text-xs text-red-400">{e}</p>
                                }) }
                                <div class="flex gap-2">
                                    <input type="text" class="input flex-1" placeholder="Ask the coach..."
                                        prop:value=move || input.get()
                                        on:input=move |ev| input.set(event_target_value(&ev)) />
                                    <button class="btn-ghost" on:click=attach_photo disabled=move || sending.get()>"Attach"</button>
                                    <button class="btn-primary" on:click=send.clone()
                                        disabled=move || sending.get() || model_status.get().is_none() || input.get().trim().is_empty()>
                                        { move || if sending.get() { "Sending..." } else { "Send" } }
                                    </button>
                                </div>
                                { move || if model_status.get().is_none() {
                                    view! { <p class="text-[10px] text-amber-400">"Import and load a model in the Models tab to chat."</p> }.into_any()
                                } else { view! { <span></span> }.into_any() } }
                            </div>
                        </section>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="space-y-4 max-w-3xl">
                        // Status
                        <div class="card p-4">
                            <h2 class="mb-2 text-xs font-black uppercase tracking-widest text-stone-500">"Loaded model"</h2>
                            { move || match model_status.get() {
                                Some(m) => view! {
                                    <div class="flex items-center justify-between">
                                        <div>
                                            <p class="text-sm font-bold text-stone-200">{m.name.clone()}</p>
                                            <p class="text-[10px] text-stone-500">{format!("{} · {}", human_size(m.size_bytes), match m.kind { ModelKind::Text => "text", ModelKind::Vision => "vision" })}</p>
                                        </div>
                                        <button class="btn-ghost" on:click=do_unload.clone()>"Unload"</button>
                                    </div>
                                }.into_any(),
                                None => view! { <p class="text-sm text-stone-500">"No model loaded - import one to use the Coach."</p> }.into_any(),
                            } }
                        </div>

                        // Import card
                        <div class="card p-4 space-y-3">
                            <h2 class="mb-2 text-xs font-black uppercase tracking-widest text-stone-500">"Import model"</h2>
                            <div class="flex gap-2">
                                <input type="text" class="input flex-1" placeholder="Name"
                                    prop:value=move || import_name.get()
                                    on:input=move |ev| import_name.set(event_target_value(&ev)) />
                                <select class="input w-32" on:change=move |ev| {
                                    import_kind.set(if event_target_value(&ev) == "vision" { ModelKind::Vision } else { ModelKind::Text });
                                }>
                                    <option value="text" selected=move || import_kind.get() == ModelKind::Text>"Text"</option>
                                    <option value="vision" selected=move || import_kind.get() == ModelKind::Vision>"Vision"</option>
                                </select>
                            </div>
                            <div class="flex items-center gap-2">
                                <button class="btn-ghost" on:click=do_choose_gguf>"Choose GGUF"</button>
                                { move || import_path.get().map(|p| view! {
                                    <span class="truncate text-xs text-stone-400">{p}</span>
                                }) }
                            </div>
                            <button class="btn-primary" on:click=do_import>"Import"</button>
                            { move || models_error.get().map(|e| view! {
                                <p class="text-xs text-red-400">{e}</p>
                            }) }
                        </div>

                        // Model list
                        <div class="card p-4 space-y-2">
                            <h2 class="mb-2 text-xs font-black uppercase tracking-widest text-stone-500">"Registered models"</h2>
                            <For each=move || models.get() key=|m| m.id.clone() let:m>
                                {
                                    let m_load = m.id.clone();
                                    let m_delete = m.id.clone();
                                    let m_id_for_loaded = m.id.clone();
                                    let is_loaded = move || model_status.get().as_ref().map(|s| s.id.clone()) == Some(m_id_for_loaded.clone());
                                    view! {
                                        <div class="flex items-center justify-between rounded-lg border border-stone-800 p-3">
                                            <div>
                                                <p class="text-sm font-bold text-stone-200">{m.name.clone()}</p>
                                                <p class="text-[10px] text-stone-500">
                                                    {format!("{} · {} · last used: {}",
                                                        human_size(m.size_bytes),
                                                        match m.kind { ModelKind::Text => "text", ModelKind::Vision => "vision" },
                                                        m.last_used_at.clone().unwrap_or_else(|| "never".to_string()))}
                                                </p>
                                            </div>
                                            <div class="flex gap-2">
                                                { if is_loaded() {
                                                    view! { <span class="text-[10px] text-garden-400">"Loaded"</span> }.into_any()
                                                } else {
                                                    let id = m_load.clone();
                                                    view! { <button class="btn-ghost text-[10px]" on:click=move |_| do_load(id.clone())>"Load"</button> }.into_any()
                                                } }
                                                <button class="btn-ghost text-[10px]" on:click=move |_| do_delete_model(m_delete.clone())>"Delete"</button>
                                            </div>
                                        </div>
                                    }
                                }
                            </For>
                            { move || if models.get().is_empty() {
                                view! { <p class="text-sm text-stone-500">"No models registered yet."</p> }.into_any()
                            } else { view! { <span></span> }.into_any() } }
                        </div>
                    </div>
                }.into_any()
            } }
        </div>
    }
}

// helper to read checkbox checked state
fn event_target_checked(ev: &leptos::ev::Event) -> bool {
    use wasm_bindgen::JsCast;
    let target = ev.target().unwrap();
    let input: web_sys::HtmlInputElement = target.dyn_into().unwrap();
    input.checked()
}