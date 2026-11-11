use crate::config::ModConfig;
use crate::discovery::Source;
use crate::manifest::Manifest;
use crate::resolver::Problem;
use crate::wasm::ModInstance;
use mod_protocol::Identifier;
use mod_protocol::abi::{GuestCall, GuestReply};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum Phase
{
    Bootstrap,
    Register,
    Frozen,
    Running,
}

pub struct ModRuntime
{
    pub m_index: usize,
    pub m_manifest: Manifest,
    pub m_file: String,
    pub m_sha256: String,
    pub m_source: RefCell<Source>,
    pub m_instance: Option<ModInstance>,
    pub m_config: RefCell<ModConfig>,
    m_failure: RefCell<Option<String>>,
}

impl ModRuntime
{
    pub fn new(
        index: usize,
        manifest: Manifest,
        file: String,
        sha256: String,
        source: Source,
        config: ModConfig,
    ) -> Self
    {
        Self
        {
            m_index: index,
            m_manifest: manifest,
            m_file: file,
            m_sha256: sha256,
            m_source: RefCell::new(source),
            m_instance: None,
            m_config: RefCell::new(config),
            m_failure: RefCell::new(None),
        }
    }

    pub fn id(&self) -> &str { &self.m_manifest.id }

    // the loader API minor version the mod was built against, calls it cannot decode are never sent to it
    pub fn apiMinor(&self) -> u32 { crate::resolver::parseApi(&self.m_manifest.api).map_or(0, |(_, minor)| minor) }

    pub fn failure(&self) -> Option<String> { self.m_failure.borrow().clone() }

    pub fn isActive(&self) -> bool { self.m_failure.borrow().is_none() }

    // a trapped mod stops receiving calls and every caller falls back to vanilla behavior
    pub fn fail(&self, reason: String)
    {
        crate::log::error(self.id(), &format!("disabled: {reason}"));
        problem(self.id(), &self.m_file, &reason);
        *self.m_failure.borrow_mut() = Some(reason);
    }

    pub fn call(&self, call: &GuestCall) -> Option<GuestReply>
    {
        if !self.isActive()
        {
            return None;
        }

        let instance: &ModInstance = self.m_instance.as_ref()?;
        match instance.call(call)
        {
            Ok(GuestReply::Error(reason)) =>
            {
                crate::log::warn(self.id(), &reason);
                None
            }

            Ok(reply) => Some(reply),

            Err(reason) =>
            {
                self.fail(reason);
                None
            }
        }
    }

    pub(crate) fn own(&self, name: &str) -> Result<Identifier, String>
    {
        Identifier::parseIn(name, self.id()).map_err(|error| error.to_string()).and_then(|id|
        {
            if id.namespace() == self.id()
            {
                Ok(id)
            }
            else
            {
                Err(format!("{id} is outside the namespace {}", self.id()))
            }
        })
    }
}

thread_local!
{
    static MODS: RefCell<Vec<Rc<ModRuntime>>> = const { RefCell::new(Vec::new()) };
    static PHASE: Cell<Phase> = const { Cell::new(Phase::Bootstrap) };
    static PROBLEMS: RefCell<Vec<Problem>> = const { RefCell::new(Vec::new()) };
    static SKIPPED: RefCell<Vec<(String, String)>> = const { RefCell::new(Vec::new()) };
    // no mods are installed but a world saved with mods is open, the loader runs only to keep that content
    static KEEPER: Cell<bool> = const { Cell::new(false) };
}

pub fn setKeeper(keeping: bool) { KEEPER.with(|slot| slot.set(keeping)); }

pub fn isKeeper() -> bool { KEEPER.with(Cell::get) }

// a problem raised before the file has a mod id
pub const UNKNOWN_MOD: &str = "?";

pub fn setMods(mods: Vec<Rc<ModRuntime>>) { MODS.with(|list| *list.borrow_mut() = mods); }

pub fn mods() -> Vec<Rc<ModRuntime>> { MODS.with(|list| list.borrow().clone()) }

pub fn hasMods() -> bool { MODS.with(|list| !list.borrow().is_empty()) }

pub fn modAt(index: usize) -> Option<Rc<ModRuntime>> { MODS.with(|list| list.borrow().get(index).cloned()) }

pub fn byId(id: &str) -> Option<Rc<ModRuntime>>
{
    MODS.with(|list| list.borrow().iter().find(|runtime| runtime.id() == id).cloned())
}

pub fn modId(index: usize) -> String
{
    modAt(index).map(|runtime| runtime.id().to_owned()).unwrap_or_else(|| crate::log::LOADER.to_owned())
}

pub fn phase() -> Phase { PHASE.with(Cell::get) }

pub fn setPhase(phase: Phase) { PHASE.with(|slot| slot.set(phase)); }

pub fn problem(modId: &str, file: &str, reason: &str)
{
    PROBLEMS.with(|list|
    {
        list.borrow_mut().push(Problem
        {
            m_mod: modId.to_owned(),
            m_file: file.to_owned(),
            m_reason: reason.to_owned(),
        })
    });
}

pub fn problems() -> Vec<Problem> { PROBLEMS.with(|list| list.borrow().clone()) }

pub fn addProblems(found: Vec<Problem>) { PROBLEMS.with(|list| list.borrow_mut().extend(found)); }

pub fn setSkipped(skipped: Vec<(String, String)>) { SKIPPED.with(|list| *list.borrow_mut() = skipped); }

pub fn skipped() -> Vec<(String, String)> { SKIPPED.with(|list| list.borrow().clone()) }
