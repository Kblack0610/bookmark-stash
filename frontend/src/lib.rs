//! Stash Frontend - Leptos WASM app for browser extension popup

mod api;
mod components;

pub use components::App;

/// Initialize and mount the Leptos app
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
