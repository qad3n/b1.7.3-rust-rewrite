use crate::screens::OpenRequest;
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

    // a multiplayer client adds server spawned mod entities under the server's entity id
    fn addClientEntity(&self, _id: i32, _entity: Rc<Entity>) -> Result<(), String>
    {
        Err("only a client adds server entities".to_owned())
    }
}

thread_local! {
    static PLATFORM: RefCell<Option<Rc<dyn Platform>>> = const { RefCell::new(None) };
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
    let screenType = crate::screens::byId(screen).ok_or_else(|| format!("screen {screen} is not registered"))?;
    let player: Rc<Entity> = crate::host::worldPlayer(player)?;
    platform()?.openScreen(&player, OpenRequest { m_screen: screenType, m_pos: pos, m_data: data, m_window: 0 })
}

pub fn teleport(player: i32, dimension: i32, pos: [f64; 3]) -> Result<(), String>
{
    let player: Rc<Entity> = crate::host::worldPlayer(player)?;
    platform()?.teleport(&player, dimension, pos)
}

pub fn screenSize() -> Option<(i32, i32)> { platform().ok()?.screenSize() }

pub fn addClientEntity(id: i32, entity: Rc<Entity>) -> Result<(), String> { platform()?.addClientEntity(id, entity) }
