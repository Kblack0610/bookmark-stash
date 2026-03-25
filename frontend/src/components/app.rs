//! Root application component

use super::{QuickSave, ReadingList, SearchBar};
use leptos::prelude::*;

/// View mode for the app
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum View {
    List,
    Reader(i64),
    QuickSave,
}

/// Root application component
#[component]
pub fn App() -> impl IntoView {
    // Current view state
    let (view, set_view) = signal(View::List);

    // Search query
    let (search_query, set_search_query) = signal(String::new());

    // Current filter
    let (filter, set_filter) = signal("unread".to_string());

    // Check if we're in quick-save mode (from extension hotkey)
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(search) = window.location().search() {
                if search.contains("quicksave=") {
                    set_view.set(View::QuickSave);
                }
            }
        }
    }

    view! {
        <div class="stash-app">
            <header class="app-header">
                <h1>"🔖 Stash"</h1>
                <button class="settings-btn" title="Settings">
                    "⚙️"
                </button>
            </header>

            <SearchBar
                query=search_query
                on_search=move |q| set_search_query.set(q)
            />

            <nav class="filter-tabs">
                <button
                    class:active=move || filter.get() == "unread"
                    on:click=move |_| set_filter.set("unread".to_string())
                >
                    "📚 Unread"
                </button>
                <button
                    class:active=move || filter.get() == "favorites"
                    on:click=move |_| set_filter.set("favorites".to_string())
                >
                    "⭐ Favorites"
                </button>
                <button
                    class:active=move || filter.get() == "archived"
                    on:click=move |_| set_filter.set("archived".to_string())
                >
                    "📁 Archived"
                </button>
            </nav>

            <main class="app-content">
                {move || match view.get() {
                    View::List => view! {
                        <ReadingList
                            filter=filter
                            search_query=search_query
                            on_select=move |id| set_view.set(View::Reader(id))
                        />
                    }.into_any(),
                    View::Reader(id) => view! {
                        <super::Reader
                            bookmark_id=id
                            on_back=move |_| set_view.set(View::List)
                        />
                    }.into_any(),
                    View::QuickSave => view! {
                        <QuickSave
                            on_done=move |_| set_view.set(View::List)
                        />
                    }.into_any(),
                }}
            </main>
        </div>
    }
}
