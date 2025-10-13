use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    let profile = env::var("PROFILE").unwrap();
    if profile == "debug" {
        let source_path = Path::new("test_page.html");
        if !source_path.exists() {
            return;
        }

        let dest_dir = Path::new(&out_dir)
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap();

        let dest_path = dest_dir.join("test_page.html");

        fs::copy(&source_path, &dest_path).expect("Failed to copy test_page.html");

        println!("cargo:rerun-if-changed=test_page.html");
    }
}
