//! For extracting out files from compression.
use std::{
    fs::{self, File},
    io,
    path::Path,
};
use flate2::read::GzDecoder;
use tar::Archive;


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