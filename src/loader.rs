pub mod loader {

    use crate::log;
    use js_sys::EvalError;
    use ply_rs::ply::{self, Property};
    use ply_rs::parser::Parser;
    use crate::ply_splat::PlySplat;




    fn get_float_from_property(p: &Property) -> Result<f32, &'static str> {
        match p {
            Property::Float(val) => return Ok(val.clone()),
            _ => return Err("Failed to parse float from property"),
        }
    }

    pub async fn load_ply() -> Result<Vec<PlySplat>, EvalError> {
        // return Ok(vec![]);
        // let body = reqwest::get("http://127.0.0.1:5500/splats/test.txt")
        // let body = reqwest::get("http://127.0.0.1:5500/splats/Shahan_03_id01-30000.ply")
        // let body = reqwest::get("http://127.0.0.1:5501/splats/corn.ply")
        let body = reqwest::get("http://127.0.0.1:5501/splats/one-corn.ply")
            .await
            .expect("error")
            .bytes()
            .await
            .expect("went wrong when reading!");

        log!("after load!");
        // for byte in body.iter().take(100) {
        //     log!("byte: {:?}", byte);
        // }

        // .text()
        // .await.expect("error 2");

        // log!("body = {:?}", body);
        use bytes::Buf;
        use std::io::Read;

        // let mut buf = Bytes::from("Hello world");
        let mut reader: Box<dyn Read> = Box::new(body.reader()) as Box<dyn Read>;

        let p = Parser::<ply::DefaultElement>::new();
        let ply = p.read_ply(&mut reader);

        assert!(ply.is_ok());
        let ply = ply.expect("failed to parse ply!");

        // log!("Ply header: {:#?}", ply.header);
        // log!("Number of splats: {:#?}", ply.payload["vertex"].len());
        // log!("x: {:#?}", ply.payload["vertex"][0]["x"]);
        // log!("y: {:#?}", ply.payload["vertex"][0]["y"]);
        // log!("z: {:#?}", ply.payload["vertex"][0]["rot_2"]);

        let vertex_vals = &ply.payload["vertex"];
	
	macro_rules! create_splat {
	($val:expr, $($field:ident),+) => {
		PlySplat {
		$(
			$field: get_float_from_property(&$val[stringify!($field)]).unwrap(),
		)+
		}
	};
	}

    let splat = vertex_vals.iter().enumerate().map(|(i, splat_ply)| {
        return create_splat!(splat_ply, x, y, z, nx, ny, nz, opacity, rot_0, rot_1, rot_2, rot_3, scale_0, scale_1, scale_2, f_dc_0, f_dc_1, f_dc_2, f_rest_0, f_rest_1, f_rest_2, f_rest_3, f_rest_4, f_rest_5, f_rest_6, f_rest_7, f_rest_8, f_rest_9, f_rest_10, f_rest_11, f_rest_12, f_rest_13, f_rest_14, f_rest_15, f_rest_16, f_rest_17, f_rest_18, f_rest_19, f_rest_20, f_rest_21, f_rest_22, f_rest_23, f_rest_24, f_rest_25, f_rest_26, f_rest_27, f_rest_28, f_rest_29, f_rest_30, f_rest_31, f_rest_32, f_rest_33, f_rest_34, f_rest_35, f_rest_36, f_rest_37, f_rest_38, f_rest_39, f_rest_40, f_rest_41, f_rest_42, f_rest_43, f_rest_44);
    }).collect();
    return Ok(splat);

    }
}
