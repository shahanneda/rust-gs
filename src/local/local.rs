use std::io::Write;
use std::{fs::File, io::Read};

use gs_rust::data_objects::SplatData;
use gs_rust::loader::loader;
use gs_rust::log;
use gs_rust::scene::Scene;
use tokio;
// use serde_json;
use rkyv::{deserialize, rancor::Error, Archive, Deserialize, Serialize};

#[tokio::main]
async fn main() {
    // let scene_name = "ninja/cake_rotate";
    // let scene_name = "ninja/cake_rotate";
    // let scene_name = "ninja/cake_rotate";
    // let scene_name = "ninja/cake_rotate";
    // let scene_name = "ninja/cake_rotate";
    // let scene_name = "ninja/cake_rotate";
    // let scene_name = "ninja/cake_rotate";
    // let scene_name = "ninja/cake_rotate";
    // let scene_name = "ninja/cake_rotate";
    // let scene_name = "ninja/cake_rotate";
    // let scene_name = "ninja/cake_rotate";
    // let scene_name = "ninja/cake_rotate";
    // let scene_name = "ninja/cake_rotate";
    // let scene_name = "ninja/cake_rotate";
    // let scene_name = "ninja/cake_rotate";
    // let scene_name = "ninja/cake_rotate";
    // let scene_name = "ninja/cake_rotate";
    let scene_name = "ninja/cake_rotate";
    // let scene_name = "ninja/cake_rotate";
    // let scene_name = "ninja/cake_rotate";
    // let scene_name = "ninja/cake_rotate";
    // let scene_name = "ninja/cake_rotate";

    // let scene_name = "ninja/cake_rotate";
    // let scene_name = "ninja/cake_rotate";
    let url = format!("http://127.0.0.1:5503/splats/{}.ply", scene_name);
    println!("Compressing ply file: {}", url);

    let mut ply_splat = loader::load_ply(&url)
        .await
        .expect("something went wrong in loading");
    // ply_splat.truncate(2);
    let mut splat = SplatData::new(ply_splat);
    // let serialized = serde_json::to_string(&scene).unwrap();
    let serialized = rkyv::to_bytes::<Error>(&splat).unwrap();
    let mut file =
        File::create(format!("splats/{}.rkyv", scene_name)).expect("Unable to create file");
    file.write_all(&serialized).expect("Unable to write data");
}
