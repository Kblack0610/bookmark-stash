//! Reading list component

use leptos::prelude::*;
use stash_shared::Bookmark;
use std::sync::Arc;

use crate::api;

/// Reading list showing bookmarks
#[component]
pub fn ReadingList(
    filter: ReadSignal<String>,
    search_query: ReadSignal<String>,
    on_select: impl Fn(i64) + Clone + Send + Sync + 'static,
) -> impl IntoView {
    // Wrap callback in Arc for cloning
    let on_select = Arc::new(on_select);

    // Bookmarks resource that refetches when filter or search changes
    // Use LocalResource since gloo-net Response is not Send
    let bookmarks = LocalResource::new(move || {
        let filter = filter.get();
        let query = search_query.get();
        async move {
            if !query.is_empty() {
                // Search mode
                api::search(&query, Some(50), None)
                    .await
                    .map(|r| r.results.into_iter().map(|sr| sr.bookmark).collect())
                    .unwrap_or_default()
            } else {
                // Filter mode
                let (status, is_favorite) = match filter.as_str() {
                    "unread" => (Some("unread"), None),
                    "favorites" => (None, Some(true)),
                    "archived" => (Some("archived"), None),
                    _ => (Some("unread"), None),
                };
                api::list_bookmarks(status, is_favorite, None, Some(50), None)
                    .await
                    .map(|r| r.bookmarks)
                    .unwrap_or_default()
            }
        }
    });

    view! {
        <div class="reading-list">
            <Suspense fallback=move || view! { <div class="loading">"Loading..."</div> }>
                {
                    let on_select_inner = Arc::clone(&on_select);
                    move || {
                        let on_select_items = Arc::clone(&on_select_inner);
                        bookmarks.get().map(|wrapper| wrapper.take()).map(|items: Vec<Bookmark>| {
                            if items.is_empty() {
                                view! {
                                    <div class="empty-state">
                                        <p>"No bookmarks found"</p>
                                        <p class="hint">"Press Ctrl+Shift+S to save the current page"</p>
                                    </div>
                                }.into_any()
                            } else {
                                let on_select_list = Arc::clone(&on_select_items);
                                view! {
                                    <ul class="bookmark-list">
                                        {items.into_iter().map(|bookmark| {
                                            let on_select_item = Arc::clone(&on_select_list);
                                            let id = bookmark.id;
                                            view! {
                                                <BookmarkItem
                                                    bookmark=bookmark
                                                    on_click=move |_| on_select_item(id)
                                                />
                                            }
                                        }).collect::<Vec<_>>()}
                                    </ul>
                                }.into_any()
                            }
                        })
                    }
                }
            </Suspense>
        </div>
    }
}

/// Single bookmark item in the list
#[component]
fn BookmarkItem(
    bookmark: Bookmark,
    on_click: impl Fn(()) + Clone + Send + Sync + 'static,
) -> impl IntoView {
    let site_name = bookmark.site_name.clone().unwrap_or_default();
    let read_time = bookmark
        .estimated_read_time
        .map(|t| format!("{} min", t))
        .unwrap_or_default();
    let tags = bookmark.tags.clone();

    view! {
        <li class="bookmark-item" on:click=move |_| on_click(())>
            <div class="bookmark-icon">
                {if bookmark.is_favorite { "⭐" } else { "📄" }}
            </div>
            <div class="bookmark-content">
                <h3 class="bookmark-title">{bookmark.title}</h3>
                <div class="bookmark-meta">
                    <span class="site-name">{site_name}</span>
                    {(!read_time.is_empty()).then(|| view! {
                        <span class="separator">" • "</span>
                        <span class="read-time">{read_time.clone()}</span>
                    })}
                </div>
                {(!tags.is_empty()).then(|| view! {
                    <div class="bookmark-tags">
                        {tags.iter().map(|tag| {
                            let name = tag.name.clone();
                            view! { <span class="tag">"🏷️ "{name}</span> }
                        }).collect::<Vec<_>>()}
                    </div>
                })}
            </div>
        </li>
    }
}
