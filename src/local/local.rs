use std::fs::File;
use std::io::Write;

use gs_rust::log;
use gs_rust::scene::Scene;
use gs_rust::loader::loader;
use tokio;
use serde_json;

#[tokio::main]
async fn main() {
	let scene_name = "corn";
	let url = format!("http://127.0.0.1:5501/splats/{}.ply", scene_name);
	println!("Compressing ply file: {}", url);

	let ply_splat = loader::load_ply(&url).await.expect("something went wrong in loading");
	let mut scene = Scene::new(ply_splat);
	let serialized = serde_json::to_string(&scene).unwrap();
	let mut file = File::create(format!("splats/{}.json", scene_name)).expect("Unable to create file");
	file.write_all(serialized.as_bytes()).expect("Unable to write data");
}
