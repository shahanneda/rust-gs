//! Local converter binary.
//!
//! Usage:
//!   cargo run --release --bin local -- <input> [more inputs...]
//!
//! Each input may be a `.ply` or `.rkyv` file on disk. For every input a
//! packed `.gsz` file (~26 bytes/splat) is written next to it, mirrored under
//! `splats/v2/` when the input lives inside `splats/`.

use std::fs;
use std::path::{Path, PathBuf};

use gs_rust::data_objects::SplatData;
use gs_rust::loader::loader;
use gs_rust::packed_format;

fn output_path(input: &Path) -> PathBuf {
    let file_name = input.with_extension("gsz");
    // Mirror splats/foo.rkyv -> splats/v2/foo.gsz so local dev matches the
    // s3://zimpmodels/splats/v2/ layout.
    let s = file_name.to_string_lossy().to_string();
    if let Some(idx) = s.find("splats/") {
        let (prefix, rest) = s.split_at(idx + "splats/".len());
        return PathBuf::from(format!("{}v2/{}", prefix, rest));
    }
    file_name
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("usage: local <input.ply|input.rkyv> [more inputs...]");
        std::process::exit(1);
    }

    for arg in &args {
        let input = Path::new(arg);
        println!("converting {}", input.display());
        let bytes = fs::read(input).expect("failed to read input file");

        let splat_data = if arg.ends_with(".ply") {
            let ply_splats = loader::parse_ply(&bytes.into()).expect("failed to parse ply");
            SplatData::new(ply_splats)
        } else {
            SplatData::new_from_bytes(&bytes)
        };
        println!("  {} splats", splat_data.splats.len());

        let packed = packed_format::encode(&splat_data.splats);
        let out = output_path(input);
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent).expect("failed to create output dir");
        }
        fs::write(&out, &packed).expect("failed to write output");
        println!(
            "  wrote {} ({:.1} MB, was {:.1} MB)",
            out.display(),
            packed.len() as f64 / 1e6,
            fs::metadata(input).map(|m| m.len()).unwrap_or(0) as f64 / 1e6
        );
    }
}
