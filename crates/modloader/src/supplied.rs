use crate::paths::LoaderPaths;
use game::resources::Source;
use mod_assets::Outcome;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;

const CACHE_DIR: &str = "assets";

thread_local!
{
    // mods whose converted cache is current, by id
    static READY: RefCell<HashMap<String, PathBuf>> = RefCell::new(HashMap::new());
}

// the cache a mod's converted art is mounted from, below the mod's own pack
pub fn pack(modId: &str) -> Option<PathBuf> { READY.with(|ready| ready.borrow().get(modId).cloned()) }

// converts once per download and recipe, a missing download leaves the mod running without that art
pub fn prepare(paths: &LoaderPaths, modId: &str, source: &mut Source) -> Result<(), String>
{
    let Some(bytes) = source.read(mod_assets::RECIPE_FILE)
    else
    {
        return Ok(());
    };

    let recipe: String = String::from_utf8(bytes).map_err(|_| format!("{} is not UTF-8", mod_assets::RECIPE_FILE))?;
    let input: PathBuf = mod_assets::inputDir(&paths.m_mods, modId);
    let cache: PathBuf = paths.m_cache.join(CACHE_DIR).join(modId);

    let outcome: Outcome =
        mod_assets::install(&recipe, modId, &input, &cache).map_err(|reason| format!("assets not loaded: {reason}"))?;
    if let Outcome::Converted(count) = outcome
    {
        crate::log::info(modId, &format!("converted {count} asset files into {}", cache.display()));
    }

    loadLang(modId, &cache);
    READY.with(|ready| ready.borrow_mut().insert(modId.to_owned(), cache));
    Ok(())
}

// converted lang loads first so the mod's own lang file can override it
fn loadLang(modId: &str, cache: &std::path::Path)
{
    let path: PathBuf = cache.join("assets").join(modId).join(crate::content::lang::LANG_FILE);
    match std::fs::read_to_string(&path)
    {
        Ok(text) => crate::content::lang::load(modId, &text),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound =>
        {}
        Err(error) => crate::log::warn(modId, &format!("{}: {error}", path.display())),
    }
}
