use game::damage::DamageKind;
use mod_protocol::Identifier;
use std::cell::Cell;

thread_local! {
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

pub fn current() -> Option<Identifier> { CURRENT.with(Cell::get).map(|kind| Identifier::minecraft(kind.name())) }

pub fn kind(id: &Identifier) -> Option<DamageKind>
{
    if !id.isVanilla()
    {
        return None;
    }

    DamageKind::byName(id.path())
}
