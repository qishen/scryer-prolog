extern crate indexmap;

use crate::indexmap::IndexSet;

use std::env;
use std::fs::{File, copy, create_dir_all, read_dir};
use std::io::Write;
use std::path::Path;

fn main()
{
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("libraries.rs");

    let mut libraries = File::create(&dest_path).unwrap();
    let mut library_index = IndexSet::new();

    let mut paths: Vec<_> = read_dir("./src/lib").unwrap()
        .map(|e| e.unwrap().path()).collect();

    while let Some(item) = paths.pop() {
        if item.is_file() {
            let file_name = item.strip_prefix(".").unwrap();

            if let Some(ext) = item.extension() {
                if ext == "pl" {
                    let file_stem = item.file_stem().unwrap();
                    let file_str  = file_stem.to_string_lossy().to_uppercase();
                    let dest = Path::new(&out_dir).join(file_name);
                    let dir_name = dest.parent().unwrap();

                    if !dir_name.exists() {
                        create_dir_all(dir_name).unwrap();
                    }

                    match copy(&item, dest) {
                        Ok(_) => {
                        }
                        Err(e) => {
                            panic!("die: {:?}", e)
                        }
                    };

                    let include_line = format!("static {}: &str = include_str!({:?});\n",
                                               file_str, file_name); //file_stem.to_string_lossy());

                    libraries.write_all(include_line.as_bytes()).unwrap();
                    library_index.insert(file_stem.to_string_lossy().to_string());
                }
            }
        } else if item.is_dir() {
            for entry in read_dir(item).unwrap() {
                let path = entry.unwrap().path();
                paths.push(path);
            }
        }
    }

    libraries.write_all(b"\nref_thread_local! {
    pub static managed LIBRARIES: IndexMap<&'static str, &'static str> = {
        let mut m = IndexMap::new();\n").unwrap();

    for item in library_index {
        let line = format!("\n        m.insert(\"{}\", {});", item, item.to_uppercase());
        libraries.write_all(line.as_bytes()).unwrap();
    }

    libraries.write_all(b"\n\n        m\n    };
}\n").unwrap();

    libraries.write_all(b"\npub static PROJECT_DIR: &'static str = \"").unwrap();
    libraries.write_all(env::var("CARGO_MANIFEST_DIR").unwrap().as_bytes()).unwrap();
    libraries.write_all(b"\";\n").unwrap();
}
