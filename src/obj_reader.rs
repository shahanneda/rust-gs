use crate::data_objects::MeshData;
use crate::log;
use crate::timer::Timer;

pub async fn read_obj(url: &str) -> MeshData {
    // Read file content

    let _timer = Timer::new("loading obj file");
    let loaded_file = reqwest::get(url)
        .await
        .expect("error")
        .text()
        .await
        .expect("went wrong when reading!");
    let content = loaded_file.as_str();

    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut colors = Vec::new();
    let mut normals = Vec::new();

    // Parse file line by line
    for line in content.lines() {
        let mut tokens = line.split_whitespace();

        match tokens.next() {
            Some("v") => {
                // Parse vertex coordinates
                let x = tokens.next().unwrap().parse::<f32>().unwrap();
                let y = tokens.next().unwrap().parse::<f32>().unwrap();
                let z = tokens.next().unwrap().parse::<f32>().unwrap();
                vertices.push(x);
                vertices.push(y);
                vertices.push(z);

                // Add default color (white) for each vertex
                colors.push(1.0);
                colors.push(0.0);
                colors.push(0.0);
            }
            Some("f") => {
                // Parse face indices (subtract 1 as OBJ indices are 1-based)
                for idx in tokens {
                    let vertex_idx = idx.split('/').next().unwrap().parse::<u32>().unwrap() - 1;
                    indices.push(vertex_idx);
                }
            }
            Some("vn") => {
                let x = tokens.next().unwrap().parse::<f32>().unwrap();
                let y = tokens.next().unwrap().parse::<f32>().unwrap();
                let z = tokens.next().unwrap().parse::<f32>().unwrap();
                normals.push(x);
                normals.push(y);
                normals.push(z);
            }
            _ => {} // Ignore other lines
        }
    }
    log!("vertices: {}", vertices.len());
    log!("indices: {}", indices.len());
    log!("colors: {}", colors.len());

    MeshData::new(vertices, indices, colors, normals)
}
