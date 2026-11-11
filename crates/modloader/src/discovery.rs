use crate::manifest::{MANIFEST_FILE, Manifest};
use game::resources::Source;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub const MOD_EXTENSION: &str = "b173mod";

pub struct Discovered
{
    pub m_manifest: Manifest,
    pub m_source: Source,
    pub m_sha256: String,
}

fn hashFile(path: &Path) -> std::io::Result<String>
{
    let bytes: Vec<u8> = std::fs::read(path)?;
    Ok(hex(&Sha256::digest(&bytes)))
}

pub fn hex(bytes: &[u8]) -> String { bytes.iter().map(|byte| format!("{byte:02x}")).collect() }

// a development folder hashes its manifest and wasm, which is what the network check compares
fn hashFolder(path: &Path) -> String
{
    let mut hasher: Sha256 = Sha256::new();

    for name in [MANIFEST_FILE, crate::manifest::WASM_FILE]
    {
        if let Ok(bytes) = std::fs::read(path.join(name))
        {
            hasher.update(&bytes);
        }
    }

    hex(&hasher.finalize())
}

fn open(path: &Path) -> Result<Discovered, String>
{
    let mut source: Source = if path.is_dir()
    {
        Source::Folder(path.to_path_buf())
    }
    else
    {
        let file: std::fs::File = std::fs::File::open(path).map_err(|error| error.to_string())?;
        let archive: zip::ZipArchive<std::fs::File> =
            zip::ZipArchive::new(file).map_err(|error| format!("not a valid {MOD_EXTENSION} zip: {error}"))?;
        Source::Archive(path.to_path_buf(), archive)
    };
    let manifestBytes: Vec<u8> = source.read(MANIFEST_FILE).ok_or_else(|| format!("{MANIFEST_FILE} is missing"))?;
    let text: String = String::from_utf8(manifestBytes).map_err(|_| format!("{MANIFEST_FILE} is not UTF-8"))?;
    let manifest: Manifest = Manifest::parse(&text).map_err(|error| format!("{MANIFEST_FILE}: {error}"))?;

    let sha256: String = match &source
    {
        Source::Archive(path, _) => hashFile(path).map_err(|error| error.to_string())?,
        Source::Folder(path) => hashFolder(path),
    };
    Ok(Discovered { m_manifest: manifest, m_source: source, m_sha256: sha256 })
}

pub struct DiscoveryError
{
    pub m_path: PathBuf,
    pub m_reason: String,
}

// every b173mod file and every folder holding a mod.toml, in file name order
pub fn discover(modsDir: &Path) -> (Vec<Discovered>, Vec<DiscoveryError>)
{
    let mut paths: Vec<PathBuf> = match std::fs::read_dir(modsDir)
    {
        Ok(entries) => entries.flatten().map(|entry| entry.path()).collect(),
        Err(_) => Vec::new(),
    };
    paths.sort();

    let mut found: Vec<Discovered> = Vec::new();
    let mut errors: Vec<DiscoveryError> = Vec::new();

    for path in paths
    {
        let isArchive: bool = path.extension().is_some_and(|extension| extension == MOD_EXTENSION);
        let isFolder: bool = path.is_dir() && path.join(MANIFEST_FILE).is_file();
        if !isArchive && !isFolder
        {
            continue;
        }

        match open(&path)
        {
            Ok(discovered) => found.push(discovered),
            Err(reason) => errors.push(DiscoveryError { m_path: path, m_reason: reason }),
        }
    }

    (found, errors)
}

pub fn hasMods(modsDir: &Path) -> bool
{
    std::fs::read_dir(modsDir).is_ok_and(|entries|
    {
        entries.flatten().any(|entry|
        {
            let path: PathBuf = entry.path();
            path.extension().is_some_and(|extension| extension == MOD_EXTENSION)
                || (path.is_dir() && path.join(MANIFEST_FILE).is_file())
        })
    })
}
