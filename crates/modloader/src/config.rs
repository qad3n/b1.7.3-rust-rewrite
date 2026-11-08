use crate::state::ModRuntime;
use mod_protocol::abi::{ConfigEntry, ConfigKind, ConfigOption, ConfigScope, GuestCall, Lifecycle};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const BACKUP_EXTENSION: &str = "toml.bak";

// one config/<modid>.toml per mod, defaults come from the mod and edits survive restarts
pub struct ModConfig
{
    m_modId: String,
    m_path: PathBuf,
    m_options: Vec<ConfigOption>,
    // canonical value text per key, checked against the option kind
    m_values: BTreeMap<String, String>,
    // server scope values received from a server win over local ones while connected
    m_synced: BTreeMap<String, String>,
}

fn validName(name: &str) -> bool
{
    !name.is_empty() && name.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn validKey(key: &str) -> bool
{
    match key.split_once('.')
    {
        Some((category, name)) => validName(category) && validName(name),
        None => validName(key),
    }
}

pub fn category(option: &ConfigOption) -> Option<&str> { option.key.split_once('.').map(|(category, _)| category) }

fn name(option: &ConfigOption) -> &str { option.key.split_once('.').map_or(option.key.as_str(), |(_, name)| name) }

pub fn humanize(name: &str) -> String
{
    let spaced: String = name.replace('_', " ");
    let mut chars: std::str::Chars<'_> = spaced.chars();
    chars.next().map(|first| first.to_uppercase().chain(chars).collect()).unwrap_or_default()
}

pub fn label(option: &ConfigOption) -> String
{
    if option.label.is_empty()
    {
        humanize(name(option))
    }
    else
    {
        option.label.clone()
    }
}

fn kindText(kind: &ConfigKind) -> String
{
    match kind
    {
        ConfigKind::Bool { .. } => "true or false".to_owned(),
        ConfigKind::Int { min: i64::MIN, max: i64::MAX, .. } => "whole number".to_owned(),
        ConfigKind::Int { min, max, .. } => format!("whole number {min} to {max}"),
        ConfigKind::Float { min, max, .. } if *min == f64::MIN && *max == f64::MAX => "number".to_owned(),
        ConfigKind::Float { min, max, .. } => format!("number {min} to {max}"),
        ConfigKind::Text { .. } => "text".to_owned(),
        ConfigKind::Choice { choices, .. } => format!("one of {}", choices.join(", ")),
    }
}

pub fn describe(option: &ConfigOption) -> String
{
    let mut parts: Vec<String> = vec![kindText(&option.kind), format!("default {}", option.kind.defaultText())];
    if option.scope == ConfigScope::Server
    {
        parts.push("set by the server while connected".to_owned());
    }
    if option.restart
    {
        parts.push("needs a restart".to_owned());
    }
    parts.join(", ")
}

// define without types: true and false become toggles, whole numbers and numbers get number controls
fn legacyKind(default: &str) -> ConfigKind
{
    if let Ok(value) = default.parse::<bool>()
    {
        return ConfigKind::Bool { default: value };
    }
    if let Ok(value) = default.parse::<i64>()
    {
        return ConfigKind::Int { default: value, min: i64::MIN, max: i64::MAX };
    }
    match default.parse::<f64>()
    {
        Ok(value) if value.is_finite() => ConfigKind::Float { default: value, min: f64::MIN, max: f64::MAX },
        _ => ConfigKind::Text { default: default.to_owned() },
    }
}

fn legacyOption(entry: ConfigEntry) -> ConfigOption
{
    ConfigOption {
        kind: legacyKind(&entry.default),
        key: entry.key,
        label: String::new(),
        comment: entry.comment,
        scope: entry.scope,
        restart: false,
    }
}

fn checkOption(option: &ConfigOption) -> Result<(), String>
{
    if !validKey(&option.key)
    {
        return Err(format!("option key '{}' must be name or category.name with [a-z0-9_]", option.key));
    }
    match &option.kind
    {
        ConfigKind::Int { min, max, .. } if min > max => Err(format!("{}: min {min} is above max {max}", option.key)),
        ConfigKind::Float { min, max, .. } if !(min <= max) =>
        {
            Err(format!("{}: min {min} is above max {max}", option.key))
        }
        ConfigKind::Choice { choices, .. } if choices.is_empty() =>
        {
            Err(format!("{}: a choice needs options", option.key))
        }
        kind =>
        {
            kind.check(&kind.defaultText()).map(|_| ()).map_err(|reason| format!("{} default: {reason}", option.key))
        }
    }
}

fn lookup<'a>(table: &'a toml::Table, key: &str) -> Option<&'a toml::Value>
{
    match key.split_once('.')
    {
        Some((category, name)) => table.get(category)?.as_table()?.get(name),
        None => table.get(key),
    }
}

