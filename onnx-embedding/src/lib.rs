//! Downloads the ONNX runtime lib and embeds it into Rust code.
extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Literal;
use syn::{parse_macro_input, LitStr};
use quote::quote;
use std::{
    env::consts,
    fs::{self, File},
    io::{self, Write},
    path::Path,
};
use flate2::read::GzDecoder;
use tar::Archive;

/// Extracts a `.tgz` file to the specified directory.
/// 
/// # Arguments
/// - tgz_path: path to .tgz to be extracted from
/// - extract_to: path that the extracted .tgx is unloaded to
fn extract_tgz<P: AsRef<Path>>(tgz_path: P, extract_to: P) -> io::Result<()> {
    let file = File::open(&tgz_path)?;
    let decompressor = GzDecoder::new(file);
    let mut archive = Archive::new(decompressor);
    archive.unpack(&extract_to)?;
    Ok(())
}


/// Constructs the URL and details for downloading ONNX Runtime based on platform.
/// 
/// # Arguments
/// - onnx_version: the version of ONNX to download
/// 
/// # Returns
/// (url, package_name, ext, dylib_name)
fn get_onnxruntime_url(onnx_version: &str) -> (String, String, String, String) {
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
        ),
        ("linux", "aarch64") => (
            format!("{}onnxruntime-linux-aarch64-{}.tgz", base_url, onnx_version),
            format!("onnxruntime-linux-aarch64-{}", onnx_version),
            "tgz".to_string(),
            "libonnxruntime.so".to_string(),
        ),
        ("macos", "x86_64") => (
            format!("{}onnxruntime-osx-x86_64-{}.tgz", base_url, onnx_version),
            format!("onnxruntime-osx-x86_64-{}", onnx_version),
            "tgz".to_string(),
            "libonnxruntime.dylib".to_string(),
        ),
        ("macos", "aarch64") => (
            format!("{}onnxruntime-osx-arm64-{}.tgz", base_url, onnx_version),
            format!("onnxruntime-osx-arm64-{}", onnx_version),
            "tgz".to_string(),
            "libonnxruntime.dylib".to_string(),
        ),
        ("windows", "x86_64") => (
            format!("{}onnxruntime-win-x64-{}.zip", base_url, onnx_version),
            format!("onnxruntime-win-x64-{}", onnx_version),
            "zip".to_string(),
            "onnxruntime.dll".to_string(),
        ),
        ("windows", "aarch64") => (
            format!("{}onnxruntime-win-arm64-{}.zip", base_url, onnx_version),
            format!("onnxruntime-win-arm64-{}", onnx_version),
            "zip".to_string(),
            "onnxruntime.dll".to_string(),
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

    let (url, package_name, ext, dylib_name) = get_onnxruntime_url(onnx_version);

    // Create a temporary directory
    let temp_dir = tempfile::Builder::new()
        .prefix("onnxruntime_embed_")
        .tempdir()
        .expect("Failed to create temporary directory");

    let temp_path = temp_dir.path().to_path_buf();
    let filename = format!("{}.{}", package_name, ext);
    let download_path = temp_path.join(&filename);
    let extract_target = temp_path.join(&package_name);
    let tgz_path_str = download_path.to_str().expect("cannot convert download path to string").to_string();
    let dylib_path = extract_target.join("lib").join(dylib_name);

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
        extract_tgz(&tgz_path_str, &temp_path.to_str().expect("cannot convert temp path to string").to_owned()).expect("Failed to extract ONNX archive");
    }

    let bytes: Vec<u8> = fs::read(&dylib_path).expect("Failed to read extracted library");

    // Explicitly clean up the temporary directory
    temp_dir.close().expect("Failed to remove temporary directory");

    let byte_string = Literal::byte_string(&bytes);

    let tokens = quote! {
        #byte_string
    };

    TokenStream::from(tokens)
}
