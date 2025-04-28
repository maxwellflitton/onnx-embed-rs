//! Downloads the ONNX runtime lib and embeds it into Rust code.
extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Literal;
use syn::{parse_macro_input, LitStr};
use quote::quote;
use std::{
    env,
    env::consts,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};
mod file_extraction;
use file_extraction::{
    FileType,
    extract_tgz,
    extract_zip
};


/// Constructs the URL and details for downloading ONNX Runtime based on platform.
/// 
/// # Arguments
/// - onnx_version: the version of ONNX to download
/// 
/// # Returns
/// (url, package_name, ext, dylib_name)
fn get_onnxruntime_url(onnx_version: &str) -> (String, String, String, String, FileType) {
    let base_url = format!(
        "https://github.com/microsoft/onnxruntime/releases/download/v{}/",
        onnx_version
    );

    match (consts::OS, consts::ARCH) {
        ("linux", "x86_64") => (
            format!("{}onnxruntime-linux-x64-{}.tgz", base_url, onnx_version),
            format!("onnxruntime-linux-x64-{}", onnx_version),
            "tgz".to_string(),
            "libonnxruntime.so".to_string(),
            FileType::Tgz
        ),
        ("linux", "aarch64") => (
            format!("{}onnxruntime-linux-aarch64-{}.tgz", base_url, onnx_version),
            format!("onnxruntime-linux-aarch64-{}", onnx_version),
            "tgz".to_string(),
            "libonnxruntime.so".to_string(),
            FileType::Tgz
        ),
        ("macos", "x86_64") => (
            format!("{}onnxruntime-osx-x86_64-{}.tgz", base_url, onnx_version),
            format!("onnxruntime-osx-x86_64-{}", onnx_version),
            "tgz".to_string(),
            "libonnxruntime.dylib".to_string(),
            FileType::Tgz
        ),
        ("macos", "aarch64") => (
            format!("{}onnxruntime-osx-arm64-{}.tgz", base_url, onnx_version),
            format!("onnxruntime-osx-arm64-{}", onnx_version),
            "tgz".to_string(),
            "libonnxruntime.dylib".to_string(),
            FileType::Tgz
        ),
        ("windows", "x86_64") => (
            format!("{}onnxruntime-win-x64-{}.zip", base_url, onnx_version),
            format!("onnxruntime-win-x64-{}", onnx_version),
            "zip".to_string(),
            "onnxruntime.dll".to_string(),
            FileType::Zip
        ),
        ("windows", "aarch64") => (
            format!("{}onnxruntime-win-arm64-{}.zip", base_url, onnx_version),
            format!("onnxruntime-win-arm64-{}", onnx_version),
            "zip".to_string(),
            "onnxruntime.dll".to_string(),
            FileType::Zip
        ),
        _ => panic!(
            "Unsupported platform or architecture: {} {}",
            consts::OS,
            consts::ARCH
        ),
    }
}

#[proc_macro]
pub fn embed_onnx(attr: TokenStream) -> TokenStream {

    // get the onnx version
    let input = parse_macro_input!(attr as LitStr);
    let supported_versions = ["1.20.0"];
    let onnx_version = match input.value().as_str() {
        "1.20.0" => "1.20.0",
        _ => panic!(
            "{} passed in as version, only the following versions are supported: {:?}", 
            input.value(), supported_versions
        )
    };

    let (url, package_name, ext, dylib_name, file_type) = get_onnxruntime_url(
        onnx_version
    );

    // Persistent cache under target directory
    let target_root = std::env::var("CARGO_TARGET_DIR")
    .map(PathBuf::from)
    .unwrap_or_else(|_| {
        Path::new(&env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("crate not in a workspace?")
            .join("target")
    });

    let cache = target_root.join("onnxruntime_cache").join(onnx_version);
    fs::create_dir_all(&cache).expect("failed to create cache dir");

    let filename = format!("{}.{}", package_name, ext);
    let download_path = cache.join(&filename);
    let extract_target = cache.join(&package_name);
    let dylib_path = extract_target.join("lib").join(&dylib_name);

    let lock_path = cache.join("onnx_download.lock");
    let mut lock = fslock::LockFile::open(&lock_path).expect("Failed to open lock file");
    lock.lock().expect("Failed to acquire download lock");

    if !download_path.exists() {
        println!("Downloading ONNX Runtime from {}", url);
        let response = reqwest::blocking::get(&url)
            .expect("Failed to download ONNX Runtime")
            .bytes()
            .expect("Failed to read ONNX Runtime response");

        let mut file = File::create(&download_path).expect("Failed to create ONNX file");
        file.write_all(&response).expect("Failed to write ONNX file");
        println!("Saved to {}", download_path.display());
    }

    if !dylib_path.exists() {
        match file_type {
            FileType::Tgz => extract_tgz(&download_path, &cache).expect("Failed to extract ONNX archive for tgz"),
            FileType::Zip => extract_zip(&download_path, &cache).expect("Failed to extract ONNX archive for zip")
        };
    }

    let bytes: Vec<u8> = fs::read(&dylib_path).expect("Failed to read extracted library");

    lock.unlock().expect("Failed to release download lock");

    let byte_string = Literal::byte_string(&bytes);

    let tokens = quote! {
        #byte_string
    };

    TokenStream::from(tokens)
}
