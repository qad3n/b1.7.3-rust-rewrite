use mod_protocol::handshake::ModSide;
use serde::Deserialize;
use std::collections::BTreeMap;

pub const MANIFEST_FILE: &str = "mod.toml";
pub const WASM_FILE: &str = "mod.wasm";

#[derive(Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub enum SideSpec
{
    #[default]
    Both,
    Client,
    Server,
}

impl SideSpec
{
    pub fn wire(self) -> ModSide
    {
        match self
        {
            Self::Both => ModSide::Both,
            Self::Client => ModSide::Client,
            Self::Server => ModSide::Server,
        }
    }
}

#[derive(Deserialize, Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct NetworkSpec
{
    #[serde(default = "firstVersion")]
    pub version: String,
    pub required: Option<bool>,
}

fn firstVersion() -> String { "1".to_owned() }

#[derive(Deserialize, Clone, Debug, Default)]
#[serde(default, deny_unknown_fields)]
pub struct LoadOrder
{
    pub after: Vec<String>,
    pub before: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Manifest
{
    pub id: String,
    pub version: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub license: String,
    pub api: String,
    #[serde(default)]
    pub side: SideSpec,
    #[serde(default)]
    pub network: NetworkSpec,
    #[serde(default)]
    pub depends: BTreeMap<String, String>,
    #[serde(default)]
    pub optional: BTreeMap<String, String>,
    #[serde(default)]
    pub breaks: BTreeMap<String, String>,
    #[serde(default)]
    pub provides: Vec<String>,
    #[serde(default)]
    pub load: LoadOrder,
    // renamed content, old path to new path in this namespace
    #[serde(default)]
    pub remap: BTreeMap<String, String>,
}

impl Manifest
{
    pub fn parse(text: &str) -> Result<Self, String> { toml::from_str(text).map_err(|error| error.to_string()) }

    pub fn displayName(&self) -> &str
    {
        if self.name.is_empty()
        {
            &self.id
        }
        else
        {
            &self.name
        }
    }
}
