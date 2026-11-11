use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::OnceLock;

mod env
{
    pub const HOME: &str = "B173_HOME";
    pub const ASSETS: &str = "B173_ASSETS";
}

mod folders
{
    pub const CLIENT: &[&str] =
        &["saves", "mods", "config", "logs", "resources", "screenshots", "texturepacks", "stats"];
    pub const SERVER: &[&str] = &["mods", "config", "logs"];
}

const FOLDER: &str = "beta173";

// one per-user folder holds everything the launcher, client and server write, like .minecraft for the original game
pub fn root() -> PathBuf
{
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| envPath(env::HOME).unwrap_or_else(|| osDataDir().join(FOLDER))).clone()
}

pub fn assets() -> PathBuf
{
    static ASSETS: OnceLock<PathBuf> = OnceLock::new();
    ASSETS.get_or_init(|| envPath(env::ASSETS).unwrap_or_else(|| root().join("assets"))).clone()
}

pub fn server() -> PathBuf { root().join("server") }

// a fresh data folder gets its whole layout on first launch
pub fn createClient() -> std::io::Result<PathBuf> { create(root(), folders::CLIENT) }

pub fn createServer() -> std::io::Result<PathBuf> { create(server(), folders::SERVER) }

fn create(dir: PathBuf, names: &[&str]) -> std::io::Result<PathBuf>
{
    for name in names
    {
        std::fs::create_dir_all(dir.join(name))?;
    }

    Ok(dir)
}

// the server changes its working folder, so relative paths from the environment resolve once against the start folder
fn envPath(name: &str) -> Option<PathBuf>
{
    let path: PathBuf = PathBuf::from(nonEmpty(name)?);
    match std::path::absolute(&path)
    {
        Ok(absolute) => Some(absolute),
        Err(error) => panic!("{name}={} is not a usable path: {error}", path.display()),
    }
}

fn osDataDir() -> PathBuf
{
    match std::env::consts::OS
    {
        "windows" =>
        {
            nonEmpty("APPDATA").map_or_else(|| required("USERPROFILE").join("AppData").join("Roaming"), PathBuf::from)
        }

        "macos" => required("HOME").join("Library").join("Application Support"),
        // the XDG spec says to ignore a relative XDG_DATA_HOME
        _ => nonEmpty("XDG_DATA_HOME")
            .map(PathBuf::from)
            .filter(|dir| dir.is_absolute())
            .unwrap_or_else(|| required("HOME").join(".local").join("share")),
    }
}

fn nonEmpty(name: &str) -> Option<OsString> { std::env::var_os(name).filter(|value| !value.is_empty()) }

fn required(name: &str) -> PathBuf
{
    match nonEmpty(name)
    {
        Some(value) => PathBuf::from(value),
        None => panic!("{name} is not set, set {} to the game data folder", env::HOME),
    }
}
