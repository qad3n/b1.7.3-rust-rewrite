use crate::render::particle::entity_fx::{self, FXFields};
use crate::render::particle::{entity_portal_fx, entity_slime_fx};
use game::entity::Entity;
use game::item::Item;
use game::world::World;
use std::rc::Rc;

const ICON_CRACK: &str = "iconcrack_";
const TINTED_PORTAL: &str = "tintportal_";
const HEX: u32 = 16;
const CHANNEL: f32 = 255.0;

// particles mods name that vanilla RenderGlobal.spawnParticle does not know
pub fn spawn(name: &str, world: &Rc<World>, pos: [f64; 3], motion: [f64; 3]) -> Option<Rc<Entity>>
{
    let [x, y, z] = pos;
    iconCrack(name, world, x, y, z).or_else(|| tintedPortal(name, world, pos, motion))
}

// tintportal_RRGGBB: a portal particle in one color, as Aether's leaves and portal recolor EntityPortalFX
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
