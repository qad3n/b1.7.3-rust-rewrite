use jvm::io::FileInputStream;
use std::path::PathBuf;

// where the gitignored assets folder is looked up, the folder the game runs in like the original resources dir
pub fn assetRoot() -> PathBuf
{
    match std::env::var_os("B173_ASSETS")
    {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from("assets"),
    }
}

// Class.getResourceAsStream for a jar path such as /lang/en_US.lang, null when the file is missing
pub fn getResourceAsStream(path: &str) -> Option<FileInputStream>
{
    let file: PathBuf = assetRoot().join(path.trim_start_matches('/'));
    FileInputStream::open(&file).ok()
}
