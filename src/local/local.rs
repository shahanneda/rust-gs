use std::{fs::File, io::Read};
use std::io::Write;

use gs_rust::log;
use gs_rust::scene::Scene;
use gs_rust::loader::loader;
use tokio;
// use serde_json;
use rkyv::{deserialize, rancor::Error, Archive, Deserialize, Serialize};


#[tokio::main]
async fn main() {
	// let scene_name = "Shahan_03_id01-30000";
	// let scene_name = "shahan_head";
	// let scene_name = "Shahan_03_id01-30000";
	// let scene_name = "Shahan_03_id01-30000";
	let url = format!("http://127.0.0.1:5501/splats/{}.ply", scene_name);
	println!("Compressing ply file: {}", url);

	let ply_splat = loader::load_ply(&url).await.expect("something went wrong in loading");
	let mut scene = Scene::new(ply_splat);
	// let serialized = serde_json::to_string(&scene).unwrap();
	let serialized = rkyv::to_bytes::<Error>(&scene).unwrap();
	let mut file = File::create(format!("splats/{}.rkyv", scene_name)).expect("Unable to create file");
	file.write_all(&serialized).expect("Unable to write data");
}
