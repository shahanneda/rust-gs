#[allow(dead_code)]
pub mod scene;
pub mod loader;
pub mod gui;
pub mod ply_splat;
pub mod scene_geo;
pub mod timer;
pub mod shared_utils;

#[cfg(target_arch = "wasm32")]
pub mod web;
#[cfg(target_arch = "wasm32")]
pub mod utils;
#[cfg(target_arch = "wasm32")]
pub mod shader;
#[cfg(target_arch = "wasm32")]
pub mod camera;
#[cfg(target_arch = "wasm32")]
pub mod renderer;