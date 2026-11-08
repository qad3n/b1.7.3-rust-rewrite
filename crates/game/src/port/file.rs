use std::path::{Path, PathBuf};

pub fn exists(path: &Path) -> bool { path.exists() }

pub fn isDirectory(path: &Path) -> bool { path.is_dir() }

pub fn mkdir(path: &Path) -> bool { std::fs::create_dir(path).is_ok() }

pub fn mkdirs(path: &Path) -> bool
{
    if path.exists()
    {
        return false;
    }
    std::fs::create_dir_all(path).is_ok()
}

pub fn delete(path: &Path) -> bool
{
    if path.is_dir()
    {
        return std::fs::remove_dir(path).is_ok();
    }
    std::fs::remove_file(path).is_ok()
}

pub fn renameTo(from: &Path, to: &Path) -> bool { std::fs::rename(from, to).is_ok() }

pub fn length(path: &Path) -> i64 { std::fs::metadata(path).map_or(0, |metadata| metadata.len() as i64) }

pub fn lastModified(path: &Path) -> i64
{
    std::fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |duration| duration.as_secs() as i64 * 1000)
}

pub fn getName(path: &Path) -> String
{
    path.file_name().map_or_else(String::new, |name| name.to_string_lossy().into_owned())
}

pub fn listFiles(path: &Path) -> Option<Vec<PathBuf>>
{
    let entries: std::fs::ReadDir = std::fs::read_dir(path).ok()?;
    Some(entries.filter_map(Result::ok).map(|entry| entry.path()).collect())
}

pub fn toStringRadix36(value: i32) -> String
{
    const DIGITS: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut magnitude: i64 = (value as i64).abs();
    let mut digits: Vec<u8> = Vec::new();
    loop
    {
        digits.push(DIGITS[(magnitude % 36) as usize]);
        magnitude /= 36;
        if magnitude == 0
        {
            break;
        }
    }
    if value < 0
    {
        digits.push(b'-');
    }
    digits.reverse();
    String::from_utf8(digits).expect("radix digits are ascii")
}
