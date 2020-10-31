use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();

    for entry in fs::read_dir("frames").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        let file_name = path.file_name().unwrap().to_str().unwrap();

        if path.is_file() {
            let dest_path = Path::new(&out_dir).join(file_name).with_extension("raw");

            //panic!("{:?}", dest_path);
            let _ = std::fs::create_dir_all(dest_path.parent().unwrap());

            Command::new("convert")
                .arg(&path)
                .args(&[
                    "-background",
                    "black",
                    "-filter",
                    "Box",
                    "-define",
                    "filter:blur=0",
                    "-resize",
                    "33x42",
                    "-monochrome",
                    "-depth",
                    "1",
                ])
                .arg(format!("gray:{}", dest_path.display()))
                .status()
                .expect("could not download icons");

            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
    println!("cargo:rerun-if-changed=build.rs");
}
