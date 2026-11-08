use std::path::{Path, PathBuf};

pub const MODS_DIR: &str = "mods";
pub const CONFIG_DIR: &str = "config";
pub const CACHE_DIR: &str = ".cache/mods";

#[derive(Clone, Debug)]
pub struct LoaderPaths
{
    pub m_root: PathBuf,
    pub m_mods: PathBuf,
    pub m_config: PathBuf,
    pub m_cache: PathBuf,
}

impl LoaderPaths
{
    pub fn under(root: &Path) -> LoaderPaths
    {
        LoaderPaths {
            m_root: root.to_path_buf(),
            m_mods: root.join(MODS_DIR),
            m_config: root.join(CONFIG_DIR),
            m_cache: root.join(CACHE_DIR),
        }
    }

    pub fn create(&self) -> std::io::Result<()>
    {
        std::fs::create_dir_all(&self.m_mods)?;
        std::fs::create_dir_all(&self.m_config)
    }
}
