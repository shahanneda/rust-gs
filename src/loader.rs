pub mod loader {
    use crate::log;
    use js_sys::EvalError;
    use std::num;
    use std::thread::current;
    use std::time::Instant;   
    use crate::ply_splat::PlySplat;
    use web_sys::console;
    use bytes::Buf;
    use bytes::Bytes;
    use std::io::{BufRead, BufReader, Cursor, Read};
    const MAX_ITERATIONS: u32 = 500;




    // fn get_float_from_property(p: &Property) -> Result<f32, &'static str> {
    //     match p {
    //         Property::Float(val) => return Ok(val.clone()),
    //         _ => return Err("Failed to parse float from property"),
    //     }
    // }

    struct ReadHeaderResult{
        num_splats: usize,
        data_start: usize,
        property_names: Vec<String>,
        format: String,
    }

    pub fn read_header(bytes : &Bytes) -> Result<ReadHeaderResult, String>{
        let mut reader = BufReader::new(Cursor::new(bytes));
        let mut current_line = String::new();
        let mut found_end = false;
        let mut i = 0;
        let mut num_splats = 0;
        let mut property_names: Vec<String> = vec![];
        let mut format = "".to_string();
        loop{
            current_line = "".to_string();
            reader.read_line(&mut current_line).unwrap();
            log!("current line is {}", current_line);
            if current_line.starts_with("element vertex") {
                let split = current_line.split("element vertex");
                num_splats = split.last().unwrap().trim().parse::<usize>().unwrap();
            }else if current_line.starts_with("format ") {
                let split = current_line.split("format ");
                format = split.last().unwrap().trim().to_string();
            }else if current_line.starts_with("property float ") {
                let split = current_line.split("property float");
                let property_name = split.last().unwrap().trim();

                property_names.push(property_name.to_string());

            }else if current_line == "end_header\n"{
                found_end = true;
                i += 1;
                break;

            }
            i += 1;

            if i > MAX_ITERATIONS {
                break;
            }
        }

        log!("num_splats is: {}", num_splats);
        log!(" data starts at line: {}", i);

        if !found_end {
            return Err(String::from("test"));
        }

        return Ok(ReadHeaderResult{num_splats, data_start: i as usize, property_names: property_names, format: format});
    }

    pub fn read_body(bytes: &Bytes, header: ReadHeaderResult){
        let mut reader = BufReader::new(Cursor::new(bytes));
        let mut current_line = String::new();
        for i in 0..header.data_start{
            current_line = "".to_string();
            reader.read_line(&mut current_line).unwrap();
        }

        for i in 0..header.num_splats{
            current_line = "".to_string();

            let mut buffer = [0; 4];
            reader.read_exact(&mut buffer).unwrap();
            log!("current line is {:?}", buffer);
            // TODO: Detect if in little endian and switch to big endian!
            let float: f32 = f32::from_be_bytes(buffer);
            log!("float is: {}", float);
        }
    }


    pub async fn load_ply() -> Result<Vec<PlySplat>, String> {
        // return Ok(vec![]);
        // let body = reqwest::get("http://127.0.0.1:5500/splats/test.txt")
        // let body = reqwest::get("http://127.0.0.1:5500/splats/Shahan_03_id01-30000.ply")
        // let body = reqwest::get("http://127.0.0.1:5501/splats/Shahan_03_id01-30000.cleaned.ply")
        // let body = reqwest::get("http://127.0.0.1:5501/splats/corn.ply")
        let body = reqwest::get("http://127.0.0.1:5501/splats/one-corn.ply")
            .await
            .expect("error")
            .bytes()
            .await
            .expect("went wrong when reading!");

    let header = read_header(&body).unwrap();
    read_body(&body, header);
        // let mut reader: Box<dyn Read> = Box::new(body.reader()) as Box<dyn Read>;

    // let splat = vertex_vals.iter().enumerate().map(|(i, splat_ply)| {
    //     return create_splat!(splat_ply, x, y, z, nx, ny, nz, opacity, rot_0, rot_1, rot_2, rot_3, scale_0, scale_1, scale_2, f_dc_0, f_dc_1, f_dc_2, f_rest_0, f_rest_1, f_rest_2, f_rest_3, f_rest_4, f_rest_5, f_rest_6, f_rest_7, f_rest_8, f_rest_9, f_rest_10, f_rest_11, f_rest_12, f_rest_13, f_rest_14, f_rest_15, f_rest_16, f_rest_17, f_rest_18, f_rest_19, f_rest_20, f_rest_21, f_rest_22, f_rest_23, f_rest_24, f_rest_25, f_rest_26, f_rest_27, f_rest_28, f_rest_29, f_rest_30, f_rest_31, f_rest_32, f_rest_33, f_rest_34, f_rest_35, f_rest_36, f_rest_37, f_rest_38, f_rest_39, f_rest_40, f_rest_41, f_rest_42, f_rest_43, f_rest_44);
    // }).collect();
    // return Ok(splat);
    return Err(String::from("failed to parse!"));

    }
}
