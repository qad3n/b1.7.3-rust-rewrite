use game::resources::{Location, MINECRAFT};
use jvm::warning;
use serde_json::{Map, Value};
use std::path::PathBuf;

const SOUNDS_JSON: &str = "sounds.json";
const SOUNDS: &str = "sounds/";
const OGG: &str = ".ogg";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pool
{
    Sounds,
    Streaming,
    Music,
}

struct Folder
{
    m_pool: Pool,
    // where a modern pack keeps the files of this pool under sounds
    m_folder: &'static str,
    // the sounds.json event prefix that adds to this pool
    m_event: &'static str,
}

// the sounds pool takes every event without a prefix so it is last
const FOLDERS: &[Folder] = &[
    Folder { m_pool: Pool::Streaming, m_folder: "records/", m_event: "records." },
    Folder { m_pool: Pool::Music, m_folder: "music/", m_event: "music." },
    Folder { m_pool: Pool::Sounds, m_folder: "", m_event: "" },
];

fn folder(pool: Pool) -> &'static Folder
{
    FOLDERS.iter().find(|folder| folder.m_pool == pool).expect("every pool has a folder row")
}

// one file the sound pools hold with the identifier the sound system caches it by
pub struct Install
{
    pub m_pool: Pool,
    pub m_path: String,
    pub m_file: PathBuf,
    // files from the resources folder that a texture pack may replace
    pub m_vanilla: bool,
}

pub struct Sound
{
    pub m_name: String,
    pub m_file: PathBuf,
}

// a sounds.json event merged over every texture pack
pub struct Event
{
    pub m_pool: Pool,
    pub m_key: String,
    pub m_replace: bool,
    pub m_sounds: Vec<Sound>,
}

// the texture pack file standing in for a vanilla file or the file itself
pub fn resolve(install: &Install) -> Sound
{
    let location: Location =
        Location::minecraft(&format!("{SOUNDS}{}{}", folder(install.m_pool).m_folder, install.m_path));
    match game::resources::disk(&location).filter(|_| install.m_vanilla)
    {
        // a new identifier keeps the sound system from replaying the vanilla buffer it cached
        Some((pack, file)) => Sound { m_name: format!("{pack}/{}", location.m_path), m_file: file },
        None => Sound { m_name: install.m_path.clone(), m_file: install.m_file.clone() },
    }
}

// a sounds.json name is namespace:path or a minecraft path without the ogg extension
fn sound(name: &str) -> Option<Sound>
{
    let (namespace, path): (&str, &str) = name.split_once(':').unwrap_or((MINECRAFT, name));
    let location: Location = Location::new(namespace, &format!("{SOUNDS}{path}{OGG}"));
    let Some((pack, file)) = game::resources::disk(&location)
    else
    {
        warning!("Texture pack sound {name} is missing");
        return None;
    };

    Some(Sound { m_name: format!("{pack}/{namespace}/{path}{OGG}"), m_file: file })
}

// a sounds list entry is a bare name or an object with name weight and type
fn entry(value: &Value, sounds: &mut Vec<Sound>)
{
    let (name, weight): (Option<&str>, i64) = match value
    {
        Value::String(name) => (Some(name), 1),
        Value::Object(entry) =>
        {
            if entry.get("type").and_then(Value::as_str) == Some("event")
            {
                warning!("Texture pack sound events that point at other events are not supported");
                return;
            }

            (entry.get("name").and_then(Value::as_str), entry.get("weight").and_then(Value::as_i64).unwrap_or(1))
        }

        _ => (None, 0),
    };

    let Some(found) = name.and_then(sound)
    else
    {
        return;
    };

    // a weight repeats the file in the pool since beta picks entries evenly
    for _ in 1..weight.max(1)
    {
        sounds.push(Sound { m_name: found.m_name.clone(), m_file: found.m_file.clone() });
    }

    sounds.push(found);
}

fn pool(event: &str) -> (Pool, String)
{
    let row: &Folder = FOLDERS.iter().find(|row| event.starts_with(row.m_event)).expect("the sounds row has no prefix");
    (row.m_pool, event[row.m_event.len()..].to_owned())
}

// sounds.json copies merge lowest first where an event adds to the one below unless it sets replace
pub fn events() -> Vec<Event>
{
    let mut merged: Vec<Event> = Vec::new();

    for bytes in game::resources::readAll(&Location::minecraft(SOUNDS_JSON))
    {
        let events: Map<String, Value> = match serde_json::from_slice(&bytes)
        {
            Ok(Value::Object(events)) => events,

            Ok(_) | Err(_) =>
            {
                warning!("Skipping a texture pack {SOUNDS_JSON} that is not a json object");
                continue;
            }
        };

        for (name, definition) in &events
        {
            let replace: bool = definition.get("replace").and_then(Value::as_bool).unwrap_or(false);
            let (pool, key): (Pool, String) = pool(name);
            let index: usize = match merged.iter().position(|event| event.m_pool == pool && event.m_key == key)
            {
                Some(index) => index,
                None =>
                {
                    merged.push(Event { m_pool: pool, m_key: key, m_replace: false, m_sounds: Vec::new() });
                    merged.len() - 1
                }
            };

            let event: &mut Event = &mut merged[index];
            if replace
            {
                event.m_replace = true;
                event.m_sounds.clear();
            }

            let list: &[Value] = definition.get("sounds").and_then(Value::as_array).map_or(&[], Vec::as_slice);
            for value in list
            {
                entry(value, &mut event.m_sounds);
            }
        }
    }

    merged
}
