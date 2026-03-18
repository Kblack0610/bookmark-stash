//! Reader view component for reading articles

use leptos::prelude::*;
use std::sync::Arc;

use crate::api;

/// Reader view for displaying article content
#[component]
pub fn Reader(
    bookmark_id: i64,
    on_back: impl Fn(()) + Clone + Send + Sync + 'static,
) -> impl IntoView {
    // Wrap callback in Arc for cloning
    let on_back = Arc::new(on_back);

    // Fetch bookmark details using LocalResource (gloo-net Response is not Send)
    let bookmark =
        LocalResource::new(move || async move { api::get_bookmark(bookmark_id).await.ok() });

    // Fetch content separately (can be large)
    let content = LocalResource::new(move || async move {
        api::get_bookmark_content(bookmark_id).await.ok().flatten()
    });

    view! {
        <div class="reader-view">
            <header class="reader-header">
                <button class="back-btn" on:click=move |_| on_back(())>
                    "← Back"
                </button>
                <div class="reader-actions">
                    <button class="action-btn" title="Toggle favorite">"⭐"</button>
                    <button class="action-btn" title="Archive">"📁"</button>
                    <button class="action-btn" title="Open original">"🔗"</button>
                </div>
            </header>

            <Suspense fallback=move || view! { <div class="loading">"Loading..."</div> }>
                {move || {
                    bookmark.get().map(|w| w.take()).flatten().map(|bm| {
                        let url = bm.url.clone();
                        let title = bm.title.clone();
                        let site_name = bm.site_name.clone();
                        let read_time = bm.estimated_read_time;
                        view! {
                            <article class="reader-article">
                                <h1 class="article-title">{title}</h1>
                                <div class="article-meta">
                                    {site_name.map(|s| view! {
                                        <span class="site-name">{s}</span>
                                    })}
                                    {read_time.map(|t| view! {
                                        <span class="read-time">{format!("{} min read", t)}</span>
                                    })}
                                </div>
                                <a href={url} target="_blank" class="original-link">
                                    "View original →"
                                </a>

                                <Suspense fallback=move || view! { <div class="loading">"Loading content..."</div> }>
                                    {move || {
                                        match content.get().map(|w| w.take()).flatten() {
                                            Some(html) => {
                                                view! {
                                                    <div class="article-content" inner_html={html}></div>
                                                }.into_any()
                                            }
                                            None => {
                                                view! {
                                                    <div class="no-content">
                                                        <p>"No content available for this bookmark."</p>
                                                        <p>"Click 'View original' to read on the source website."</p>
                                                    </div>
                                                }.into_any()
                                            }
                                        }
                                    }}
                                </Suspense>
                            </article>
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}
