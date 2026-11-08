use crate::player::entity_player_mp;
use crate::server::PropertyManager;
use game::entity::{Entity, entity_player};
use game::inventory::IInventory;
use modloader::network::loader::{self, OpenScreen};
use modloader::screens::OpenRequest;
use std::path::Path;
use std::rc::Rc;

// what only the dedicated server can do for mods: open containers over the network and move players
struct ServerPlatform;

impl modloader::netplay::Platform for ServerPlatform
{
    fn openScreen(&self, player: &Rc<Entity>, request: OpenRequest) -> Result<(), String>
    {
        entity_player_mp::getNextWidowId(player);
        let window: i32 = entity_player_mp::currentWindowId(player);
        if !request.m_screen.m_spec.slots.is_empty()
        {
            let world = player.m_worldObj.get().ok_or("the player is in no world")?;
            let inventory: Rc<dyn IInventory> = match request.m_pos
            {
                Some(pos) => modloader::containers::tileInventory(&world, pos)
                    .ok_or("no mod tile entity inventory at that position")?,
                None => modloader::containers::scratchInventory(&request.m_screen),
            };
            let playerInventory: Rc<dyn IInventory> = entity_player::inventory(player);
            let scratch: bool = request.m_pos.is_none();
            let container = modloader::containers::create(&request.m_screen, playerInventory, inventory, scratch);
            entity_player_mp::openContainer(player, container);
        }
        let screen: i32 = modloader::screens::numeric(&request.m_screen.m_id).ok_or("unknown screen")?;
        let message: OpenScreen = OpenScreen { screen, window, pos: request.m_pos, data: request.m_data };
        loader::send(loader::OPEN_SCREEN, player, &message);
        Ok(())
    }

    fn teleport(&self, player: &Rc<Entity>, dimension: i32, pos: [f64; 3]) -> Result<(), String>
    {
        crate::modded::teleport(player, dimension, pos)
    }

    fn syncCursor(&self, player: &Rc<Entity>) { entity_player_mp::updateHeldItem(player); }

    fn isOp(&self, player: &Rc<Entity>) -> bool
    {
        entity_player_mp::mcServer(player).configManager().isOp(entity_player::fields(player).m_username.get().as_ref())
    }
}

mod properties
{
    pub const MODS_LOADED: &str = "modsloaded";
    pub const STRICT_MODS: &str = "strictmods";
}

// modsloaded=false keeps the server vanilla, strictmods=true makes clients match the server mod list exactly
pub fn start(propertyManager: &PropertyManager)
{
    if !propertyManager.getBooleanProperty(properties::MODS_LOADED, true)
    {
        modloader::log::info(modloader::log::LOADER, "mods are off, modsloaded=false in server.properties");
        return;
    }
    modloader::netplay::install(Rc::new(ServerPlatform));
    modloader::bootstrap(Path::new("."));
    modloader::netsync::setStrictMods(propertyManager.getBooleanProperty(properties::STRICT_MODS, false));
    modloader::netsync::logServerMods();
    modloader::play::lifecycle(modloader::EventKind::ServerStarting);
}

pub fn teleport(player: &Rc<Entity>, dimension: i32, pos: [f64; 3]) -> Result<(), String>
{
    let current: i32 = entity_player::fields(player).m_dimension.get();
    let handler = entity_player_mp::netServerHandler(player);
    if current != dimension
    {
        travel(player, current, dimension, pos)?;
    }
    player.setLocationAndAngles(pos[0], pos[1], pos[2], player.m_rotationYaw.get(), player.m_rotationPitch.get());
    handler.teleportTo(pos[0], pos[1], pos[2], player.m_rotationYaw.get(), player.m_rotationPitch.get());
    Ok(())
}

// sendPlayerToOtherDimension for any pair of dimensions and an exact target point
fn travel(player: &Rc<Entity>, from: i32, to: i32, pos: [f64; 3]) -> Result<(), String>
{
    let server = entity_player_mp::mcServer(player);
    if !modloader::dimensions::serverDimensions().contains(&to)
    {
        return Err(format!("dimension {to} does not exist on this server"));
    }
    let source = server.getWorldManager(from);
    let target = server.getWorldManager(to);
    entity_player::fields(player).m_dimension.set(to);
    entity_player_mp::netServerHandler(player)
        .sendPacket(Box::new(game::network::packets::Packet9Respawn::new(to as i8)));
    source.removePlayer(player);
    player.m_isDead.set(false);
    player.setLocationAndAngles(pos[0], pos[1], pos[2], player.m_rotationYaw.get(), player.m_rotationPitch.get());
    target.entityJoinedWorld(player);
    target.updateEntityWithOptionalForce(player, false);
    server.configManager().onChangedDimension(player);
    player.setWorld(Some(Rc::clone(&target)));
    modloader::play::dimensionChanged(player, from, to);
    Ok(())
}
