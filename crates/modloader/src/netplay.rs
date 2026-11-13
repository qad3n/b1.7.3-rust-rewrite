use crate::screens::{OpenRequest, ScreenType};
use game::entity::Entity;
use mod_protocol::Identifier;
use mod_protocol::abi::BlockPos;
use std::cell::RefCell;
use std::rc::Rc;

// what only the client or the server binary can do, installed by that binary at startup
pub trait Platform
{
    fn openScreen(&self, player: &Rc<Entity>, request: OpenRequest) -> Result<(), String>;

    fn teleport(&self, player: &Rc<Entity>, dimension: i32, pos: [f64; 3]) -> Result<(), String>;

    fn screenSize(&self) -> Option<(i32, i32)> { None }

    fn clipboard(&self) -> Option<String> { None }

    // tells the player's client what it holds on the cursor after a mod changed it
    fn syncCursor(&self, _player: &Rc<Entity>) {}

    // FontRenderer.getStringWidth on a client, nothing to measure on a server
    fn textWidth(&self, _text: &str) -> i32 { 0 }

    // a first person client hides mod particles on its own player's body since they would cover the view
    fn hidesParticle(&self, _pos: [f64; 3]) -> bool { false }

    // ServerConfigurationManager.isOp on a server, a single player world has no operators to check
    fn isOp(&self, _player: &Rc<Entity>) -> bool { true }

    // a multiplayer client adds server spawned mod entities under the server's entity id
    fn addClientEntity(&self, _id: i32, _entity: Rc<Entity>) -> Result<(), String>
    {
        Err("only a client adds server entities".to_owned())
    }
}

thread_local!
{
    static PLATFORM: RefCell<Option<Rc<dyn Platform>>> = const { RefCell::new(None) };
    static TELEPORTS: RefCell<Vec<(i32, i32, [f64; 3])>> = const { RefCell::new(Vec::new()) };
}

pub fn install(platform: Rc<dyn Platform>) { PLATFORM.with(|slot| *slot.borrow_mut() = Some(platform)); }

fn platform() -> Result<Rc<dyn Platform>, String>
{
    PLATFORM.with(|slot| slot.borrow().clone()).ok_or_else(|| "the game binary installed no loader platform".to_owned())
}

pub fn canSend(player: i32, channel: &Identifier) -> bool
{
    crate::network::channelInfo(channel).is_some() && crate::netsync::peerHasChannel(player, channel)
}

pub fn openScreen(player: i32, screen: &Identifier, pos: Option<BlockPos>, data: Vec<u8>) -> Result<(), String>
{
    let screenType: Rc<ScreenType> =
        crate::screens::byId(screen).ok_or_else(|| format!("screen {screen} is not registered"))?;
    let player: Rc<Entity> = crate::host::worldPlayer(player)?;
    platform()?.openScreen(&player, OpenRequest { m_screen: screenType, m_pos: pos, m_data: data, m_window: 0 })
}

// a guest asks from inside a world tick or a block callback, so the move waits for the end of the tick
pub fn teleport(player: i32, dimension: i32, pos: [f64; 3]) -> Result<(), String>
{
    crate::host::worldPlayer(player)?;
    platform()?;
    TELEPORTS.with(|queue| queue.borrow_mut().push((player, dimension, pos)));
    Ok(())
}

pub fn flushTeleports()
{
    let queued: Vec<(i32, i32, [f64; 3])> = TELEPORTS.with(|queue| std::mem::take(&mut *queue.borrow_mut()));
    for (id, dimension, pos) in queued
    {
        let moved: Result<(), String> = crate::host::worldPlayer(id)
            .and_then(|player| platform().and_then(|platform| platform.teleport(&player, dimension, pos)));

        if let Err(reason) = moved
        {
            crate::log::warn(crate::log::LOADER, &format!("teleport of player {id} failed: {reason}"));
        }
    }
}

pub fn screenSize() -> Option<(i32, i32)> { platform().ok()?.screenSize() }

pub fn clipboard() -> Option<String> { platform().ok()?.clipboard() }

pub fn hidesParticle(pos: [f64; 3]) -> bool { platform().is_ok_and(|platform| platform.hidesParticle(pos)) }

pub fn isOp(player: &Rc<Entity>) -> bool { platform().is_ok_and(|platform| platform.isOp(player)) }

pub fn syncCursor(player: &Rc<Entity>)
{
    if let Ok(platform) = platform()
    {
        platform.syncCursor(player);
    }
}

pub fn textWidth(text: &str) -> i32 { platform().map_or(0, |platform| platform.textWidth(text)) }

pub fn addClientEntity(id: i32, entity: Rc<Entity>) -> Result<(), String> { platform()?.addClientEntity(id, entity) }
