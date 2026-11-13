use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};

const NAME: &str = "yt-dlp";
const RELEASES: &str = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/";
const SUMS: &str = "SHA2-256SUMS";
const DIR: &str = "tools";
// sites change often and an old yt-dlp stops working so the kept copy is replaced weekly
const REFRESH: Duration = Duration::from_secs(7 * 24 * 60 * 60);
const MAX_BYTES: u64 = 64 * 1024 * 1024;

// the official single file builds that need no python, by target
const BUILDS: [Build; 4] = [
    Build { m_os: "windows", m_arch: "x86_64", m_file: "yt-dlp.exe" },
    Build { m_os: "linux", m_arch: "x86_64", m_file: "yt-dlp_linux" },
    Build { m_os: "linux", m_arch: "aarch64", m_file: "yt-dlp_linux_aarch64" },
    Build { m_os: "macos", m_arch: "", m_file: "yt-dlp_macos" },
];

struct Build
{
    m_os: &'static str,
    m_arch: &'static str,
    m_file: &'static str,
}

// a console program started from the game opens no window on windows
pub fn command(program: &Path) -> Command
{
    #[allow(unused_mut)]
    let mut command: Command = Command::new(program);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    command
}

// yt-dlp from the path when the player installed it, else the official build kept in the loader cache
pub fn ytDlp(cache: &Path) -> Result<PathBuf, String>
{
    let installed: bool =
        command(Path::new(NAME)).arg("--version").output().is_ok_and(|output| output.status.success());
    if installed
    {
        return Ok(PathBuf::from(NAME));
    }

    let build: &Build = BUILDS
        .iter()
        .find(|build|
        {
            build.m_os == std::env::consts::OS && (build.m_arch.is_empty() || build.m_arch == std::env::consts::ARCH)
        })
        .ok_or_else(|| format!("install {NAME}, there is no ready build for this system"))?;
    let kept: PathBuf = cache.join(DIR).join(build.m_file);

    if !stale(&kept)
    {
        return Ok(kept);
    }

    match fetch(build, &kept)
    {
        Ok(()) => Ok(kept),
        Err(reason) if kept.is_file() =>
        {
            crate::log::warn(crate::log::LOADER, &format!("kept the old {NAME}: {reason}"));
            Ok(kept)
        }
        Err(reason) => Err(reason),
    }
}

fn stale(path: &Path) -> bool
{
    let age: Option<Duration> = std::fs::metadata(path)
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|time| SystemTime::now().duration_since(time).ok());
    age.is_none_or(|age| age > REFRESH)
}

// the build is checked against the release's own checksum list before it is kept
fn fetch(build: &Build, kept: &Path) -> Result<(), String>
{
    crate::log::info(crate::log::LOADER, &format!("downloading {}", build.m_file));

    let sums: String = download(&format!("{RELEASES}{SUMS}"))
        .and_then(|bytes| String::from_utf8(bytes).map_err(|error| error.to_string()))?;
    let expected: String = sums
        .lines()
        .filter_map(|line| line.split_once(char::is_whitespace))
        .find(|(_, file)| file.trim() == build.m_file)
        .map(|(sum, _)| sum.to_lowercase())
        .ok_or_else(|| format!("{SUMS} lists no {}", build.m_file))?;

    let bytes: Vec<u8> = download(&format!("{RELEASES}{}", build.m_file))?;
    if super::hashOf(&bytes) != expected
    {
        return Err(format!("{} did not match its checksum", build.m_file));
    }

    let dir: &Path = kept.parent().ok_or("the cache has no folder")?;
    let fresh: PathBuf = kept.with_extension("part");
    std::fs::create_dir_all(dir).and_then(|()| std::fs::write(&fresh, &bytes)).map_err(|error| error.to_string())?;
    executable(&fresh)?;
    std::fs::rename(&fresh, kept).map_err(|error| error.to_string())
}

fn download(url: &str) -> Result<Vec<u8>, String>
{
    let mut response: ureq::http::Response<ureq::Body> =
        ureq::get(url).call().map_err(|error| format!("could not download {url}: {error}"))?;
    response
        .body_mut()
        .with_config()
        .limit(MAX_BYTES)
        .read_to_vec()
        .map_err(|error| format!("could not download {url}: {error}"))
}

#[cfg(unix)]
fn executable(path: &Path) -> Result<(), String>
{
    use std::os::unix::fs::PermissionsExt;
    const MODE: u32 = 0o755;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(MODE)).map_err(|error| error.to_string())
}

#[cfg(not(unix))]
fn executable(_path: &Path) -> Result<(), String> { Ok(()) }
