//! For extracting out files from compression.
use std::{
    fs::{self, File},
    io,
    path::Path,
};
use flate2::read::GzDecoder;
use tar::Archive;
use std::io::{Write, Read};
use zip::{write::FileOptions, ZipWriter};


#[derive(Clone)]
pub enum DylibName {
    So,
    Dylib,
    Dll
}

impl From<DylibName> for u8 {

    fn from(value: DylibName) -> Self {
        match value {
            DylibName::So => 1,
            DylibName::Dylib => 2,
            DylibName::Dll => 3
        }
    }

}

impl TryFrom<u8> for DylibName {

    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(DylibName::So),
            2 => Ok(DylibName::Dylib),
            3 => Ok(DylibName::Dll),
            _ => Err(format!("{} is not a supported type", value))
        }
    }

}

impl From<DylibName> for &str {

    fn from(value: DylibName) -> Self {
        match value {
            DylibName::So => "libonnxruntime.so",
            DylibName::Dylib => "libonnxruntime.dylib",
            DylibName::Dll => "onnxruntime.dll"
        }
    }

}


/// The type of file that is being extracted.
/// 
/// # Fields
/// - Tgz: .tgz file
/// - Zip: .zip file
pub enum FileType {
    Tgz,
    Zip
}


/// Extracts a `.tgz` file to the specified directory.
/// 
/// # Arguments
/// - tgz_path: path to .tgz to be extracted from
/// - extract_to: path that the extracted .tgx is unloaded to
pub fn extract_tgz<P: AsRef<Path>>(tgz_path: P, extract_to: P) -> io::Result<()> {
    let file = File::open(&tgz_path)?;
    let decompressor = GzDecoder::new(file);
    let mut archive = Archive::new(decompressor);
    archive.unpack(&extract_to)?;
    Ok(())
}


/// Extracts the `.zip` file to a specified directory.
/// 
/// # Arguments
/// - zip_path: path to .zip to be extracted from
/// - extract_to: path that the extracted .tgx is unloaded to
pub fn extract_zip<P: AsRef<Path>>(zip_path: P, extract_to: P) -> io::Result<()> {
    let file = File::open(&zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = extract_to.as_ref().join(file.mangled_name());

        if (*file.name()).ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }
    Ok(())
}


pub fn zip_dir<T: Write + std::io::Seek>(src_dir: &Path, writer: T) -> zip::result::ZipResult<()> {
    let mut zip = ZipWriter::new(writer);

    let options: FileOptions<'_, ()> = FileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .unix_permissions(0o755);

    let walkdir = walkdir::WalkDir::new(src_dir);
    let mut buffer = Vec::new();

    for entry in walkdir.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = path.strip_prefix(src_dir).unwrap();

        if path.is_file() {
            zip.start_file(name.to_string_lossy(), options)?;
            let mut f = File::open(path)?;
            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            zip.add_directory(name.to_string_lossy(), options)?;
        }
    }

    zip.finish()?;
    Ok(())
}
