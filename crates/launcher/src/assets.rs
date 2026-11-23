use anyhow::{Context, Result};
use jvm::{info, warning};
use serde_json::Value;
use std::fs::File;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

const CLIENT_JAR_ENV: &str = "B173_CLIENT_JAR";
const SOUNDS_DIR_ENV: &str = "B173_SOUNDS_DIR";
const LAUNCHER_JAR: &str = "versions/b1.7.3/b1.7.3.jar";
const VERSION_ID: &str = "b1.7.3";
const CLIENT_JAR_SHA1: &str = "43db9b498cb67058d2e12d394e6507722e71bb45";

mod url
{
    pub const VERSION_MANIFEST: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
    pub const RESOURCES: &str = "https://resources.download.minecraft.net";
    pub const MCASSET: &str = "https://mcasset.cloud/b1.7.3/assets";
}

// beta has no status effect art so the effect boxes and icons come from the first release with the same sheet
mod effects
{
    pub const VERSION_ID: &str = "1.0";
    pub const LAUNCHER_JAR: &str = "versions/1.0/1.0.jar";
    pub const SOURCE: &str = "gui/inventory.png";
    pub const SHEET: &str = "gui/effects.png";
    pub const STAMP: &str = ".effects-sheet";
}

const CLIENT_STAMP: &str = ".client-jar.sha1";
const SOUNDS_STAMP: &str = ".sounds-installed";

const SOUND_FOLDERS: &[&str] = &["sound", "newsound", "music", "newmusic", "streaming"];

// one agent keeps connections alive across the sound downloads
static AGENT: LazyLock<ureq::Agent> = LazyLock::new(ureq::Agent::new_with_defaults);

pub(crate) fn install(root: &Path, dataDir: &Path, clientJar: Option<PathBuf>) -> Result<()>
{
    let resources: PathBuf = dataDir.join("resources");

    if !stampMatches(&root.join(CLIENT_STAMP), CLIENT_JAR_SHA1)
    {
        fetchClient(root, clientJar.or_else(localJar))?;
    }

    let effectsStamp: PathBuf = root.join(effects::STAMP);
    if !stampMatches(&effectsStamp, effects::VERSION_ID)
    {
        // the game runs without the sheet so a failure only costs the effect art
        match fetchEffects(root)
        {
            Ok(()) => write(&effectsStamp, effects::VERSION_ID.as_bytes())?,
            Err(error) => warning!("No status effect sheet, effect boxes and icons will be missing: {error:#}"),
        }
    }

    let soundsStamp: PathBuf = root.join(SOUNDS_STAMP);
    if !stampMatches(&soundsStamp, &resources.to_string_lossy())
    {
        match std::env::var_os(SOUNDS_DIR_ENV).map(PathBuf::from).filter(|folder| folder.is_dir())
        {
            Some(folder) => copySounds(&resources, &folder)?,
            None => fetchSounds(&resources)?,
        }

        write(&soundsStamp, resources.to_string_lossy().as_bytes())?;
    }

    Ok(())
}

fn stampMatches(stamp: &Path, expected: &str) -> bool
{
    std::fs::read_to_string(stamp).is_ok_and(|text| text.trim() == expected)
}

fn localJar() -> Option<PathBuf>
{
    std::env::var_os(CLIENT_JAR_ENV)
        .map(PathBuf::from)
        .or_else(|| mojangDir().map(|dir| dir.join(LAUNCHER_JAR)).filter(|jar| jar.is_file()))
}

// the official launcher's .minecraft folder, where it keeps the b1.7.3 jar once that version was played
fn mojangDir() -> Option<PathBuf>
{
    let fromEnv = |name: &str| std::env::var_os(name).filter(|value| !value.is_empty()).map(PathBuf::from);

    Some(match std::env::consts::OS
    {
        "windows" => fromEnv("APPDATA")
            .or_else(|| fromEnv("USERPROFILE").map(|home| home.join("AppData").join("Roaming")))?
            .join(".minecraft"),
        "macos" => fromEnv("HOME")?.join("Library").join("Application Support").join("minecraft"),
        _ => fromEnv("HOME")?.join(".minecraft"),
    })
}

