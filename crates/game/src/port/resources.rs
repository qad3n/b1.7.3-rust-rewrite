use jvm::io::ByteArrayInputStream;
use std::path::PathBuf;

pub fn assetRoot() -> PathBuf { jvm::data_dir::assets() }

// Class.getResourceAsStream for a jar path such as /lang/en_US.lang, null when no texture pack has the file
pub fn getResourceAsStream(path: &str) -> Option<ByteArrayInputStream>
{
    crate::resources::readBeta(path).map(ByteArrayInputStream::new)
}
