pub mod gui;
pub mod loader;
pub mod ply_splat;
pub mod DataObjects;
#[allow(dead_code)]
pub mod scene;
pub mod scene_geo;
pub mod scene_object;
pub mod shared_utils;
pub mod splat;
pub mod timer;

#[cfg(target_arch = "wasm32")]
pub mod camera;
#[cfg(target_arch = "wasm32")]
pub mod renderer;
#[cfg(target_arch = "wasm32")]
pub mod shader;
#[cfg(target_arch = "wasm32")]
pub mod utils;
#[cfg(target_arch = "wasm32")]
pub mod web;
