use mod_protocol::abi::{ConfigEntry, ConfigScope};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

// one config/<modid>.toml per mod, defaults come from the mod and edits survive restarts
pub struct ModConfig
{
    m_path: PathBuf,
    m_entries: Vec<ConfigEntry>,
    m_values: BTreeMap<String, String>,
    // server scope values received from a server win over local ones while connected
    m_synced: BTreeMap<String, String>,
}

fn valueText(value: &toml::Value) -> String
{
    match value
    {
        toml::Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

// numbers and booleans stay bare in the file, everything else is quoted
fn literal(text: &str) -> String
{
    let parsed: Option<toml::Value> =
        toml::from_str::<toml::Table>(&format!("v = {text}")).ok().and_then(|table| table.get("v").cloned());
    match parsed
    {
        Some(toml::Value::Integer(_) | toml::Value::Float(_) | toml::Value::Boolean(_)) => text.to_owned(),
        _ => toml::Value::String(text.to_owned()).to_string(),
    }
}

impl ModConfig
{
    pub fn new(configDir: &Path, modId: &str) -> ModConfig
    {
        ModConfig {
            m_path: configDir.join(format!("{modId}.toml")),
            m_entries: Vec::new(),
            m_values: BTreeMap::new(),
            m_synced: BTreeMap::new(),
        }
    }

    pub fn define(&mut self, entries: Vec<ConfigEntry>) -> std::io::Result<()>
    {
        let existing: toml::Table =
            std::fs::read_to_string(&self.m_path).ok().and_then(|text| toml::from_str(&text).ok()).unwrap_or_default();
        for entry in entries
        {
            let value: String = existing.get(&entry.key).map(valueText).unwrap_or_else(|| entry.default.clone());
            self.m_values.insert(entry.key.clone(), value);
            self.m_entries.retain(|known| known.key != entry.key);
            self.m_entries.push(entry);
        }
        self.save()
    }

    pub fn save(&self) -> std::io::Result<()>
    {
        let mut text: String = String::new();
        for entry in &self.m_entries
        {
            if !entry.comment.is_empty()
            {
                text.push_str(&format!("# {}\n", entry.comment));
            }
            text.push_str(&format!("# scope {:?}, default {}\n", entry.scope, entry.default));
            let value: &str = self.m_values.get(&entry.key).map(String::as_str).unwrap_or(&entry.default);
            text.push_str(&format!("{} = {}\n\n", entry.key, literal(value)));
        }
        std::fs::write(&self.m_path, text)
    }

    pub fn value(&self, key: &str) -> Option<String>
    {
        self.m_synced.get(key).or_else(|| self.m_values.get(key)).cloned()
    }

    pub fn set(&mut self, key: &str, value: &str) -> std::io::Result<()>
    {
        self.m_values.insert(key.to_owned(), value.to_owned());
        self.save()
    }

    pub fn entries(&self) -> &[ConfigEntry] { &self.m_entries }

    pub fn serverValues(&self) -> BTreeMap<String, String>
    {
        self.m_entries
            .iter()
            .filter(|entry| entry.scope == ConfigScope::Server)
            .filter_map(|entry| self.m_values.get(&entry.key).map(|value| (entry.key.clone(), value.clone())))
            .collect()
    }

    pub fn applySynced(&mut self, values: BTreeMap<String, String>) { self.m_synced = values; }

    pub fn clearSynced(&mut self) { self.m_synced.clear(); }
}
