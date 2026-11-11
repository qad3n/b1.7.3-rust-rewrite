use jvm::warning;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

const ENTRY_LIMIT: u64 = 256 << 20;

pub enum Source
{
    Archive(PathBuf, ZipArchive<File>),
    Folder(PathBuf),
}

impl Source
{
    // a folder as it is or a zip file
    pub fn open(path: &Path) -> Result<Self, String>
    {
        if path.is_dir()
        {
            return Ok(Self::Folder(path.to_path_buf()));
        }

        let file: File = File::open(path).map_err(|error| error.to_string())?;
        Ok(Self::Archive(path.to_path_buf(), ZipArchive::new(file).map_err(|error| error.to_string())?))
    }

    pub fn path(&self) -> &Path
    {
        match self
        {
            Self::Archive(path, _) | Self::Folder(path) => path,
        }
    }

    pub fn read(&mut self, name: &str) -> Option<Vec<u8>>
    {
        match self
        {
            Self::Archive(path, archive) =>
            {
                let entry: zip::read::ZipFile<'_, File> = archive.by_name(name).ok()?;
                let mut bytes: Vec<u8> = Vec::new();
                // rewrite: the declared size is not trusted so a forged entry cannot allocate or inflate without bound
                entry.take(ENTRY_LIMIT + 1).read_to_end(&mut bytes).ok()?;

                if bytes.len() as u64 > ENTRY_LIMIT
                {
                    warning!("{name} in {} is over the {ENTRY_LIMIT} byte entry limit", path.display());
                    return None;
                }

                Some(bytes)
            }

            Self::Folder(root) => std::fs::read(root.join(name)).ok(),
        }
    }

    pub fn contains(&self, name: &str) -> bool
    {
        match self
        {
            Self::Archive(_, archive) => archive.index_for_name(name).is_some(),
            Self::Folder(root) => root.join(name).is_file(),
        }
    }

    // file names under a prefix, with forward slashes, sorted so both sides see one order
    pub fn list(&self, prefix: &str) -> Vec<String>
    {
        let mut names: Vec<String> = match self
        {
            Self::Archive(_, archive) => archive
                .file_names()
                .filter(|name| name.starts_with(prefix) && !name.ends_with('/'))
                .map(str::to_owned)
                .collect(),
            Self::Folder(root) =>
            {
                let mut found: Vec<String> = Vec::new();
                collect(root, &root.join(prefix), &mut found);
                found
            }
        };
        names.sort();
        names
    }
}

fn collect(root: &Path, directory: &Path, found: &mut Vec<String>)
{
    let Ok(entries) = std::fs::read_dir(directory)
    else
    {
        return;
    };

    for entry in entries.flatten()
    {
        let path: PathBuf = entry.path();
        if path.is_dir()
        {
            collect(root, &path, found);
        }
        else if let Ok(relative) = path.strip_prefix(root)
        {
            found.push(relative.to_string_lossy().replace('\\', "/"));
        }
    }
}
