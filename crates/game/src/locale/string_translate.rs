use crate::resources::Location;
use jvm::io::ByteArrayInputStream;
use jvm::properties::Properties;
use jvm::warning;
use serde_json::{Map, Value};
use std::cell::RefCell;
use std::rc::Rc;

thread_local!
{
    static INSTANCE: RefCell<Option<Rc<StringTranslate>>> = const { RefCell::new(None) };
}

// the beta files load in this order and a modern json file goes on top within each texture pack
const LANG_FILES: [&str; 2] = ["/lang/en_US.lang", "/lang/stats_US.lang"];
const LANG_JSON: &str = "lang/en_us.json";

// rewrite options the vanilla lang files never had
const REWRITE_KEYS: [(&str, &str); 2] = [("options.vsync", "Use VSync"), ("options.fullscreen", "Fullscreen")];

pub struct StringTranslate
{
    m_translateTable: Properties,
    // keys from texture packs the player picked that win over mod lang files too
    m_packTable: Properties,
}

fn loadJson(table: &mut Properties, bytes: &[u8])
{
    match serde_json::from_slice::<Map<String, Value>>(bytes)
    {
        Ok(entries) =>
        {
            for (key, text) in entries.iter().filter_map(|(key, value)| Some((key, value.as_str()?)))
            {
                table.setProperty(key, text);
            }
        }

        Err(error) => warning!("Skipping a texture pack {LANG_JSON}: {error}"),
    }
}

fn locations(namespace: &str) -> Vec<Location>
{
    let beta = |path: &&str| Location::new(namespace, &crate::resources::betaLocation(path).m_path);
    LANG_FILES.iter().map(beta).chain(std::iter::once(Location::new(namespace, LANG_JSON))).collect()
}

fn loadCopies(table: &mut Properties, copies: Vec<(usize, Vec<u8>)>) -> jvm::io::IoResult<()>
{
    for (which, bytes) in copies
    {
        // each namespace lists the beta files then the json file
        match LANG_FILES.get(which % (LANG_FILES.len() + 1))
        {
            Some(_) => table.load(&mut ByteArrayInputStream::new(bytes))?,
            None => loadJson(table, &bytes),
        }
    }

    Ok(())
}

// texture pack keys in every namespace since lang keys share one table
fn loadPackTable(table: &mut Properties)
{
    let all: Vec<Location> = crate::resources::namespaces().iter().flat_map(|namespace| locations(namespace)).collect();

    if let Err(error) = loadCopies(table, crate::resources::readUser(&all))
    {
        error.printStackTrace();
    }
}

// every texture pack copy lowest first so a higher pack replaces single keys
fn loadTables(table: &mut Properties) -> jvm::io::IoResult<()>
{
    let copies: Vec<(usize, Vec<u8>)> = crate::resources::readEach(&locations(crate::resources::MINECRAFT));

    // beta threw when a jar lang file was missing
    for which in 0..LANG_FILES.len()
    {
        if !copies.iter().any(|(file, _)| *file == which)
        {
            jvm::throwable::npe();
        }
    }

    loadCopies(table, copies)
}

impl StringTranslate
{
    fn new() -> Self
    {
        let mut table: Properties = Properties::new();
        if let Err(error) = loadTables(&mut table)
        {
            error.printStackTrace();
        }

        for (key, text) in REWRITE_KEYS
        {
            if table.getPropertyOrNull(key).is_none()
            {
                table.setProperty(key, text);
            }
        }

        let mut packTable: Properties = Properties::new();
        loadPackTable(&mut packTable);
        Self { m_translateTable: table, m_packTable: packTable }
    }

    pub fn getInstance() -> Rc<Self>
    {
        INSTANCE.with(|instance| Rc::clone(instance.borrow_mut().get_or_insert_with(|| Rc::new(Self::new()))))
    }

    // texture packs changed so the next lookup loads the tables again
    pub fn reload() { INSTANCE.with(|instance| *instance.borrow_mut() = None); }

    // texture pack keys then mod lang files then the vanilla table
    fn lookup(&self, key: &str) -> Option<String>
    {
        self.m_packTable
            .getPropertyOrNull(key)
            .or_else(|| crate::hooks::translate(key))
            .or_else(|| self.m_translateTable.getPropertyOrNull(key))
    }

    pub fn translateKey(&self, key: &str) -> String { self.lookup(key).unwrap_or_else(|| key.to_owned()) }

    pub fn translateKeyFormat(&self, key: &str, args: &[String]) -> String
    {
        let string: String = self.lookup(key).unwrap_or_else(|| key.to_owned());
        jvm::format::format(&string, args)
    }

    pub fn translateNamedKey(&self, key: &str) -> String { self.lookup(&format!("{key}.name")).unwrap_or_default() }
}