fn isSound(name: &str) -> bool { name.split_once('/').is_some_and(|(folder, _)| SOUND_FOLDERS.contains(&folder)) }

fn copySounds(resources: &Path, source: &Path) -> Result<()>
{
    info!("Copying sounds from {} into {}", source.display(), resources.display());

    for folder in SOUND_FOLDERS
    {
        let from: PathBuf = source.join(folder);
        if from.is_dir()
        {
            copyTree(&from, &resources.join(folder))?;
        }
    }

    Ok(())
}

fn copyTree(from: &Path, to: &Path) -> Result<()>
{
    std::fs::create_dir_all(to).with_context(|| format!("creating {}", to.display()))?;

    for entry in std::fs::read_dir(from).with_context(|| format!("reading {}", from.display()))?
    {
        let path: PathBuf = entry.with_context(|| format!("reading {}", from.display()))?.path();
        let destination: PathBuf = to.join(path.file_name().context("directory entry has no name")?);

        if path.is_dir()
        {
            copyTree(&path, &destination)?;
        }
        else
        {
            std::fs::copy(&path, &destination)
                .with_context(|| format!("copying {} to {}", path.display(), destination.display()))?;
        }
    }

    Ok(())
}

fn fetchClient(root: &Path, jar: Option<PathBuf>) -> Result<()>
{
    let bytes: Vec<u8> = match jar
    {
        Some(path) =>
        {
            info!("Extracting assets from {}", path.display());
            std::fs::read(&path).with_context(|| format!("reading {}", path.display()))?
        }

        None =>
        {
            info!("No local client jar, downloading the {VERSION_ID} client from Mojang");

            downloadClientJar().with_context(||
            {
                format!("copy the files of {} into {} to install the assets by hand", url::MCASSET, root.display())
            })?
        }
    };
    verify("client jar", &bytes, CLIENT_JAR_SHA1)?;
    extractJar(root, &bytes)?;
    write(&root.join(CLIENT_STAMP), CLIENT_JAR_SHA1.as_bytes())
}

fn downloadClientJar() -> Result<Vec<u8>> { download(text(&versionJson(VERSION_ID)?, "/downloads/client/url")?) }

fn fetchEffects(root: &Path) -> Result<()>
{
    let local: Option<PathBuf> = mojangDir().map(|dir| dir.join(effects::LAUNCHER_JAR)).filter(|jar| jar.is_file());
    let bytes: Vec<u8> = match local
    {
        Some(path) =>
        {
            info!("Taking the status effect sheet from {}", path.display());
            std::fs::read(&path).with_context(|| format!("reading {}", path.display()))?
        }

        None =>
        {
            info!("Downloading the {} client from Mojang for its status effect sheet", effects::VERSION_ID);

            let version: Value = versionJson(effects::VERSION_ID)?;
            let jar: Vec<u8> = download(text(&version, "/downloads/client/url")?)?;
            verify("1.0 client jar", &jar, text(&version, "/downloads/client/sha1")?)?;
            jar
        }
    };

    let mut archive: zip::ZipArchive<Cursor<&[u8]>> =
        zip::ZipArchive::new(Cursor::new(bytes.as_slice())).context("opening the 1.0 client jar")?;
    let mut entry: zip::read::ZipFile<'_, Cursor<&[u8]>> =
        archive.by_name(effects::SOURCE).with_context(|| format!("the 1.0 client jar has no {}", effects::SOURCE))?;
    let path: PathBuf = root.join(effects::SHEET);
    std::io::copy(&mut entry, &mut create(&path)?).with_context(|| format!("extracting {}", path.display()))?;

    info!("Status effect sheet installed at {}", path.display());
    Ok(())
}