fn valueText(value: &toml::Value) -> String
{
    match value
    {
        toml::Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

fn literal(kind: &ConfigKind, value: &str) -> String
{
    match kind
    {
        ConfigKind::Bool { .. } | ConfigKind::Int { .. } => value.to_owned(),
        ConfigKind::Float { .. } =>
        {
            value.parse::<f64>().map_or_else(|_| value.to_owned(), |number| toml::Value::Float(number).to_string())
        }
        ConfigKind::Text { .. } | ConfigKind::Choice { .. } => toml::Value::String(value.to_owned()).to_string(),
    }
}

impl ModConfig
{
    pub fn new(configDir: &Path, modId: &str) -> ModConfig
    {
        ModConfig {
            m_modId: modId.to_owned(),
            m_path: configDir.join(format!("{modId}.toml")),
            m_options: Vec::new(),
            m_values: BTreeMap::new(),
            m_synced: BTreeMap::new(),
        }
    }

    pub fn fileName(&self) -> String { format!("{}/{}.toml", crate::paths::CONFIG_DIR, self.m_modId) }

    pub fn define(&mut self, entries: Vec<ConfigEntry>) -> Result<(), String>
    {
        self.defineOptions(entries.into_iter().map(legacyOption).collect())
    }

    pub fn defineOptions(&mut self, options: Vec<ConfigOption>) -> Result<(), String>
    {
        for option in &options
        {
            checkOption(option)?;
        }
        let mut merged: Vec<ConfigOption> = self.m_options.clone();
        for option in &options
        {
            merged.retain(|known| known.key != option.key);
            merged.push(option.clone());
        }
        if let Some(clash) =
            merged.iter().find(|option| merged.iter().any(|other| category(other) == Some(&option.key)))
        {
            return Err(format!("'{}' is both an option and a category", clash.key));
        }
        let existing: toml::Table = self.read();
        for option in &options
        {
            let value: String = match lookup(&existing, &option.key).map(valueText)
            {
                Some(text) => option.kind.check(&text).unwrap_or_else(|reason| {
                    crate::log::warn(
                        &self.m_modId,
                        &format!("{} {}: {reason}, using the default", self.fileName(), option.key),
                    );
                    option.kind.defaultText()
                }),
                None => option.kind.defaultText(),
            };
            self.m_values.insert(option.key.clone(), value);
        }
        self.m_options = merged;
        self.save()
    }

    // a file that is not valid toml is kept as a backup before defaults replace it
    fn read(&self) -> toml::Table
    {
        let text: String = match std::fs::read_to_string(&self.m_path)
        {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return toml::Table::new(),
            Err(error) =>
            {
                crate::log::warn(&self.m_modId, &format!("could not read {}: {error}", self.fileName()));
                return toml::Table::new();
            }
        };
        toml::from_str(&text).unwrap_or_else(|error| {
            let backup: PathBuf = self.m_path.with_extension(BACKUP_EXTENSION);
            crate::log::error(
                &self.m_modId,
                &format!("{} is not valid toml, saved as {} and reset: {error}", self.fileName(), backup.display()),
            );
            if let Err(error) = std::fs::copy(&self.m_path, &backup)
            {
                crate::log::error(&self.m_modId, &format!("could not back up {}: {error}", self.fileName()));
            }
            toml::Table::new()
        })
    }

    fn optionText(&self, option: &ConfigOption) -> String
    {
        let mut text: String = String::new();
        if !option.comment.is_empty()
        {
            text.push_str(&format!("# {}\n", option.comment));
        }
        text.push_str(&format!("# {}\n", describe(option)));
        let value: String = self.m_values.get(&option.key).cloned().unwrap_or_else(|| option.kind.defaultText());
        text.push_str(&format!("{} = {}\n\n", name(option), literal(&option.kind, &value)));
        text
    }

    fn render(&self) -> String
    {
        let mut text: String = format!("# {} config, also editable in game from the mod menu\n\n", self.m_modId);
        for option in self.m_options.iter().filter(|option| category(option).is_none())
        {
            text.push_str(&self.optionText(option));
        }
        let mut categories: Vec<&str> = Vec::new();
        for found in self.m_options.iter().filter_map(category)
        {
            if !categories.contains(&found)
            {
                categories.push(found);
            }
        }
        for current in categories
        {
            text.push_str(&format!("[{current}]\n\n"));
            for option in self.m_options.iter().filter(|option| category(option) == Some(current))
            {
                text.push_str(&self.optionText(option));
            }
        }
        text
    }

    fn save(&self) -> Result<(), String>
    {
        std::fs::write(&self.m_path, self.render())
            .map_err(|error| format!("could not write {}: {error}", self.fileName()))
    }

    pub fn value(&self, key: &str) -> Option<String>
    {
        self.m_synced.get(key).or_else(|| self.m_values.get(key)).cloned()
    }

    // true when the stored value changed
    pub fn set(&mut self, key: &str, value: &str) -> Result<bool, String>
    {
        let option: &ConfigOption =
            self.m_options.iter().find(|option| option.key == key).ok_or_else(|| format!("unknown option {key}"))?;
        let canonical: String = option.kind.check(value)?;
        if self.m_values.get(key) == Some(&canonical)
        {
            return Ok(false);
        }
        self.m_values.insert(key.to_owned(), canonical);
        self.save().map(|()| true)
    }

    pub fn options(&self) -> &[ConfigOption] { &self.m_options }

    pub fn serverValues(&self) -> BTreeMap<String, String>
    {
        self.m_options
            .iter()
            .filter(|option| option.scope == ConfigScope::Server)
            .filter_map(|option| self.m_values.get(&option.key).map(|value| (option.key.clone(), value.clone())))
            .collect()
    }

    pub fn applySynced(&mut self, values: BTreeMap<String, String>) { self.m_synced = values; }

    pub fn clearSynced(&mut self) { self.m_synced.clear(); }
}

// a change from the mod menu, the owning mod hears about it once the file is saved
pub fn change(runtime: &ModRuntime, key: &str, value: &str) -> Result<(), String>
{
    let changed: bool = runtime.m_config.borrow_mut().set(key, value)?;
    if changed
    {
        runtime.call(&GuestCall::Lifecycle(Lifecycle::ConfigChanged(key.to_owned())));
    }
    Ok(())
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn tempDir(name: &str) -> PathBuf
    {
        let dir: PathBuf = std::env::temp_dir().join(format!("b173-config-{}-{name}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir is writable");
        dir
    }

    fn option(key: &str, kind: ConfigKind) -> ConfigOption
    {
        ConfigOption {
            key: key.to_owned(),
            label: String::new(),
            comment: "a comment".to_owned(),
            scope: ConfigScope::Common,
            kind,
            restart: false,
        }
    }

    fn sample() -> Vec<ConfigOption>
    {
        vec![
            option("enabled", ConfigKind::Bool { default: true }),
            option("spawn.rate", ConfigKind::Int { default: 65, min: 1, max: 1000 }),
            option("spawn.scale", ConfigKind::Float { default: 1.0, min: 0.0, max: 2.0 }),
            option(
                "hud.corner",
                ConfigKind::Choice { default: "left".to_owned(), choices: vec!["left".to_owned(), "right".to_owned()] },
            ),
            option("hud.title", ConfigKind::Text { default: "hello".to_owned() }),
        ]
    }

    #[test]
    fn savedValuesLoadAgain()
    {
        let dir: PathBuf = tempDir("round");
        let mut config: ModConfig = ModConfig::new(&dir, "sample");
        config.defineOptions(sample()).expect("sample options are valid");
        assert_eq!(config.set("spawn.rate", "10"), Ok(true));
        assert_eq!(config.set("spawn.rate", " 10 "), Ok(false));
        assert_eq!(config.set("hud.title", "a \"quoted\" title"), Ok(true));
        let text: String = std::fs::read_to_string(dir.join("sample.toml")).expect("config was written");
        assert!(text.contains("[spawn]\n") && text.contains("rate = 10") && text.contains("scale = 1.0"));
        let mut reloaded: ModConfig = ModConfig::new(&dir, "sample");
        reloaded.defineOptions(sample()).expect("sample options are valid");
        assert_eq!(reloaded.value("spawn.rate").as_deref(), Some("10"));
        assert_eq!(reloaded.value("hud.title").as_deref(), Some("a \"quoted\" title"));
        assert_eq!(reloaded.value("enabled").as_deref(), Some("true"));
    }

    #[test]
    fn badFileValuesFallBackToDefaults()
    {
        let dir: PathBuf = tempDir("bad");
        std::fs::write(dir.join("sample.toml"), "enabled = \"maybe\"\n[spawn]\nrate = 5000\nscale = 1\n")
            .expect("temp dir is writable");
        let mut config: ModConfig = ModConfig::new(&dir, "sample");
        config.defineOptions(sample()).expect("sample options are valid");
        assert_eq!(config.value("enabled").as_deref(), Some("true"));
        assert_eq!(config.value("spawn.rate").as_deref(), Some("65"));
        assert_eq!(config.value("spawn.scale").as_deref(), Some("1"));
    }

    #[test]
    fn brokenFileIsBackedUp()
    {
        let dir: PathBuf = tempDir("broken");
        std::fs::write(dir.join("sample.toml"), "enabled = = true").expect("temp dir is writable");
        let mut config: ModConfig = ModConfig::new(&dir, "sample");
        config.defineOptions(sample()).expect("sample options are valid");
        assert!(dir.join("sample.toml.bak").is_file());
        assert_eq!(config.value("enabled").as_deref(), Some("true"));
    }

    #[test]
    fn setChecksTheKind()
    {
        let dir: PathBuf = tempDir("set");
        let mut config: ModConfig = ModConfig::new(&dir, "sample");
        config.defineOptions(sample()).expect("sample options are valid");
        assert!(config.set("enabled", "maybe").is_err());
        assert!(config.set("spawn.rate", "0").is_err());
        assert!(config.set("spawn.scale", "NaN").is_err());
        assert!(config.set("hud.corner", "middle").is_err());
        assert!(config.set("missing", "1").is_err());
        assert_eq!(config.set("enabled", "FALSE"), Ok(true));
        assert_eq!(config.value("enabled").as_deref(), Some("false"));
    }

    #[test]
    fn badOptionsAreRejected()
    {
        let dir: PathBuf = tempDir("reject");
        let mut config: ModConfig = ModConfig::new(&dir, "sample");
        assert!(config.defineOptions(vec![option("Bad Key", ConfigKind::Bool { default: true })]).is_err());
        assert!(config.defineOptions(vec![option("a.b.c", ConfigKind::Bool { default: true })]).is_err());
        assert!(config.defineOptions(vec![option("rate", ConfigKind::Int { default: 0, min: 1, max: 9 })]).is_err());
        let clash: Vec<ConfigOption> = vec![
            option("hud", ConfigKind::Bool { default: true }),
            option("hud.title", ConfigKind::Text { default: String::new() }),
        ];
        assert!(config.defineOptions(clash).is_err());
    }

    #[test]
    fn legacyDefaultsPickAKind()
    {
        assert_eq!(legacyKind("true"), ConfigKind::Bool { default: true });
        assert_eq!(legacyKind("3"), ConfigKind::Int { default: 3, min: i64::MIN, max: i64::MAX });
        assert_eq!(legacyKind("1.5"), ConfigKind::Float { default: 1.5, min: f64::MIN, max: f64::MAX });
        assert_eq!(legacyKind("hello"), ConfigKind::Text { default: "hello".to_owned() });
    }
}
