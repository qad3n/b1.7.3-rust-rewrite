use game::damage::DamageKind;
use game::entity::{Entity, entity_player};
use std::rc::Rc;

struct DeathRow
{
    m_kind: DamageKind,
    m_alone: &'static str,
    // used when an attacker is known, the alone text otherwise
    m_by: Option<&'static str>,
}

// the generic row comes first and covers mod damage types
const DEATHS: [DeathRow; 17] = [
    DeathRow { m_kind: DamageKind::Generic, m_alone: "died", m_by: Some("was slain by") },
    DeathRow { m_kind: DamageKind::Mob, m_alone: "died", m_by: Some("was slain by") },
    DeathRow { m_kind: DamageKind::Player, m_alone: "died", m_by: Some("was slain by") },
    DeathRow { m_kind: DamageKind::Arrow, m_alone: "was shot", m_by: Some("was shot by") },
    DeathRow { m_kind: DamageKind::Thrown, m_alone: "was pummeled", m_by: Some("was pummeled by") },
    DeathRow { m_kind: DamageKind::Fireball, m_alone: "was fireballed", m_by: Some("was fireballed by") },
    DeathRow { m_kind: DamageKind::Explosion, m_alone: "blew up", m_by: Some("was blown up by") },
    DeathRow { m_kind: DamageKind::Fall, m_alone: "hit the ground too hard", m_by: None },
    DeathRow { m_kind: DamageKind::InFire, m_alone: "went up in flames", m_by: None },
    DeathRow { m_kind: DamageKind::OnFire, m_alone: "burned to death", m_by: None },
    DeathRow { m_kind: DamageKind::Lava, m_alone: "tried to swim in lava", m_by: None },
    DeathRow { m_kind: DamageKind::Drown, m_alone: "drowned", m_by: None },
    DeathRow { m_kind: DamageKind::Suffocate, m_alone: "suffocated in a wall", m_by: None },
    DeathRow { m_kind: DamageKind::Cactus, m_alone: "was pricked to death", m_by: None },
    DeathRow { m_kind: DamageKind::Void, m_alone: "fell out of the world", m_by: None },
    DeathRow { m_kind: DamageKind::Kill, m_alone: "was killed", m_by: None },
    DeathRow { m_kind: DamageKind::Magic, m_alone: "was killed by magic", m_by: Some("was killed by magic from") },
];

pub(crate) fn message(player: &Entity, kind: DamageKind, killer: Option<&Rc<Entity>>) -> String
{
    let row: &DeathRow = DEATHS.iter().find(|row| row.m_kind == kind).unwrap_or(&DEATHS[0]);
    let name: String = label(player);

    match (killer, row.m_by)
    {
        (Some(killer), Some(by)) => format!("{name} {by} {}", label(killer)),
        _ => format!("{name} {}", row.m_alone),
    }
}

fn label(entity: &Entity) -> String
{
    if entity.instanceOf("EntityPlayer")
    {
        return entity_player::fields(entity).m_username.get().map(|name| name.to_string()).unwrap_or_default();
    }

    entity.getEntityString().map_or_else(|| String::from("something"), |kind| kind.to_string())
}
