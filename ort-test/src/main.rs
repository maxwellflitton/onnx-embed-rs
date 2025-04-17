use onnx_embedding::embed_onnx;
use ort::environment::EnvironmentBuilder;
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

fn main() {
    // Embed the ONNX bytes
    let onnx_bytes = embed_onnx!("1.20.0");
    // let messed_up_bytes = b"\0\0\x06\0\0\0\x17\0\0\0X\x0b\0\0\x85\0\x91\0\0";

    // Create a named temporary file
    let mut temp_file = NamedTempFile::new().unwrap();

    // Write the ONNX bytes into the file
    temp_file.write_all(onnx_bytes).unwrap();

    // Persist the file to get a stable path (on some platforms)
    let path: PathBuf = temp_file.path().to_path_buf();

    // Initialize ORT from the temporary file path
    let environment: EnvironmentBuilder = ort::init_from(path.to_str().unwrap());
    // let another_env = environment.clone();
    let _outcome = environment.commit().unwrap();
}
