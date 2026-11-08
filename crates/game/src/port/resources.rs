use jvm::io::FileInputStream;
use std::path::PathBuf;

pub fn assetRoot() -> PathBuf { jvm::data_dir::assets() }

// Class.getResourceAsStream for a jar path such as /lang/en_US.lang, null when the file is missing
pub fn getResourceAsStream(path: &str) -> Option<FileInputStream>
{
    let file: PathBuf = assetRoot().join(path.trim_start_matches('/'));
    FileInputStream::open(&file).ok()
}
