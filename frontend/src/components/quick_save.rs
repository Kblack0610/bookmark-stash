//! Quick save component - shown when saving via hotkey

use leptos::prelude::*;
use std::sync::Arc;

/// Quick save confirmation component
#[component]
pub fn QuickSave(on_done: impl Fn(()) + Clone + Send + Sync + 'static) -> impl IntoView {
    let (status, set_status) = signal("saving".to_string());
    let (url, _set_url) = signal(String::new());
    let (title, _set_title) = signal(String::new());
    let (error, _set_error) = signal(Option::<String>::None);

    // Wrap callback in Arc for cloning
    let on_done = Arc::new(on_done);

    // Get URL from query params and save
    #[cfg(target_arch = "wasm32")]
    {
        use crate::api;

        Effect::new(move |_| {
            if let Some(window) = web_sys::window() {
                if let Ok(search) = window.location().search() {
                    if let Some(param) = search.strip_prefix("?quicksave=") {
                        // Convert to owned String before async block to satisfy 'static lifetime
                        let url_to_save =
                            urlencoding::decode(param).unwrap_or_default().into_owned();
                        _set_url.set(url_to_save.clone());

                        // Save the bookmark
                        wasm_bindgen_futures::spawn_local(async move {
                            match api::create_bookmark(&url_to_save, None).await {
                                Ok(bookmark) => {
                                    _set_title.set(bookmark.title);
                                    set_status.set("saved".to_string());
                                }
                                Err(e) => {
                                    _set_error.set(Some(e.message));
                                    set_status.set("error".to_string());
                                }
                            }
                        });
                    }
                }
            }
        });
    }

    let on_done_view = Arc::clone(&on_done);

    view! {
        <div class="quick-save">
            {
                let on_done_inner = Arc::clone(&on_done_view);
                move || {
                    let on_done_btn = Arc::clone(&on_done_inner);
                    match status.get().as_str() {
                        "saving" => view! {
                            <div class="saving-indicator">
                                <span class="spinner">"⏳"</span>
                                <p>"Saving..."</p>
                                <p class="url">{url.get()}</p>
                            </div>
                        }.into_any(),
                        "saved" => {
                            let on_done_click = Arc::clone(&on_done_btn);
                            view! {
                                <div class="saved-indicator">
                                    <span class="checkmark">"✅"</span>
                                    <p>"Saved!"</p>
                                    <h3>{title.get()}</h3>
                                    <button class="done-btn" on:click=move |_| on_done_click(())>
                                        "View Reading List"
                                    </button>
                                </div>
                            }.into_any()
                        },
                        "error" => view! {
                            <div class="error-indicator">
                                <span class="error-icon">"❌"</span>
                                <p>"Failed to save"</p>
                                <p class="error-message">{error.get()}</p>
                                <button class="retry-btn" on:click=move |_| {
                                    set_status.set("saving".to_string());
                                    // Retry logic would go here
                                }>
                                    "Retry"
                                </button>
                            </div>
                        }.into_any(),
                        _ => view! { <div></div> }.into_any(),
                    }
                }
            }
        </div>
    }
}
