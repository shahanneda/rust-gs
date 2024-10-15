pub mod loader {
    use crate::log;
    use crate::ply_splat::PlySplat;
    use bytes::Bytes;
    use std::collections::HashMap;
    use std::io::{BufRead, BufReader, Cursor, Read};
    const MAX_ITERATIONS: u32 = 500;


    struct ReadHeaderResult<'a> {
        num_splats: usize,
        data_start: usize,
        property_names: Vec<String>,
        format: String,
        reader: BufReader<Cursor<&'a Bytes>>,
    }

    fn read_header(bytes: &Bytes) -> Result<ReadHeaderResult, String> {
        let mut reader = BufReader::new(Cursor::new(bytes));
        let mut current_line ;
        let mut found_end = false;
        let mut i = 0;
        let mut num_splats = 0;
        let mut property_names: Vec<String> = vec![];
        let mut format = "".to_string();
        loop {
            current_line = "".to_string();
            reader.read_line(&mut current_line).unwrap();
            // log!("current line is {}", current_line);
            if current_line.starts_with("element vertex") {
                let split = current_line.split("element vertex");
                num_splats = split.last().unwrap().trim().parse::<usize>().unwrap();
            } else if current_line.starts_with("format ") {
                let split: Vec<&str> = current_line.split(" ").collect();
                format = split.get(1).unwrap().to_string();
            } else if current_line.starts_with("property float ") {
                let split = current_line.split("property float");
                let property_name = split.last().unwrap().trim();

                property_names.push(property_name.to_string());
            } else if current_line == "end_header\n" {
                found_end = true;
                i += 1;
                break;
            }
            i += 1;

            if i > MAX_ITERATIONS {
                break;
            }
        }
        if !["binary_little_endian", "binary_big_endian"].contains(&format.as_str()) {
            return Err(format!("format not supported! {}", format));
        }

        // log!("num_splats is: {}", num_splats);
        // log!(" data starts at line: {}", i);

        if !found_end {
            return Err(String::from("test"));
        }

        return Ok(ReadHeaderResult {
            num_splats,
            data_start: i as usize,
            property_names: property_names,
            format: format,
            reader: reader,
        });
    }

    fn read_body(bytes: &Bytes, header: ReadHeaderResult) -> Vec<PlySplat> {
        let mut reader = header.reader;
        let mut current_line = String::new();

        // for i in 0..header.data_start {
        //     current_line = "".to_string();
        //     reader.read_line(&mut current_line).unwrap();
        // }

        let mut splats: Vec<PlySplat> = vec![];
        for i in (0..header.num_splats) {
            // map of prop name to value

            let mut vals: HashMap<String, f32> = HashMap::new();

            for prop_name in header.property_names.iter() {
                current_line = "".to_string();

                let mut buffer = [0; 4];
                reader.read_exact(&mut buffer).unwrap();
                if header.format == "binary_little_endian" {
                    buffer.reverse();
                }
                let float: f32 = f32::from_be_bytes(buffer);
                vals.insert(prop_name.to_string(), float);
                // log!("name is {}", prop_name);
                // log!("float is: {}", float);
            }

            splats.push(PlySplat {
                // Position coordinates
                x: *vals.get("x").unwrap(),
                y: *vals.get("y").unwrap(),
                z: *vals.get("z").unwrap(),

                // Normal vectors
                nx: *vals.get("nx").unwrap(),
                ny: *vals.get("ny").unwrap(),
                nz: *vals.get("nz").unwrap(),

                // Rotations
                rot_0: *vals.get("rot_0").unwrap(),
                rot_1: *vals.get("rot_1").unwrap(),
                rot_2: *vals.get("rot_2").unwrap(),
                rot_3: *vals.get("rot_3").unwrap(),

                // Scales
                scale_0: *vals.get("scale_0").unwrap(),
                scale_1: *vals.get("scale_1").unwrap(),
                scale_2: *vals.get("scale_2").unwrap(),

                // Opacity
                opacity: *vals.get("opacity").unwrap(),

                // f_rest fields in ascending order
                f_rest_0: *vals.get("f_rest_0").unwrap(),
                f_rest_1: *vals.get("f_rest_1").unwrap(),
                f_rest_2: *vals.get("f_rest_2").unwrap(),
                f_rest_3: *vals.get("f_rest_3").unwrap(),
                f_rest_4: *vals.get("f_rest_4").unwrap(),
                f_rest_5: *vals.get("f_rest_5").unwrap(),
                f_rest_6: *vals.get("f_rest_6").unwrap(),
                f_rest_7: *vals.get("f_rest_7").unwrap(),
                f_rest_8: *vals.get("f_rest_8").unwrap(),
                f_rest_9: *vals.get("f_rest_9").unwrap(),
                f_rest_10: *vals.get("f_rest_10").unwrap(),
                f_rest_11: *vals.get("f_rest_11").unwrap(),
                f_rest_12: *vals.get("f_rest_12").unwrap(),
                f_rest_13: *vals.get("f_rest_13").unwrap(),
                f_rest_14: *vals.get("f_rest_14").unwrap(),
                f_rest_15: *vals.get("f_rest_15").unwrap(),
                f_rest_16: *vals.get("f_rest_16").unwrap(),
                f_rest_17: *vals.get("f_rest_17").unwrap(),
                f_rest_18: *vals.get("f_rest_18").unwrap(),
                f_rest_19: *vals.get("f_rest_19").unwrap(),
                f_rest_20: *vals.get("f_rest_20").unwrap(),
                f_rest_21: *vals.get("f_rest_21").unwrap(),
                f_rest_22: *vals.get("f_rest_22").unwrap(),
                f_rest_23: *vals.get("f_rest_23").unwrap(),
                f_rest_24: *vals.get("f_rest_24").unwrap(),
                f_rest_25: *vals.get("f_rest_25").unwrap(),
                f_rest_26: *vals.get("f_rest_26").unwrap(),
                f_rest_27: *vals.get("f_rest_27").unwrap(),
                f_rest_28: *vals.get("f_rest_28").unwrap(),
                f_rest_29: *vals.get("f_rest_29").unwrap(),
                f_rest_30: *vals.get("f_rest_30").unwrap(),
                f_rest_31: *vals.get("f_rest_31").unwrap(),
                f_rest_32: *vals.get("f_rest_32").unwrap(),
                f_rest_33: *vals.get("f_rest_33").unwrap(),
                f_rest_34: *vals.get("f_rest_34").unwrap(),
                f_rest_35: *vals.get("f_rest_35").unwrap(),
                f_rest_36: *vals.get("f_rest_36").unwrap(),
                f_rest_37: *vals.get("f_rest_37").unwrap(),
                f_rest_38: *vals.get("f_rest_38").unwrap(),
                f_rest_39: *vals.get("f_rest_39").unwrap(),
                f_rest_40: *vals.get("f_rest_40").unwrap(),
                f_rest_41: *vals.get("f_rest_41").unwrap(),
                f_rest_42: *vals.get("f_rest_42").unwrap(),
                f_rest_43: *vals.get("f_rest_43").unwrap(),
                f_rest_44: *vals.get("f_rest_44").unwrap(),

                // f_dc fields in ascending order
                f_dc_0: *vals.get("f_dc_0").unwrap(),
                f_dc_1: *vals.get("f_dc_1").unwrap(),
                f_dc_2: *vals.get("f_dc_2").unwrap(),
            });
        }
        return splats;
    }

    pub async fn load_ply() -> Result<Vec<PlySplat>, String> {
        // return Ok(vec![]);
        log!("### loading ply!!!");
        // let body = reqwest::get("http://127.0.0.1:5501/splats/gaussians.ply")
        // let body = reqwest::get("http://127.0.0.1:5501/splats/Shahan_03_id01-30000.ply")
        // let body = reqwest::get("http://127.0.0.1:5501/splats/Shahan_03_id01-30000.cleaned.ply")
        // let body = reqwest::get("http://127.0.0.1:5501/splats/shahan_head.ply")
        // let body = reqwest::get("http://127.0.0.1:5501/splats/guassians.ply")
        let body = reqwest::get("http://127.0.0.1:5501/splats/corn.ply")
        // let body = reqwest::get("http://127.0.0.1:5501/splats/one-corn.ply")
            .await
            .expect("error")
            .bytes()
            .await
            .expect("went wrong when reading!");

        let header = read_header(&body).unwrap();
        let splats = read_body(&body, header);
        return Ok(splats);
    }
}
