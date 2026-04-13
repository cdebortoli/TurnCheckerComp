use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=graphics.toml");
    println!("cargo:rerun-if-changed=parameters.txt");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=locales");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let profile_dir = out_dir
        .ancestors()
        .nth(3)
        .expect("OUT_DIR should contain target/<profile>/build/<pkg>/out")
        .to_path_buf();

    for file_name in ["graphics.toml", "parameters.txt"] {
        let source = manifest_dir.join(file_name);
        let destination = profile_dir.join(file_name);

        fs::copy(&source, &destination).unwrap_or_else(|error| {
            panic!(
                "failed to copy {} to {}: {}",
                source.display(),
                destination.display(),
                error
            )
        });
    }
}
