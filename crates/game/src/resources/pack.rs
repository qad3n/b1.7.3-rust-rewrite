use crate::resources::{Location, Source, legacy};
use jvm::{debug, info, warning};
use regex::Regex;
use serde_json::Value;
use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};

mod files
{
    pub const META: &str = "pack.mcmeta";
    pub const TEXT: &str = "pack.txt";
    pub const ICON: &str = "pack.png";
}

const TEXTURE: &str = ".png";
// zip files that must exist on disk such as sounds are unpacked here under the data folder
const CACHE: &str = ".cache/texturepacks";

// the list screen shows two description lines of at most this many utf16 units
const LINE_LIMIT: usize = 34;
const LINES: usize = 2;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Layout
{
    // assets/<namespace>/... behind a pack.mcmeta
    Modern,
    // the extracted beta jar itself
    Jar,
    // a beta texture pack zip that beta only read textures from
    Legacy,
}

// a pack.mcmeta filter.block entry hiding files of the packs below
struct Filter
{
    m_namespace: Option<Regex>,
    m_path: Option<Regex>,
}

impl Filter
{
    fn blocks(&self, location: &Location) -> bool
    {
        let namespace: bool = self.m_namespace.as_ref().is_none_or(|pattern| pattern.is_match(&location.m_namespace));
        namespace && self.m_path.as_ref().is_none_or(|pattern| pattern.is_match(&location.m_path))
    }
}

pub struct Pack
{
    m_name: String,
    m_description: Vec<String>,
    m_layout: Layout,
    m_source: RefCell<Source>,
    m_filters: Vec<Filter>,
    m_unpacked: Cell<bool>,
}

impl Pack
{
    pub fn new(name: &str, description: &[&str], layout: Layout, source: Source) -> Self
    {
        Self
        {
            m_name: name.to_owned(),
            m_description: description.iter().map(|line| truncate(line)).collect(),
            m_layout: layout,
            m_source: RefCell::new(source),
            m_filters: Vec::new(),
            m_unpacked: Cell::new(false),
        }
    }

    // a zip or a folder from the texturepacks folder, legacy when it has no pack.mcmeta
    pub fn open(path: &Path) -> Result<Self, String>
    {
        let mut source: Source = if path.is_dir()
        {
            Source::Folder(path.to_path_buf())
        }
        else
        {
            let file: std::fs::File = std::fs::File::open(path).map_err(|error| error.to_string())?;
            Source::Archive(path.to_path_buf(), zip::ZipArchive::new(file).map_err(|error| error.to_string())?)
        };

        let name: String = path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
        let Some(bytes) = source.read(files::META)
        else
        {
            let text: String = source.read(files::TEXT).map_or_else(String::new, |bytes| legacyText(&bytes));
            let lines: Vec<&str> = text.lines().take(LINES).collect();
            return Ok(Self::new(&name, &lines, Layout::Legacy, source));
        };

        let meta: Value = serde_json::from_slice(&bytes).map_err(|error| format!("{}: {error}", files::META))?;
        let pack: &Value = meta.get("pack").ok_or_else(|| format!("{} has no pack section", files::META))?;
        let mut text: String = String::new();
        flatten(pack.get("description").unwrap_or(&Value::Null), &mut text);

        let lines: Vec<&str> = text.lines().take(LINES).collect();
        let filters: Vec<Filter> = meta.pointer("/filter/block").map_or_else(Vec::new, |block| filters(&name, block));
        Ok(Self { m_filters: filters, ..Self::new(&name, &lines, Layout::Modern, source) })
    }

    pub fn name(&self) -> &str { &self.m_name }

    pub fn description(&self) -> &[String] { &self.m_description }

    pub fn layout(&self) -> Layout { self.m_layout }

    pub fn icon(&self) -> Option<Vec<u8>> { self.m_source.borrow_mut().read(files::ICON) }

    pub(crate) fn read(&self, location: &Location) -> Option<Vec<u8>>
    {
        let file: String = self.file(location)?;
        self.m_source.borrow_mut().read(&file)
    }

