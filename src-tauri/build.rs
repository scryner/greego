use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let has_apple_feature = env::var("CARGO_FEATURE_APPLE").is_ok();

    if target_os == "macr" && has_apple_feature { // "macr" is likely a typo in my thought, assuming "macos" or standard cargo cfg
         // Wait, target_os is usually "macos".
    }

    if target_os == "macos" && has_apple_feature {
        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let swift_file = manifest_dir.join("src/llm/provider/apple/bridge.swift");
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let lib_name = "apple_llm_bridge";
        let lib_filename = format!("lib{}.a", lib_name);
        let lib_path = out_dir.join(&lib_filename);

        println!("cargo:rerun-if-changed={}", swift_file.display());

        // Check if swiftc is available
        let status = Command::new("swiftc").arg("--version").output();

        if status.is_ok() {
            let status = Command::new("swiftc")
                .args(&[
                    "-emit-library",
                    "-static",
                    "-module-name",
                    "AppleLLMBridge",
                    "-o",
                    lib_path.to_str().unwrap(),
                    swift_file.to_str().unwrap(),
                    "-target",
                    "arm64-apple-macosx26.0",
                ])
                .status()
                .expect("Failed to execute swiftc");

            if !status.success() {
                panic!("Swift compilation failed with status: {}", status);
            }

            if status.success() {
                println!("cargo:rustc-link-search=native={}", out_dir.display());
                println!("cargo:rustc-link-lib=static={}", lib_name);

                // Add Swift runtime paths to rpath
                let target_info = Command::new("swiftc")
                    .arg("-print-target-info")
                    .output()
                    .expect("Failed to get swiftc target info");

                if target_info.status.success() {
                    let output = String::from_utf8_lossy(&target_info.stdout);
                    // Simple JSON parsing to find runtimeLibraryPaths
                    // We look for "runtimeLibraryPaths" and extract the paths
                    if let Some(start) = output.find("\"runtimeLibraryPaths\": [") {
                        let rest = &output[start..];
                        if let Some(end) = rest.find(']') {
                            let paths_block = &rest[..end];
                            for line in paths_block.lines() {
                                if let Some(start_quote) = line.find('"') {
                                    if let Some(end_quote) = line.rfind('"') {
                                        if start_quote < end_quote {
                                            let path = &line[start_quote + 1..end_quote];
                                            if path != "runtimeLibraryPaths" {
                                                // Skip the key itself if matched
                                                println!(
                                                    "cargo:rustc-link-arg=-Wl,-rpath,{}",
                                                    path
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    tauri_build::build()
}
