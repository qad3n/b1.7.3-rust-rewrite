use crate::render::particle::entity_fx::{self, FXFields};
use crate::render::particle::{entity_digging_fx, entity_portal_fx, entity_slime_fx};
use crate::render::{RenderGlobal, vanillaParticle};
use game::block::Block;
use game::entity::Entity;
use game::item::Item;
use game::world::World;
use modloader::content::particles::ParticleKind;
use std::rc::Rc;

const ICON_CRACK: &str = "iconcrack_";
const BLOCK_CRACK: &str = "blockcrack_";
const TINTED_PORTAL: &str = "tintportal_";
const HEX: u32 = 16;
// the effect renderer layer that draws from particles.png
const PARTICLE_SHEET: i32 = 0;

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
        .or_else(|| blockCrack(name, world, pos, motion))
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
        let [red, green, blue]: [f32; 3] = game::math::math_helper::rgb(color);
        fields.m_particleRed.set(red);
        fields.m_particleGreen.set(green);
        fields.m_particleBlue.set(blue);
    }

    // a mod sprite on the particle sheet, only for bases that draw from that sheet
    let sprite: Option<i32> = spec
        .m_texture
        .as_ref()
        .and_then(|texture| modloader::atlas::slotOf(modloader::atlas::Atlas::Particles, texture));
    if let Some(sprite) = sprite.filter(|_| particle.getFXLayer() == PARTICLE_SHEET)
    {
        fields.m_particleTextureIndex.set(sprite);
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
    let [red, green, blue]: [f32; 3] = game::math::math_helper::rgb(color);
    fields.m_particleRed.set(red);
    fields.m_particleGreen.set(green);
    fields.m_particleBlue.set(blue);
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

// blockcrack_<block id>_<meta>: bits of a block flying off with the given motion, as later versions name EntityDiggingFX
fn blockCrack(name: &str, world: &Rc<World>, pos: [f64; 3], motion: [f64; 3]) -> Option<Rc<Entity>>
{
    let rest: &str = name.strip_prefix(BLOCK_CRACK)?;
    let (id, meta) = rest.split_once('_').unwrap_or((rest, "0"));
    let block: Rc<Block> = Block::byId(id.parse().ok()?)?;
    let [x, y, z] = pos;
    let [velocityX, velocityY, velocityZ] = motion;
    Some(entity_digging_fx::new(world, x, y, z, velocityX, velocityY, velocityZ, &block, 0, meta.parse().ok()?))
}
