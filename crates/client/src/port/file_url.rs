use std::path::{Path, PathBuf};

fn isUriChar(c: char) -> bool { c.is_ascii_alphanumeric() || "-_.!~*'()/:@&=+$,;".contains(c) || !c.is_ascii() }

pub fn fileUrl(file: &Path) -> String
{
    let absolute: PathBuf = std::path::absolute(file).unwrap_or_else(|_| file.to_path_buf());
    let mut path: String = absolute.to_string_lossy().into_owned();
    if absolute.is_dir() && !path.ends_with('/')
    {
        path.push('/');
    }
    let mut url: String = "file:".to_owned();
    for c in path.chars()
    {
        if isUriChar(c)
        {
            url.push(c);
        }
        else
        {
            let mut buffer: [u8; 4] = [0; 4];
            for byte in c.encode_utf8(&mut buffer).bytes()
            {
                url.push_str(&format!("%{byte:02X}"));
            }
        }
    }
    url
}
