use crate::render::particle::entity_fx::{self, FXFields};
use crate::render::particle::{entity_portal_fx, entity_slime_fx};
use crate::render::{RenderGlobal, vanillaParticle};
use game::entity::Entity;
use game::item::Item;
use game::world::World;
use modloader::content::particles::ParticleKind;
use std::rc::Rc;

const ICON_CRACK: &str = "iconcrack_";
const TINTED_PORTAL: &str = "tintportal_";
const HEX: u32 = 16;
const CHANNEL: f32 = 255.0;

// particles mods name that vanilla RenderGlobal.spawnParticle does not know
pub fn spawn(
    global: &RenderGlobal,
    name: &str,
    world: &Rc<World>,
    pos: [f64; 3],
    motion: [f64; 3],
) -> Option<Rc<Entity>>
{
    let [x, y, z] = pos;
    registered(global, name, world, pos, motion)
        .or_else(|| iconCrack(name, world, x, y, z))
        .or_else(|| tintedPortal(name, world, pos, motion))
}

// a particle a mod registered, a beta particle with its own colour, size, lifetime or gravity
fn registered(
    global: &RenderGlobal,
    name: &str,
    world: &Rc<World>,
    pos: [f64; 3],
    motion: [f64; 3],
) -> Option<Rc<Entity>>
{
    let spec: ParticleKind = modloader::content::particles::kind(name)?;
    let particle: Rc<Entity> = vanillaParticle(global, &spec.m_base, world, pos, motion)?;
    let fields: &FXFields = entity_fx::fx(&particle);

    if let Some(color) = spec.m_color
    {
        let channel = |shift: u32| ((color >> shift) & 0xFF) as f32 / CHANNEL;
        fields.m_particleRed.set(channel(16));
        fields.m_particleGreen.set(channel(8));
        fields.m_particleBlue.set(channel(0));
    }

    if let Some(scale) = spec.m_scale
    {
        fields.m_particleScale.set(fields.m_particleScale.get() * scale);
    }

    if let Some(lifetime) = spec.m_lifetime
    {
        fields.m_particleMaxAge.set(lifetime);
    }

    if let Some(gravity) = spec.m_gravity
    {
        fields.m_particleGravity.set(gravity);
    }

    Some(particle)
}

// tintportal_RRGGBB: a portal particle in one color, EntityPortalFX recolored
fn tintedPortal(name: &str, world: &Rc<World>, pos: [f64; 3], motion: [f64; 3]) -> Option<Rc<Entity>>
{
    let color: u32 = u32::from_str_radix(name.strip_prefix(TINTED_PORTAL)?, HEX).ok()?;
    let [x, y, z] = pos;
    let [velocityX, velocityY, velocityZ] = motion;
    let particle: Rc<Entity> = entity_portal_fx::new(world, x, y, z, velocityX, velocityY, velocityZ);
    let fields: &FXFields = entity_fx::fx(&particle);
    let channel = |shift: u32| ((color >> shift) & 0xFF) as f32 / CHANNEL;
    fields.m_particleRed.set(channel(16));
    fields.m_particleGreen.set(channel(8));
    fields.m_particleBlue.set(channel(0));
    Some(particle)
}

// iconcrack_<item id>_<damage>: bits of an item's icon, as later versions name the item break particle
fn iconCrack(name: &str, world: &Rc<World>, x: f64, y: f64, z: f64) -> Option<Rc<Entity>>
{
    let (id, damage) = name.strip_prefix(ICON_CRACK)?.split_once('_').unwrap_or((name.strip_prefix(ICON_CRACK)?, "0"));
    let item: Rc<Item> = Item::byId(id.parse().ok()?)?;
    let particle: Rc<Entity> = entity_slime_fx::new(world, x, y, z, &item);
    entity_fx::fx(&particle).m_particleTextureIndex.set(item.getIconFromDamage(damage.parse().ok()?));
    Some(particle)
}
