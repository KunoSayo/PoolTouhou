fn main() {
    let out_dir = std::env::var("OUT_DIR").expect("Failed to get out dir");
    let out_times = std::env::var("STD_LIB_COPY_UP_TIMES").unwrap_or("3".into()).parse::<i32>().expect("Times must be number");
    let mut out_path = std::path::Path::new(&out_dir).join(".");
    for _ in 0..out_times {
        out_path = out_path.join("..");
    }
    let rustup_home = std::env::var("RUSTUP_HOME").expect("Need path to get standard dynamic library to run");
    let rustup_path = std::path::Path::new(&rustup_home);
    let toolchains = rustup_path.join("toolchains");
    for toolchain in toolchains.read_dir().expect("Failed to read toolchains dir") {
        match toolchain {
            Ok(toolchain_dir) => {
                let name = toolchain_dir.file_name();
                let lib_loc = toolchain_dir.path().join("lib").join("rustlib")
                    .join(format!("{}", name.to_string_lossy().replace("stable", "").replace("nightly", "").trim_start_matches("-")))
                    .join("lib");
                // println!("cargo:rustc-link-search={}", lib_loc.to_string_lossy());
                for entry in lib_loc.read_dir().expect("Failed to read lib dir") {
                    if let Ok(entry) = entry {
                        let file_os_name = entry.file_name();
                        let file_name = file_os_name.to_string_lossy();
                        let out_file = out_path.join(&file_os_name);
                        if file_name.starts_with("std") && !file_name.ends_with(".lib") && !file_name.ends_with(".pdb") {
                            std::fs::copy(entry.path(), out_file).expect("Failed to copy dyn lib");
                            // println!("cargo:rustc-link-lib=static={}", &file_name[..file_name.len() - 4]);
                        }
                    }
                }
            }
            Err(e) => {
                println!("cargo:warning=Failed to read for {:?}", e);
            }
        }
    }
}
