use std::{env, path::PathBuf};

fn main() {
    #[cfg(target_os = "windows")]
    {
        let library_system = "native-system";
        println!("cargo:rustc-link-lib=static={}", library_system);

        let library_system = "winconpty";
        println!("cargo:rustc-link-lib=static={}", library_system);

        let library_mman = "mman";
        println!("cargo:rustc-link-lib=static={}", library_mman);
    }

    let library_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    println!(
        "cargo:rustc-link-search=native={}/",
        env::join_paths([library_dir]).unwrap().to_str().unwrap()
    );

    let library_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    println!(
        "cargo:rustc-link-search=dylib={}/",
        env::join_paths([library_dir]).unwrap().to_str().unwrap()
    );
}
