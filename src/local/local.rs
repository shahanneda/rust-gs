use std::io::Write;
use std::{fs::File, io::Read};

use final_project::data_objects::SplatData;
use final_project::loader::loader;
use final_project::log;
use final_project::scene::Scene;
use tokio;
// use serde_json;
use rkyv::{deserialize, rancor::Error, Archive, Deserialize, Serialize};

#[tokio::main]
async fn main() {
    // let scene_name = "Shahan_03_id01-30000";
    // let scene_name = "Shahan_03_id01-30000.cleaned";
    // let scene_name = "E7_01_id01-30000";
    // let scene_name = "soc_01_polycam";
    // let scene_name = "sci_01";
    // let scene_name = "soc_02_edited";
    // let scene_name = "Week-09-Sat-Nov-16-2024";
    let scene_name = "socratica_01_edited";
    // let scene_name = "Shahan_03_id01-30000";
    // let scene_name = "Shahan_03_id01-30000";
    let url = format!("http://127.0.0.1:5502/splats/{}.ply", scene_name);
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
