use std::{env::{self, consts}, fs, io::Write, path::PathBuf, string};


fn get_onnxruntime_url() -> (String, String, String) {

    let onnx_version = "1.20.0";

    let base_url = format!("https://github.com/microsoft/onnxruntime/releases/download/v{}/", onnx_version);

    match (consts::OS, consts::ARCH) {
        ("linux", "x86_64") => (
            format!("{}onnxruntime-linux-x64-{}.tgz", base_url, onnx_version),
            format!("onnxruntime-linux-x64-{}", onnx_version),
            "tgz".to_string()
        ),
        ("linux", "aarch64") => (
            format!("{}onnxruntime-linux-aarch64-{}.tgz", base_url, onnx_version),
            format!("onnxruntime-linux-aarch64-{}", onnx_version),
            "tgz".to_string()
        ),
        ("macos", "x86_64") => (
            format!("{}onnxruntime-osx-x86_64-{}.tgz", base_url, onnx_version),
            format!("onnxruntime-osx-x86_64-{}", onnx_version),
            "tgz".to_string()
        ),
        ("macos", "aarch64") => (
            format!("{}onnxruntime-osx-arm64-{}.tgz", base_url, onnx_version),
            format!("onnxruntime-osx-arm64-{}", onnx_version),
            "tgz".to_string()
        ),
        ("windows", "x86_64") => (
            format!("{}onnxruntime-win-x64-{}.zip", base_url, onnx_version),
            format!("onnxruntime-win-x64-{}", onnx_version),
            "zip".to_string()
        ),
        ("windows", "aarch64") => (
            format!("{}onnxruntime-win-arm64-{}.zip", base_url, onnx_version),
            format!("onnxruntime-win-arm64-{}", onnx_version),
            "zip".to_string()
        ),
        _ => panic!("Unsupported platform or architecture: {} {}", consts::OS, consts::ARCH),
    }
}


fn main() {
    // let (url, package_name, ext) = get_onnxruntime_url();

    // // Path to write the file
    // let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    // let filename = format!("{}.{}", package_name, ext);
    // let download_path = PathBuf::from(&out_dir).join(&filename);

    // // Download file if not exists
    // if !download_path.exists() {
    //     println!("Downloading ONNX Runtime from {}", url);
    //     let response = reqwest::blocking::get(&url)
    //         .expect("Failed to download ONNX Runtime")
    //         .bytes()
    //         .expect("Failed to read ONNX Runtime response");

    //     let mut file = fs::File::create(&download_path).expect("Failed to create ONNX file");
    //     file.write_all(&response).expect("Failed to write ONNX file");
    //     println!("Saved to {}", download_path.display());
    // } else {
    //     println!("ONNX Runtime already exists at {}", download_path.display());
    // }

    // // Inject the path back into the calling crate as a string literal
    // let path_str = download_path.display().to_string();

    // println!("cargo:rustc-env=ONNX_PATH={}", path_str);
}
