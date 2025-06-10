#[cfg(not(target_arch = "wasm32"))]
mod local;
#[cfg(target_arch = "wasm32")]
mod web;
