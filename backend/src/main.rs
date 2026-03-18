//! Stash Backend - REST API server for the read-it-later system

mod api;
mod bookmark_import;
mod content;
mod db;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::db::Database;

/// Application state shared across handlers
pub struct AppState {
    pub db: Arc<Mutex<Database>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "stash_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Get config from environment
    let db_path = std::env::var("STASH_DB_PATH").unwrap_or_else(|_| {
        let home = std::env::var("HOME").expect("HOME not set");
        format!("{}/.stash/stash.db", home)
    });
    let port: u16 = std::env::var("STASH_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3030);

    // Ensure database directory exists
    if let Some(parent) = std::path::Path::new(&db_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Initialize database
    tracing::info!("Opening database at {}", db_path);
    let db = Database::open(&db_path)?;
    db.migrate()?;

    let state = Arc::new(AppState {
        db: Arc::new(Mutex::new(db)),
    });

    // Build router
    let app = Router::new()
        .nest("/api", api::router())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::info!("Stash backend listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
