use crate::host::context::{self, dimension, withWorld};
use crate::{events, state};
use game::entity::{Entity, entity_player};
use game::world::World;
use mod_protocol::abi::{BlockPos, EventData, EventKind, Sender};
use std::rc::Rc;

// the client and server binaries call these at their own lifecycle points, all are no-ops without mods
pub fn active() -> bool { state::hasMods() || state::isKeeper() }

pub fn worldChanged(previous: Option<&Rc<World>>, world: Option<&Rc<World>>)
{
    if !active()
    {
        return;
    }
    if let Some(previous) = previous
    {
        withWorld(previous, || events::post(EventKind::WorldUnload, EventData::World { world: dimension(previous) }));
        context::forgetWorld(previous);
    }
    let Some(world) = world
    else
    {
        crate::network::setLocalPlayer(None);
        return;
    };
    worldLoaded(world);
}

// an authoritative world applies its saved id map before its chunks are read
pub fn worldLoaded(world: &Rc<World>)
{
    if !active()
    {
        return;
    }
    context::rememberWorld(world);
    if crate::side::isAuthority(world)
    {
        crate::remap::onWorldLoad(world);
        crate::persist::loadWorld(world);
    }
    withWorld(world, || events::post(EventKind::WorldLoad, EventData::World { world: dimension(world) }));
}

pub fn playerChanged(player: Option<Rc<Entity>>)
{
    if active()
    {
        crate::network::setLocalPlayer(player);
    }
}

// true when the watchdog asks the game to stop
pub fn tick(pre: bool) -> bool
{
    if !active()
    {
        return false;
    }
    if !pre
    {
        crate::netplay::flushTeleports();
    }
    events::post(
        if pre
        {
            EventKind::TickPre
        }
        else
        {
            EventKind::TickPost
        },
        EventData::Empty,
    );
    !pre && crate::profiler::tick()
}

pub fn clientTick()
{
    if active()
    {
        crate::network::loader::flushPendingSpawns();
        events::post(EventKind::ClientTick, EventData::Empty);
    }
}

pub fn key(key: i32, down: bool)
{
    if !active()
    {
        return;
    }
    crate::keybinds::pressed(key, down);
    if events::hasListeners(EventKind::KeyInput)
    {
        events::post(EventKind::KeyInput, EventData::Key { key, pressed: down });
    }
}

fn sender(player: Option<&Rc<Entity>>, op: bool) -> Sender
{
    match player
    {
        Some(player) => Sender {
            player: Some(player.m_entityId.get()),
            name: entity_player::fields(player).m_username.get().map(|name| name.to_string()).unwrap_or_default(),
            world: player.m_worldObj.get().map_or(0, |world| dimension(&world)),
            op,
        },
        None => Sender { player: None, name: "CONSOLE".to_owned(), world: 0, op: true },
    }
}

// a slash line without its slash, Some(reply) when a mod command took it
pub fn command(line: &str, player: Option<&Rc<Entity>>, op: bool) -> Option<String>
{
    if !active()
    {
        return None;
    }

    let source: String = player.map_or_else(|| String::from("console"), crate::gamelog::name);
    crate::log::debug(crate::log::GAME, &format!("{source} ran command {line}"));
    let world: Option<Rc<World>> = player.and_then(|player| player.m_worldObj.get());
    let run = || crate::commands::execute(line, sender(player, op));
    match world
    {
        Some(world) => withWorld(&world, run),
        None => run(),
    }
}

// chat listeners may cancel a line or rewrite it, None means the line is dropped
pub fn chat(player: &Rc<Entity>, message: &str) -> Option<String>
{
    if !active() || !events::hasListeners(EventKind::PlayerChat)
    {
        return Some(message.to_owned());
    }
    let envelope = events::post(
        EventKind::PlayerChat,
        EventData::Chat { player: player.m_entityId.get(), message: message.to_owned() },
    );
    if envelope.canceled
    {
        return None;
    }
    match envelope.data
    {
        EventData::Chat { message, .. } => Some(message),
        _ => Some(message.to_owned()),
    }
}

pub fn lifecycle(kind: EventKind)
{
    if active()
    {
        events::post(kind, EventData::Empty);
    }
}

pub fn player(kind: EventKind, player: &Rc<Entity>)
{
    if !active()
    {
        return;
    }
    if kind == EventKind::PlayerLogin
    {
        crate::attributes::syncOwn(player);
        crate::content::effects::syncOwn(player);
        crate::content::progress::syncOwn(player);
    }

    let action: &str = match kind
    {
        EventKind::PlayerLogin => "logged in",
        EventKind::PlayerLogout => "logged out",
        EventKind::PlayerRespawn => "respawned",
        _ => "changed",
    };
    crate::log::debug(crate::log::GAME, &format!("{} {action}", crate::gamelog::name(player)));
    if !events::hasListeners(kind)
    {
        return;
    }
    let world: i32 = player.m_worldObj.get().map_or(0, |world| dimension(&world));
    let name: String = entity_player::fields(player).m_username.get().map(|name| name.to_string()).unwrap_or_default();
    events::post(kind, EventData::Player { player: player.m_entityId.get(), world, name });
}

