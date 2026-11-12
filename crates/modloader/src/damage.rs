use game::damage::DamageKind;
use mod_protocol::Identifier;
use std::cell::Cell;

thread_local!
{
    // the kind of the hit whose events and behaviors are running, so mods can ask for it
    static CURRENT: Cell<Option<DamageKind>> = const { Cell::new(None) };
}

// a hit inside a hit restores the outer kind when it finishes
pub fn with<R>(kind: DamageKind, body: impl FnOnce() -> R) -> R
{
    let outer: Option<DamageKind> = CURRENT.with(|current| current.replace(Some(kind)));
    let result: R = body();

    CURRENT.with(|current| current.set(outer));
    result
}

pub fn current() -> Option<Identifier> { CURRENT.with(Cell::get).and_then(|kind| Identifier::parse(&kind.name()).ok()) }

pub fn kind(id: &Identifier) -> Option<DamageKind> { DamageKind::byName(&id.to_string()) }

pub fn register(id: Identifier, _spec: mod_protocol::spec::DamageTypeSpec) -> Result<(), String>
{
    crate::registry::claim(mod_protocol::abi::RegistryKind::DamageType, &id)?;
    game::damage::register(&id.to_string());
    Ok(())
}
