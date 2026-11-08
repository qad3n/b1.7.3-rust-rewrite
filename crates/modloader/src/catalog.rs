use serde::Deserialize;
use std::sync::LazyLock;

const CATALOG_TOML: &str = include_str!("../hooks.toml");

#[derive(Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum HookKind
{
    Event,
    Component,
    // save and load sites where the loader names blocks and items and keeps per mod data
    Storage,
}

#[derive(Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum HookSide
{
    Both,
    Client,
    Server,
}

#[derive(Deserialize, Clone, Debug)]
pub struct HookEntry
{
    pub name: String,
    pub version: u32,
    pub kind: HookKind,
    pub side: HookSide,
    pub cancellable: bool,
    pub result: bool,
    pub fires_in: String,
}

#[derive(Deserialize)]
struct CatalogFile
{
    hook: Vec<HookEntry>,
}

static CATALOG: LazyLock<Vec<HookEntry>> = LazyLock::new(|| {
    let file: CatalogFile = toml::from_str(CATALOG_TOML).expect("the bundled hook catalog parses");
    file.hook
});

pub fn hooks() -> &'static [HookEntry] { &CATALOG }

pub fn find(name: &str, version: u32) -> Option<&'static HookEntry>
{
    hooks().iter().find(|entry| entry.name == name && entry.version == version)
}

#[cfg(test)]
mod tests
{
    #[test]
    fn catalogNamesAreUnique()
    {
        let mut seen: std::collections::HashSet<(&str, u32)> = std::collections::HashSet::new();
        for entry in super::hooks()
        {
            assert!(seen.insert((entry.name.as_str(), entry.version)), "duplicate hook {}", entry.name);
        }
        assert!(super::find("block_break", 1).is_some_and(|entry| entry.cancellable));
    }
}