pub fn dimensionChanged(player: &Rc<Entity>, from: i32, to: i32)
{
    if active()
    {
        let name: String = crate::gamelog::name(player);
        crate::log::debug(crate::log::GAME, &format!("{name} moved from dimension {from} to {to}"));
        events::post(
            EventKind::PlayerChangeDimension,
            EventData::Dimension { player: player.m_entityId.get(), from, to },
        );
    }
}

// true when a listener canceled the break or the activation
pub fn blockBreak(world: &Rc<World>, x: i32, y: i32, z: i32, player: &Rc<Entity>) -> bool
{
    if !active()
    {
        return false;
    }
    game::hooks::fire(game::hooks::GameEvent::BlockBreak {
        world,
        x,
        y,
        z,
        block: world.getBlockId(x, y, z),
        meta: world.getBlockMetadata(x, y, z),
        player: Some(player),
    })
}

pub fn blockActivate(world: &Rc<World>, x: i32, y: i32, z: i32, player: &Rc<Entity>) -> bool
{
    if !active()
    {
        return false;
    }
    game::hooks::fire(game::hooks::GameEvent::BlockActivate {
        world,
        x,
        y,
        z,
        block: world.getBlockId(x, y, z),
        player,
    })
}

pub fn interact(player: &Rc<Entity>, pos: BlockPos, face: i32) -> bool
{
    if !active() || !events::hasListeners(EventKind::PlayerInteract)
    {
        return false;
    }
    let world: i32 = player.m_worldObj.get().map_or(0, |world| dimension(&world));
    let data: EventData =
        EventData::ItemUse { world, player: player.m_entityId.get(), stack: None, pos: Some(pos), face };
    events::post(EventKind::PlayerInteract, data).canceled
}

// /mods/<namespace>/<path> textures come from the mod that owns the namespace
pub fn modAsset(path: &str) -> Option<Vec<u8>>
{
    let rest: &str = path.strip_prefix("/mods/")?;
    let (namespace, file) = rest.split_once('/')?;
    let runtime = state::byId(namespace)?;
    let bytes: Option<Vec<u8>> = runtime.m_source.borrow_mut().read(&format!("assets/{namespace}/{file}"));
    bytes
}

pub fn shutdown() { crate::lifecycle::shutdown(); }

pub fn hud(partial: f32, width: i32, height: i32) -> Vec<mod_protocol::abi::DrawList>
{
    if events::hasListeners(EventKind::RenderHud)
    {
        events::post(EventKind::RenderHud, EventData::Render { partial, width, height });
    }
    crate::screens::drawHuds(partial, width, height)
}

// true cancels opening the screen
pub fn screenOpen(name: &str) -> bool
{
    active()
        && events::hasListeners(EventKind::ScreenOpen)
        && events::post(EventKind::ScreenOpen, EventData::Screen { name: name.to_owned() }).canceled
}

pub fn renderWorldLast(partial: f32)
{
    if !active()
    {
        return;
    }
    if events::hasListeners(EventKind::RenderWorldLast)
    {
        events::post(EventKind::RenderWorldLast, EventData::Render { partial, width: 0, height: 0 });
    }
    crate::screens::drawWorld(partial);
}

pub fn blockHighlight(world: &Rc<World>, x: i32, y: i32, z: i32) -> bool
{
    if !active() || !events::hasListeners(EventKind::RenderBlockHighlight)
    {
        return false;
    }
    let data: EventData = EventData::Block {
        world: dimension(world),
        pos: BlockPos::new(x, y, z),
        block: world.getBlockId(x, y, z),
        meta: 0,
        player: None,
    };
    events::post(EventKind::RenderBlockHighlight, data).canceled
}

pub fn itemUseFirst(
    world: &Rc<World>,
    player: &Rc<Entity>,
    stack: Option<&Rc<game::item::ItemStack>>,
    pos: BlockPos,
    face: i32,
) -> bool
{
    if !active() || !events::hasListeners(EventKind::ItemUseFirst)
    {
        return false;
    }
    let data: EventData = EventData::ItemUse {
        world: dimension(world),
        player: player.m_entityId.get(),
        stack: stack.map(|stack| crate::host::context::stackData(stack)),
        pos: Some(pos),
        face,
    };
    withWorld(world, || events::post(EventKind::ItemUseFirst, data)).canceled
}

pub fn present() -> bool { active() || !state::problems().is_empty() || !state::skipped().is_empty() }
