use crate::resources::{Location, Source, legacy};
use serde_json::Value;
use std::cell::RefCell;
use std::path::Path;

mod files
{
    pub const META: &str = "pack.mcmeta";
    pub const TEXT: &str = "pack.txt";
    pub const ICON: &str = "pack.png";
}

const TEXTURE: &str = ".png";

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

pub struct Pack
{
    m_name: String,
    m_description: Vec<String>,
    m_layout: Layout,
    m_source: RefCell<Source>,
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

        let (layout, text): (Layout, String) = match source.read(files::META)
        {
            Some(bytes) => (Layout::Modern, description(&bytes)?),
            None => (Layout::Legacy, source.read(files::TEXT).map_or_else(String::new, |bytes| legacyText(&bytes))),
        };
        let name: String = path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
        let lines: Vec<&str> = text.lines().take(LINES).collect();
        Ok(Self::new(&name, &lines, layout, source))
    }

    pub fn name(&self) -> &str { &self.m_name }

    pub fn description(&self) -> &[String] { &self.m_description }

    pub fn icon(&self) -> Option<Vec<u8>> { self.m_source.borrow_mut().read(files::ICON) }

    pub(crate) fn read(&self, location: &Location) -> Option<Vec<u8>>
    {
        let file: String = self.file(location)?;
        self.m_source.borrow_mut().read(&file)
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

fn description(bytes: &[u8]) -> Result<String, String>
{
    let meta: Value = serde_json::from_slice(bytes).map_err(|error| format!("{}: {error}", files::META))?;
    let pack: &Value = meta.get("pack").ok_or_else(|| format!("{} has no pack section", files::META))?;
    let mut text: String = String::new();
    flatten(pack.get("description").unwrap_or(&Value::Null), &mut text);
    Ok(text)
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
