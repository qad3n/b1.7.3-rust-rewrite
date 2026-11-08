use anyhow::{Context, Result};
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

const CLIENT_STAMP: &str = ".client-jar.sha1";
const SOUNDS_STAMP: &str = ".sounds-installed";

const SOUND_FOLDERS: &[&str] = &["sound", "newsound", "music", "newmusic", "streaming"];

// one agent keeps connections alive across the sound downloads
static AGENT: LazyLock<ureq::Agent> = LazyLock::new(ureq::Agent::new_with_defaults);

pub fn install(root: &Path, gameDir: &Path, clientJar: Option<PathBuf>) -> Result<()>
{
    let resources: PathBuf = gameDir.join("resources");
    if !stampMatches(&root.join(CLIENT_STAMP), CLIENT_JAR_SHA1)
    {
        fetchClient(root, clientJar.or_else(|| localJar(gameDir)))?;
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

fn localJar(gameDir: &Path) -> Option<PathBuf>
{
    std::env::var_os(CLIENT_JAR_ENV)
        .map(PathBuf::from)
        .or_else(|| Some(gameDir.join(LAUNCHER_JAR)).filter(|jar| jar.is_file()))
}

fn isSound(name: &str) -> bool { name.split_once('/').is_some_and(|(folder, _)| SOUND_FOLDERS.contains(&folder)) }

fn copySounds(resources: &Path, source: &Path) -> Result<()>
{
    eprintln!("beta173_launcher: copying sounds from {} into {}", source.display(), resources.display());
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
            eprintln!("beta173_launcher: extracting assets from {}", path.display());
            std::fs::read(&path).with_context(|| format!("reading {}", path.display()))?
        }
        None =>
        {
            eprintln!("beta173_launcher: no local client jar, downloading the {VERSION_ID} client from Mojang");
            downloadClientJar().with_context(|| {
                format!("copy the files of {} into {} to install the assets by hand", url::MCASSET, root.display())
            })?
        }
    };
    verify("client jar", &bytes, CLIENT_JAR_SHA1)?;
    extractJar(root, &bytes)?;
    write(&root.join(CLIENT_STAMP), CLIENT_JAR_SHA1.as_bytes())
}

fn downloadClientJar() -> Result<Vec<u8>> { download(text(&versionJson()?, "/downloads/client/url")?) }

fn fetchSounds(resources: &Path) -> Result<()>
{
    let version: Value = versionJson()?;
    let indexUrl: &str = text(&version, "/assetIndex/url")?;
    let indexBytes: Vec<u8> = download(indexUrl)?;
    verify(indexUrl, &indexBytes, text(&version, "/assetIndex/sha1")?)?;
    let index: Value = parseJson(indexUrl, &indexBytes)?;
    let objects: &serde_json::Map<String, Value> =
        index.get("objects").and_then(Value::as_object).with_context(|| format!("{indexUrl} has no objects map"))?;
    let wanted: Vec<(&String, &Value)> = objects.iter().filter(|(name, _)| isSound(name)).collect();
    eprintln!("beta173_launcher: downloading {} sound files into {}", wanted.len(), resources.display());
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

fn versionJson() -> Result<Value>
{
    let manifest: Value = parseJson(url::VERSION_MANIFEST, &download(url::VERSION_MANIFEST)?)?;
    let entry: &Value = manifest
        .get("versions")
        .and_then(Value::as_array)
        .and_then(|versions| {
            versions.iter().find(|version| version.get("id").and_then(Value::as_str) == Some(VERSION_ID))
        })
        .with_context(|| format!("{} has no {VERSION_ID} entry", url::VERSION_MANIFEST))?;
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
    eprintln!("beta173_launcher: assets extracted into {}", root.display());
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
