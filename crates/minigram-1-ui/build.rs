use embedded_graphics::{pixelcolor::BinaryColor, prelude::*};
use std::{env, fs, path::Path};
use tinybmp::Bmp;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let assets_dir = Path::new("assets");

    if !assets_dir.exists() {
        return;
    }

    println!("cargo:rerun-if-changed=assets");

    let mut consts = String::new();

    for entry in fs::read_dir(assets_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "bmp") {
            let data = fs::read(&path).unwrap();
            let bmp = Bmp::<BinaryColor>::from_slice(&data).unwrap();
            let width = bmp.size().width;
            let height = bmp.size().height;

            let row_bytes = ((width + 7) / 8) as usize;
            let mut raw = vec![0u8; row_bytes * height as usize];

            for Pixel(pos, color) in bmp.pixels() {
                if color == BinaryColor::On {
                    let idx = (pos.y as usize) * row_bytes + (pos.x as usize) / 8;
                    let bit = 7 - (pos.x as u32 % 8);
                    raw[idx] |= 1 << bit;
                }
            }

            let stem = path.file_stem().unwrap().to_str().unwrap();
            let const_name = stem.to_uppercase();

            let raw_path = Path::new(&out_dir).join(format!("{stem}.raw"));
            fs::write(&raw_path, &raw).unwrap();

            consts.push_str(&format!(
                "pub const {const_name}: ::embedded_graphics::image::ImageRaw<'static, ::embedded_graphics::pixelcolor::BinaryColor> =\n    \
                 ::embedded_graphics::image::ImageRaw::new(\n        \
                 include_bytes!(concat!(env!(\"OUT_DIR\"), \"/{stem}.raw\")),\n        \
                 {width},\n    \
                 );\n\n"
            ));

            println!("cargo:rerun-if-changed={}", path.display());
        }
    }

    let generated_path = Path::new(&out_dir).join("icons.rs");
    fs::write(generated_path, consts).unwrap();
}
