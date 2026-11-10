use crate::entity::Entity;
use std::rc::Rc;

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
}

pub struct DamageRow
{
    pub m_kind: DamageKind,
    pub m_name: &'static str,
}

pub const ROWS: [DamageRow; 16] = [
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
];

impl DamageKind
{
    pub fn name(self) -> &'static str { ROWS.iter().find(|row| row.m_kind == self).map_or("generic", |row| row.m_name) }

    pub fn byName(name: &str) -> Option<DamageKind> { ROWS.iter().find(|row| row.m_name == name).map(|row| row.m_kind) }
}

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
    pub fn new(kind: DamageKind) -> DamageSource<'a> { DamageSource { m_kind: kind, m_attacker: None, m_direct: None } }

    pub fn withAttacker(kind: DamageKind, attacker: Option<&'a Rc<Entity>>) -> DamageSource<'a>
    {
        DamageSource { m_kind: kind, m_attacker: attacker, m_direct: attacker }
    }

    pub fn withProjectile(
        kind: DamageKind,
        projectile: &'a Rc<Entity>,
        attacker: Option<&'a Rc<Entity>>,
    ) -> DamageSource<'a>
    {
        DamageSource { m_kind: kind, m_attacker: attacker, m_direct: Some(projectile) }
    }
}