    // a file on disk for players that open paths so zip entries are unpacked to the cache first
    pub(crate) fn disk(&self, location: &Location) -> Option<PathBuf>
    {
        let file: String = self.file(location)?;
        if !inside(&file)
        {
            warning!("Texture pack {} names a file outside the pack: {file}", self.m_name);
            return None;
        }

        match &mut *self.m_source.borrow_mut()
        {
            Source::Folder(root) => Some(root.join(&file)).filter(|path| path.is_file()),
            source @ Source::Archive(..) =>
            {
                let bytes: Vec<u8> = source.read(&file)?;
                let target: PathBuf = jvm::data_dir::root().join(CACHE).join(&self.m_name).join(&file);
                if std::fs::read(&target).is_ok_and(|cached| cached == bytes)
                {
                    return Some(target);
                }

                if !self.m_unpacked.replace(true)
                {
                    info!(
                        "Unpacking texture pack {} files to the cache, the first load may take a moment",
                        self.m_name
                    );
                }

                debug!("Unpacking {file} from texture pack {}", self.m_name);
                let written: std::io::Result<()> = target
                    .parent()
                    .map_or(Ok(()), std::fs::create_dir_all)
                    .and_then(|()| std::fs::write(&target, bytes));
                match written
                {
                    Ok(()) => Some(target),

                    Err(error) =>
                    {
                        warning!("Texture pack {} file {file} could not be unpacked: {error}", self.m_name);
                        None
                    }
                }
            }
        }
    }

    // true when this pack hides the location in the packs below it
    pub(crate) fn blocks(&self, location: &Location) -> bool
    {
        self.m_filters.iter().any(|filter| filter.blocks(location))
    }

    pub(crate) fn has(&self, location: &Location) -> bool
    {
        self.file(location).is_some_and(|file| self.m_source.borrow().contains(&file))
    }

    // where this pack keeps a location since a beta texture pack only holds textures
    fn file(&self, location: &Location) -> Option<String>
    {
        match self.m_layout
        {
            Layout::Modern => Some(location.file()),
            Layout::Jar => legacy::path(location).map(str::to_owned),
            Layout::Legacy => legacy::path(location).filter(|path| path.ends_with(TEXTURE)).map(str::to_owned),
        }
    }
}

// beta split pack.txt on any line break and dropped a trailing empty line
fn legacyText(bytes: &[u8]) -> String { String::from_utf8_lossy(bytes).replace("\r\n", "\n").replace('\r', "\n") }

fn truncate(line: &str) -> String
{
    let units: Vec<u16> = line.encode_utf16().collect();
    if units.len() <= LINE_LIMIT
    {
        return line.to_owned();
    }

    String::from_utf16_lossy(&units[..LINE_LIMIT])
}

// slash separated parts with no empty dot or parent part and no drive or stream colon
fn inside(file: &str) -> bool
{
    !file.contains([':', '\0', '\\']) && file.split('/').all(|part| !part.is_empty() && part != "." && part != "..")
}

// java patterns match the whole text so each one is anchored
fn pattern(pack: &str, value: Option<&Value>) -> Option<Option<Regex>>
{
    let Some(text) = value.and_then(Value::as_str)
    else
    {
        return Some(None);
    };

    match Regex::new(&format!("^(?:{text})$"))
    {
        Ok(regex) => Some(Some(regex)),

        Err(error) =>
        {
            warning!("Texture pack {pack} filter pattern {text} is ignored: {error}");
            None
        }
    }
}

fn filters(pack: &str, block: &Value) -> Vec<Filter>
{
    let entries: &[Value] = block.as_array().map_or(&[], Vec::as_slice);
    entries
        .iter()
        .filter_map(|entry| -> Option<Filter>
        {
            let namespace: Option<Regex> = pattern(pack, entry.get("namespace"))?;
            let path: Option<Regex> = pattern(pack, entry.get("path"))?;
            Some(Filter { m_namespace: namespace, m_path: path })
        })
        .collect()
}

// a modern description is a string or a text component tree
fn flatten(value: &Value, text: &mut String)
{
    match value
    {
        Value::String(part) => text.push_str(part),
        Value::Number(number) => text.push_str(&number.to_string()),
        Value::Bool(flag) => text.push_str(&flag.to_string()),
        Value::Array(parts) => parts.iter().for_each(|part| flatten(part, text)),
        Value::Object(component) =>
        {
            for key in ["text", "extra"]
            {
                if let Some(part) = component.get(key)
                {
                    flatten(part, text);
                }
            }
        }

        Value::Null =>
        {}
    }
}
