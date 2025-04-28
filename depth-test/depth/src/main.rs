use onnx_embedding::embed_onnx;
use ort::environment::EnvironmentBuilder;
use tempfile::TempDir;
use std::path::PathBuf;

use tempfile::tempdir;
use std::io::Cursor;
use zip::ZipArchive;


fn unzip_to_temp_dir(zip_bytes: &[u8]) -> std::io::Result<(PathBuf, TempDir)> {
    // 1. Create a temp dir
    let temp_dir = tempdir()?;
    let temp_path = temp_dir.path().to_path_buf(); // clone path before move

    // 2. Open a ZipArchive from the embedded bytes
    let reader = Cursor::new(zip_bytes);
    let mut archive = ZipArchive::new(reader)?;

    // 3. Extract files
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = temp_path.join(file.mangled_name());

        if (*file.name()).ends_with('/') {
            // It's a directory
            std::fs::create_dir_all(&outpath)?;
        } else {
            // It's a file
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    // 4. IMPORTANT: TempDir will be deleted when dropped
    // You need to keep the TempDir alive if you want to use the path
    // -> So you must return both the TempDir and the PathBuf!
    Ok((temp_path, temp_dir))
}

fn main() {
    // Embed the ONNX bytes
    // let onnx_bytes = embed_onnx!("1.20.0");
    let onnx_bytes = embed_onnx!("1.20.0");
    // let messed_up_bytes = b"\0\0\x06\0\0\0\x17\0\0\0X\x0b\0\0\x85\0\x91\0\0";

    println!("writing ONNX to file");
    let (extracted_lib_dir, _temp_dir) = unzip_to_temp_dir(onnx_bytes).expect("Failed to unzip ONNX runtime");
    println!("written ONNX to file");

    let onnx_lib_path = if cfg!(target_os = "windows") {
        extracted_lib_dir.join("onnxruntime.dll")
    } else if cfg!(target_os = "macos") {
        extracted_lib_dir.join("libonnxruntime.dylib")
    } else {
        extracted_lib_dir.join("libonnxruntime.so")
    };

    // Initialize ORT from the temporary file path
    println!("constructing ML environment");
    let environment: EnvironmentBuilder = ort::init_from(onnx_lib_path.to_str().unwrap());
    println!("environment constructed");
    let _outcome = environment.commit().unwrap();
    println!("environment committed");
}