fn fetchSounds(resources: &Path) -> Result<()>
{
    let version: Value = versionJson(VERSION_ID)?;
    let indexUrl: &str = text(&version, "/assetIndex/url")?;
    let indexBytes: Vec<u8> = download(indexUrl)?;
    verify(indexUrl, &indexBytes, text(&version, "/assetIndex/sha1")?)?;

    let index: Value = parseJson(indexUrl, &indexBytes)?;
    let objects: &serde_json::Map<String, Value> =
        index.get("objects").and_then(Value::as_object).with_context(|| format!("{indexUrl} has no objects map"))?;
    let wanted: Vec<(&String, &Value)> = objects.iter().filter(|(name, _)| isSound(name)).collect();
    info!("Downloading {} sound files into {}", wanted.len(), resources.display());

    for (name, object) in wanted
    {
        let hash: &str = text(object, "/hash")?;
        let destination: PathBuf = resources.join(name);
        if std::fs::read(&destination).is_ok_and(|bytes| verify(name, &bytes, hash).is_ok())
        {
            continue;
        }

        let folder: &str = hash.get(..2).with_context(|| format!("short hash for {name}"))?;
        let bytes: Vec<u8> = download(&format!("{}/{folder}/{hash}", url::RESOURCES))?;
        verify(name, &bytes, hash)?;
        write(&destination, &bytes)?;
    }

    Ok(())
}

fn versionJson(id: &str) -> Result<Value>
{
    let manifest: Value = parseJson(url::VERSION_MANIFEST, &download(url::VERSION_MANIFEST)?)?;
    let entry: &Value = manifest
        .get("versions")
        .and_then(Value::as_array)
        .and_then(|versions| versions.iter().find(|version| version.get("id").and_then(Value::as_str) == Some(id)))
        .with_context(|| format!("{} has no {id} entry", url::VERSION_MANIFEST))?;
    let url: &str = text(entry, "/url")?;
    let bytes: Vec<u8> = download(url)?;
    verify(url, &bytes, text(entry, "/sha1")?)?;
    parseJson(url, &bytes)
}

fn text<'a>(json: &'a Value, pointer: &str) -> Result<&'a str>
{
    json.pointer(pointer).and_then(Value::as_str).with_context(|| format!("metadata has no {pointer}"))
}

fn parseJson(url: &str, bytes: &[u8]) -> Result<Value>
{
    serde_json::from_slice(bytes).with_context(|| format!("parsing {url}"))
}

fn download(url: &str) -> Result<Vec<u8>>
{
    AGENT
        .get(url)
        .call()
        .and_then(|mut response| response.body_mut().with_config().limit(u64::MAX).read_to_vec())
        .with_context(|| format!("downloading {url}"))
}

fn verify(what: &str, bytes: &[u8], sha1: &str) -> Result<()>
{
    let actual: String = sha1_smol::Sha1::from(bytes).digest().to_string();
    anyhow::ensure!(actual.eq_ignore_ascii_case(sha1), "sha1 mismatch for {what}: expected {sha1}, got {actual}");
    Ok(())
}

fn extractJar(root: &Path, bytes: &[u8]) -> Result<()>
{
    let mut archive: zip::ZipArchive<Cursor<&[u8]>> =
        zip::ZipArchive::new(Cursor::new(bytes)).context("opening the client jar")?;

    for index in 0..archive.len()
    {
        let mut entry: zip::read::ZipFile<'_, Cursor<&[u8]>> =
            archive.by_index(index).context("reading the client jar")?;

        let Some(relative) = entry.enclosed_name()
        else
        {
            continue;
        };
        if entry.is_dir() || relative.starts_with("META-INF") || relative.extension().is_some_and(|ext| ext == "class")
        {
            continue;
        }

        let path: PathBuf = root.join(&relative);
        std::io::copy(&mut entry, &mut create(&path)?).with_context(|| format!("extracting {}", path.display()))?;
    }

    info!("Assets extracted into {}", root.display());
    Ok(())
}

fn create(path: &Path) -> Result<File>
{
    if let Some(parent) = path.parent()
    {
        std::fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }

    File::create(path).with_context(|| format!("creating {}", path.display()))
}

fn write(path: &Path, bytes: &[u8]) -> Result<()>
{
    create(path)?.write_all(bytes).with_context(|| format!("writing {}", path.display()))
}
