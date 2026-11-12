use crate::entity::Entity;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

const NAMESPACE: &str = "minecraft:";

// why an entity is hurt, beta passed only the attacker so these are labels and change no vanilla rule
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DamageKind
{
    Generic,
    Mob,
    Player,
    Arrow,
    Thrown,
    Fireball,
    Explosion,
    Fall,
    InFire,
    OnFire,
    Lava,
    Drown,
    Suffocate,
    Cactus,
    Void,
    Kill,
    Magic,
    // a mod damage type, the index of its name in the registered list
    Mod(u16),
}

pub struct DamageRow
{
    pub m_kind: DamageKind,
    pub m_name: &'static str,
}

pub const ROWS: [DamageRow; 17] = [
    DamageRow { m_kind: DamageKind::Generic, m_name: "generic" },
    DamageRow { m_kind: DamageKind::Mob, m_name: "mob" },
    DamageRow { m_kind: DamageKind::Player, m_name: "player" },
    DamageRow { m_kind: DamageKind::Arrow, m_name: "arrow" },
    DamageRow { m_kind: DamageKind::Thrown, m_name: "thrown" },
    DamageRow { m_kind: DamageKind::Fireball, m_name: "fireball" },
    DamageRow { m_kind: DamageKind::Explosion, m_name: "explosion" },
    DamageRow { m_kind: DamageKind::Fall, m_name: "fall" },
    DamageRow { m_kind: DamageKind::InFire, m_name: "in_fire" },
    DamageRow { m_kind: DamageKind::OnFire, m_name: "on_fire" },
    DamageRow { m_kind: DamageKind::Lava, m_name: "lava" },
    DamageRow { m_kind: DamageKind::Drown, m_name: "drown" },
    DamageRow { m_kind: DamageKind::Suffocate, m_name: "in_wall" },
    DamageRow { m_kind: DamageKind::Cactus, m_name: "cactus" },
    DamageRow { m_kind: DamageKind::Void, m_name: "out_of_world" },
    DamageRow { m_kind: DamageKind::Kill, m_name: "kill" },
    DamageRow { m_kind: DamageKind::Magic, m_name: "magic" },
];

thread_local!
{
    static MODDED: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    // the kind of the hit whose damage is being applied, so armor can ask whether the kind passes it
    static APPLYING: Cell<Option<DamageKind>> = const { Cell::new(None) };
}

const BYPASSES_ARMOR: &str = "minecraft:bypasses_armor";

// runs the damage step of a hit with its kind known, a hit inside a hit restores the outer kind
pub fn applying<R>(kind: DamageKind, body: impl FnOnce() -> R) -> R
{
    let outer: Option<DamageKind> = APPLYING.with(|current| current.replace(Some(kind)));
    let result: R = body();

    APPLYING.with(|current| current.set(outer));
    result
}

// beta armor reduces every hit, a damage type in the bypasses_armor tag goes straight through
pub fn bypassesArmor() -> bool
{
    APPLYING.with(Cell::get).is_some_and(|kind| crate::hooks::damageTagged(&kind.name(), BYPASSES_ARMOR))
}

impl DamageKind
{
    // the full registry name, minecraft:<row> for beta kinds
    pub fn name(self) -> String
    {
        match self
        {
            Self::Mod(index) =>
            {
                MODDED.with(|names| names.borrow().get(usize::from(index)).cloned()).unwrap_or_default()
            }
            kind =>
            {
                format!("{NAMESPACE}{}", ROWS.iter().find(|row| row.m_kind == kind).map_or("generic", |row| row.m_name))
            }
        }
    }

    pub fn byName(name: &str) -> Option<Self>
    {
        if let Some(path) = name.strip_prefix(NAMESPACE)
        {
            return ROWS.iter().find(|row| row.m_name == path).map(|row| row.m_kind);
        }

        MODDED
            .with(|names| names.borrow().iter().position(|known| known == name))
            .and_then(|index| u16::try_from(index).ok())
            .map(Self::Mod)
    }
}

// a mod damage type keeps its index for the session, registering a name again returns the same kind
pub fn register(name: &str) -> DamageKind
{
    if let Some(kind) = DamageKind::byName(name)
    {
        return kind;
    }

    MODDED.with(|names|
    {
        let mut names = names.borrow_mut();
        names.push(name.to_owned());
        DamageKind::Mod(u16::try_from(names.len() - 1).expect("fewer than 65536 mod damage types"))
    })
}

pub fn names() -> Vec<String> { ROWS.iter().map(|row| format!("{NAMESPACE}{}", row.m_name)).collect() }

// the attacker is who beta passed as the source, the direct entity is the arrow or ball that hit
#[derive(Clone, Copy)]
pub struct DamageSource<'a>
{
    pub m_kind: DamageKind,
    pub m_attacker: Option<&'a Rc<Entity>>,
    pub m_direct: Option<&'a Rc<Entity>>,
}

impl<'a> DamageSource<'a>
{
    pub fn new(kind: DamageKind) -> Self { DamageSource { m_kind: kind, m_attacker: None, m_direct: None } }

    pub fn withAttacker(kind: DamageKind, attacker: Option<&'a Rc<Entity>>) -> Self
    {
        DamageSource { m_kind: kind, m_attacker: attacker, m_direct: attacker }
    }

    pub fn withProjectile(kind: DamageKind, projectile: &'a Rc<Entity>, attacker: Option<&'a Rc<Entity>>) -> Self
    {
        DamageSource { m_kind: kind, m_attacker: attacker, m_direct: Some(projectile) }
    }
}
