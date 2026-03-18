//! Search bar component

use leptos::prelude::*;
use std::sync::Arc;

/// Search bar component
#[component]
pub fn SearchBar(
    query: ReadSignal<String>,
    on_search: impl Fn(String) + Clone + Send + Sync + 'static,
) -> impl IntoView {
    let (input, set_input) = signal(String::new());

    // Wrap callback in Arc for cloning
    let on_search = Arc::new(on_search);
    let on_search_input = Arc::clone(&on_search);
    let on_search_clear = Arc::clone(&on_search);

    // Sync with external query
    Effect::new(move |_| {
        set_input.set(query.get());
    });

    view! {
        <div class="search-bar">
            <input
                type="text"
                placeholder="🔍 Search..."
                prop:value=move || input.get()
                on:input=move |ev| {
                    let value = event_target_value(&ev);
                    set_input.set(value.clone());
                    on_search_input(value);
                }
            />
            {
                let on_search_btn = Arc::clone(&on_search_clear);
                move || {
                    let on_search_inner = Arc::clone(&on_search_btn);
                    (!input.get().is_empty()).then(|| view! {
                        <button
                            class="clear-btn"
                            on:click=move |_| {
                                set_input.set(String::new());
                                on_search_inner(String::new());
                            }
                        >
                            "✕"
                        </button>
                    })
                }
            }
        </div>
    }
}
