use crate::network::{NetLoginHandler, NetServerHandler};
use crate::player::{ItemInWorldManager, PlayerManager, entity_player_mp};
use crate::server::MinecraftServer;
use crate::world::world_server;
use game::entity::{Entity, entity_player};
use game::network::packet::Packet;
use game::network::packets::{Packet3Chat, Packet4UpdateTime, Packet9Respawn, Packet70Bed};
use game::storage::IPlayerFileData;
use game::tileentity::TileEntity;
use game::world::{ChunkCoordinates, Teleporter, World};
use jvm::array_list::JArrayList;
use jvm::field::Field;
use jvm::hash_map::JHashSet;
use jvm::io::{FileInputStream, FileOutputStream, InputStream, IoResult, OutputStream};
use jvm::logging::Logger;
use jvm::string::JString;
use jvm::throwable::OrNpe;
use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

fn logger() -> &'static Logger { Logger::getLogger("Minecraft") }

fn readAllLines(file: &Path) -> IoResult<Vec<JString>>
{
    let mut stream: FileInputStream = FileInputStream::open(file)?;
    let mut bytes: Vec<u8> = Vec::new();
    let mut buffer: [u8; 8192] = [0; 8192];
    loop
    {
        let count: i32 = stream.readInto(&mut buffer, 0, 8192)?;
        if count < 0
        {
            break;
        }
        bytes.extend_from_slice(&buffer[..count as usize]);
    }
    stream.close()?;
    let text: String = game::port::text::decodePlatform(&bytes);
    Ok(game::port::text::readLines(&text).into_iter().map(JString::from).collect())
}

fn writeAllLines(file: &Path, lines: &[JString]) -> IoResult<()>
{
    let mut printwriter: FileOutputStream = FileOutputStream::create(file, false)?;
    let separator: &str = if cfg!(windows) { "\r\n" } else { "\n" };
    let mut bytes: Vec<u8> = Vec::new();
    for s in lines
    {
        bytes.extend(JString::from(format!("{s}{separator}").as_str()).getBytes());
    }
    // printwriter keeps write errors in its error flag which nothing checks
    printwriter.writeFrom(&bytes, 0, bytes.len() as i32).ok();
    printwriter.close().ok();
    Ok(())
}

pub struct ServerConfigurationManager
{
    pub m_playerEntities: JArrayList<Rc<Entity>>,
    m_mcServer: Rc<MinecraftServer>,
    m_playerManagerObj: [Rc<PlayerManager>; 2],
    m_maxPlayers: i32,
    m_bannedPlayers: Field<JHashSet<JString>>,
    m_bannedIPs: Field<JHashSet<JString>>,
    m_ops: Field<JHashSet<JString>>,
    m_whiteListedIPs: Field<JHashSet<JString>>,
    m_bannedPlayersFile: PathBuf,
    m_ipBanFile: PathBuf,
    m_opFile: PathBuf,
    m_whitelistPlayersFile: PathBuf,
    m_playerNBTManagerObj: Field<Option<Rc<dyn IPlayerFileData>>>,
    m_whiteListEnforced: Cell<bool>,
}

impl ServerConfigurationManager
{
    pub fn new(server: &Rc<MinecraftServer>) -> Rc<ServerConfigurationManager>
    {
        let playerEntities: JArrayList<Rc<Entity>> = JArrayList::new();
        let bannedPlayers: Field<JHashSet<JString>> = Field::new(JHashSet::new());
        let bannedIPs: Field<JHashSet<JString>> = Field::new(JHashSet::new());
        let ops: Field<JHashSet<JString>> = Field::new(JHashSet::new());
        let whiteListedIPs: Field<JHashSet<JString>> = Field::new(JHashSet::new());
        let bannedPlayersFile: PathBuf = server.getFile("banned-players.txt");
        let ipBanFile: PathBuf = server.getFile("banned-ips.txt");
        let opFile: PathBuf = server.getFile("ops.txt");
        let whitelistPlayersFile: PathBuf = server.getFile("white-list.txt");
        let i: i32 = server.propertyManager().getIntProperty("view-distance", 10);
        let manager0: Rc<PlayerManager> = PlayerManager::new(server, 0, i);
        let manager1: Rc<PlayerManager> = PlayerManager::new(server, -1, i);
        let maxPlayers: i32 = server.propertyManager().getIntProperty("max-players", 20);
        let whiteListEnforced: bool = server.propertyManager().getBooleanProperty("white-list", false);
        let manager: Rc<ServerConfigurationManager> = Rc::new(ServerConfigurationManager {
            m_playerEntities: playerEntities,
            m_mcServer: Rc::clone(server),
            m_playerManagerObj: [manager0, manager1],
            m_maxPlayers: maxPlayers,
            m_bannedPlayers: bannedPlayers,
            m_bannedIPs: bannedIPs,
            m_ops: ops,
            m_whiteListedIPs: whiteListedIPs,
            m_bannedPlayersFile: bannedPlayersFile,
            m_ipBanFile: ipBanFile,
            m_opFile: opFile,
            m_whitelistPlayersFile: whitelistPlayersFile,
            m_playerNBTManagerObj: Field::new(None),
            m_whiteListEnforced: Cell::new(whiteListEnforced),
        });
        manager.readBannedPlayers();
        manager.loadBannedList();
        manager.loadOps();
        manager.loadWhiteList();
        manager.writeBannedPlayers();
        manager.saveBannedList();
        manager.saveOps();
        manager.saveWhiteList();
        manager
    }

    fn playerNBTManagerObj(&self) -> Rc<dyn IPlayerFileData> { self.m_playerNBTManagerObj.get().orNpe() }

    pub fn setPlayerManager(&self, worlds: &[Option<Rc<World>>])
    {
        let world: &Rc<World> = worlds[0].as_ref().orNpe();
        self.m_playerNBTManagerObj.set(Some(world.getWorldFile().orNpe().getPlayerDataStorage()));
    }

    pub fn onChangedDimension(&self, player: &Rc<Entity>)
    {
        self.m_playerManagerObj[0].removePlayer(player);
        self.m_playerManagerObj[1].removePlayer(player);
        let dimension: i32 = entity_player::fields(player).m_dimension.get();
        self.getPlayerManager(dimension).addPlayer(player);
        let worldserver: Rc<World> = self.m_mcServer.getWorldManager(dimension);
        world_server::chunkProviderServer(&worldserver)
            .loadChunk(player.m_posX.get() as i32 >> 4, player.m_posZ.get() as i32 >> 4);
    }

    pub fn getMaxTrackingDistance(&self) -> i32 { self.m_playerManagerObj[0].getMaxTrackingDistance() }

    fn getPlayerManager(&self, dimension: i32) -> Rc<PlayerManager>
    {
        if dimension == -1
        {
            Rc::clone(&self.m_playerManagerObj[1])
        }
        else
        {
            Rc::clone(&self.m_playerManagerObj[0])
        }
    }

    pub fn readPlayerDataFromFile(&self, player: &Rc<Entity>) { self.playerNBTManagerObj().readPlayerData(player); }

    pub fn playerLoggedIn(&self, player: &Rc<Entity>)
    {
        self.m_playerEntities.add(Rc::clone(player));
        let dimension: i32 = entity_player::fields(player).m_dimension.get();
        let worldserver: Rc<World> = self.m_mcServer.getWorldManager(dimension);
        world_server::chunkProviderServer(&worldserver)
            .loadChunk(player.m_posX.get() as i32 >> 4, player.m_posZ.get() as i32 >> 4);
        while worldserver.getCollidingBoundingBoxes(player, &player.m_boundingBox).size() != 0
        {
            player.setPosition(player.m_posX.get(), player.m_posY.get() + 1.0, player.m_posZ.get());
        }
        worldserver.entityJoinedWorld(player);
        self.getPlayerManager(dimension).addPlayer(player);
    }

    pub fn r#move(&self, player: &Rc<Entity>)
    {
        self.getPlayerManager(entity_player::fields(player).m_dimension.get()).movePlayer(player);
    }

    pub fn playerLoggedOut(&self, player: &Rc<Entity>)
    {
        self.playerNBTManagerObj().writePlayerData(player);
        let dimension: i32 = entity_player::fields(player).m_dimension.get();
        self.m_mcServer.getWorldManager(dimension).setEntityDead(player);
        self.m_playerEntities.remove(player);
        self.getPlayerManager(dimension).removePlayer(player);
    }

    pub fn login(&self, networkHandler: &NetLoginHandler, name: Option<&JString>) -> Option<Rc<Entity>>
    {
        let name: &JString = name.orNpe();
        if self.m_bannedPlayers.with(|set| set.contains(&name.trim().toLowerCase()))
        {
            networkHandler.kickUser("You are banned from this server!");
            return None;
        }
        if !self.isAllowedToLogin(name)
        {
            networkHandler.kickUser("You are not white-listed on this server!");
            return None;
        }
        let mut s: JString = JString::from(networkHandler.netManager().getRemoteAddress().as_str());
        s = s.substringFrom(s.indexOf(b'/' as u16) + 1);
        s = s.substring(0, s.indexOf(b':' as u16));
        if self.m_bannedIPs.with(|set| set.contains(&s))
        {
            networkHandler.kickUser("Your IP address is banned from this server!");
            return None;
        }
        if self.m_playerEntities.size() >= self.m_maxPlayers
        {
            networkHandler.kickUser("The server is full!");
            return None;
        }
        let mut i: i32 = 0;
        while i < self.m_playerEntities.size()
        {
            let entityplayermp: Rc<Entity> = self.m_playerEntities.get(i);
            if entity_player::fields(&entityplayermp).m_username.get().orNpe().equalsIgnoreCase(Some(name))
            {
                entity_player_mp::netServerHandler(&entityplayermp).kickPlayer("You logged in from another location");
            }
            i += 1;
        }
        Some(entity_player_mp::new(
            &self.m_mcServer,
            &self.m_mcServer.getWorldManager(0),
            Some(name.clone()),
            &ItemInWorldManager::new(&self.m_mcServer.getWorldManager(0)),
        ))
    }

    pub fn recreatePlayerEntity(&self, player: &Rc<Entity>, dimension: i32) -> Rc<Entity>
    {
        let playerFields: &entity_player::PlayerFields = entity_player::fields(player);
        self.m_mcServer.getEntityTracker(playerFields.m_dimension.get()).removeTrackedPlayerSymmetric(player);
        self.m_mcServer.getEntityTracker(playerFields.m_dimension.get()).untrackEntity(player);
        self.getPlayerManager(playerFields.m_dimension.get()).removePlayer(player);
        self.m_playerEntities.remove(player);
        self.m_mcServer.getWorldManager(playerFields.m_dimension.get()).removePlayer(player);
        let chunkcoordinates: Option<ChunkCoordinates> = entity_player::getPlayerSpawnCoordinate(player);
        playerFields.m_dimension.set(dimension);
        let entityplayermp: Rc<Entity> = entity_player_mp::new(
            &self.m_mcServer,
            &self.m_mcServer.getWorldManager(playerFields.m_dimension.get()),
            playerFields.m_username.get(),
            &ItemInWorldManager::new(&self.m_mcServer.getWorldManager(playerFields.m_dimension.get())),
        );
        entityplayermp.m_entityId.set(player.m_entityId.get());
        entity_player_mp::fields(&entityplayermp)
            .m_playerNetServerHandler
            .set(entity_player_mp::fields(player).m_playerNetServerHandler.get());
        let worldserver: Rc<World> = self.m_mcServer.getWorldManager(playerFields.m_dimension.get());
        if let Some(chunkcoordinates) = chunkcoordinates
        {
            let chunkcoordinates1: Option<ChunkCoordinates> = entity_player::loadSpawnPoint(
                &self.m_mcServer.getWorldManager(playerFields.m_dimension.get()),
                &chunkcoordinates,
            );
            match chunkcoordinates1
            {
                Some(chunkcoordinates1) =>
                {
                    entityplayermp.setLocationAndAngles(
                        (chunkcoordinates1.m_x as f32 + 0.5) as f64,
                        (chunkcoordinates1.m_y as f32 + 0.1) as f64,
                        (chunkcoordinates1.m_z as f32 + 0.5) as f64,
                        0.0,
                        0.0,
                    );
                    entity_player::setPlayerSpawnCoordinate(&entityplayermp, Some(chunkcoordinates));
                }
                None => entity_player_mp::netServerHandler(&entityplayermp).sendPacket(Box::new(Packet70Bed::new(0))),
            }
        }
        world_server::chunkProviderServer(&worldserver)
            .loadChunk(entityplayermp.m_posX.get() as i32 >> 4, entityplayermp.m_posZ.get() as i32 >> 4);
        while worldserver.getCollidingBoundingBoxes(&entityplayermp, &entityplayermp.m_boundingBox).size() != 0
        {
            entityplayermp.setPosition(
                entityplayermp.m_posX.get(),
                entityplayermp.m_posY.get() + 1.0,
                entityplayermp.m_posZ.get(),
            );
        }
        let newDimension: i32 = entity_player::fields(&entityplayermp).m_dimension.get();
        let handler: Rc<NetServerHandler> = entity_player_mp::netServerHandler(&entityplayermp);
        handler.sendPacket(Box::new(Packet9Respawn::new(newDimension as i8)));
        handler.teleportTo(
            entityplayermp.m_posX.get(),
            entityplayermp.m_posY.get(),
            entityplayermp.m_posZ.get(),
            entityplayermp.m_rotationYaw.get(),
            entityplayermp.m_rotationPitch.get(),
        );
        self.sendWorldInfo(&entityplayermp, &worldserver);
        self.getPlayerManager(newDimension).addPlayer(&entityplayermp);
        worldserver.entityJoinedWorld(&entityplayermp);
        self.m_playerEntities.add(Rc::clone(&entityplayermp));
        entity_player_mp::initMenu(&entityplayermp);
        entity_player_mp::afterRespawn(&entityplayermp);
        entityplayermp
    }

    pub fn sendPlayerToOtherDimension(&self, player: &Rc<Entity>)
    {
        let playerFields: &entity_player::PlayerFields = entity_player::fields(player);
        let worldserver: Rc<World> = self.m_mcServer.getWorldManager(playerFields.m_dimension.get());
        let b0: i8 = if playerFields.m_dimension.get() == -1
        {
            0
        }
        else
        {
            -1
        };
        playerFields.m_dimension.set(b0 as i32);
        let worldserver1: Rc<World> = self.m_mcServer.getWorldManager(playerFields.m_dimension.get());
        let handler: Rc<NetServerHandler> = entity_player_mp::netServerHandler(player);
        handler.sendPacket(Box::new(Packet9Respawn::new(playerFields.m_dimension.get() as i8)));
        worldserver.removePlayer(player);
        player.m_isDead.set(false);
        let mut d0: f64 = player.m_posX.get();
        let mut d1: f64 = player.m_posZ.get();
        let d2: f64 = 8.0;
        if playerFields.m_dimension.get() == -1
        {
            d0 /= d2;
            d1 /= d2;
            player.setLocationAndAngles(
                d0,
                player.m_posY.get(),
                d1,
                player.m_rotationYaw.get(),
                player.m_rotationPitch.get(),
            );
            if player.isEntityAlive()
            {
                worldserver.updateEntityWithOptionalForce(player, false);
            }
        }
        else
        {
            d0 *= d2;
            d1 *= d2;
            player.setLocationAndAngles(
                d0,
                player.m_posY.get(),
                d1,
                player.m_rotationYaw.get(),
                player.m_rotationPitch.get(),
            );
            if player.isEntityAlive()
            {
                worldserver.updateEntityWithOptionalForce(player, false);
            }
        }
        if player.isEntityAlive()
        {
            worldserver1.entityJoinedWorld(player);
            player.setLocationAndAngles(
                d0,
                player.m_posY.get(),
                d1,
                player.m_rotationYaw.get(),
                player.m_rotationPitch.get(),
            );
            worldserver1.updateEntityWithOptionalForce(player, false);
            world_server::chunkProviderServer(&worldserver1).m_chunkLoadOverride.set(true);
            Teleporter::new().onDimensionChanged(&worldserver1, player);
            world_server::chunkProviderServer(&worldserver1).m_chunkLoadOverride.set(false);
        }
        self.onChangedDimension(player);
        handler.teleportTo(
            player.m_posX.get(),
            player.m_posY.get(),
            player.m_posZ.get(),
            player.m_rotationYaw.get(),
            player.m_rotationPitch.get(),
        );
        player.setWorld(Some(Rc::clone(&worldserver1)));
        self.sendWorldInfo(player, &worldserver1);
        self.sendPlayerInfo(player);
    }

    pub fn onTick(&self)
    {
        for manager in &self.m_playerManagerObj
        {
            manager.updatePlayerInstances();
        }
    }

    pub fn markBlockNeedsUpdate(&self, x: i32, y: i32, z: i32, dimension: i32)
    {
        self.getPlayerManager(dimension).markBlockNeedsUpdate(x, y, z);
    }

    pub fn sendPacketToAllPlayers(&self, packet: Box<dyn Packet>)
    {
        let mut i: i32 = 0;
        while i < self.m_playerEntities.size()
        {
            let entityplayermp: Rc<Entity> = self.m_playerEntities.get(i);
            entity_player_mp::netServerHandler(&entityplayermp).sendPacket(packet.clonePacket());
            i += 1;
        }
    }

    pub fn sendPacketToAllPlayersInDimension(&self, packet: Box<dyn Packet>, dimension: i32)
    {
        let mut i: i32 = 0;
        while i < self.m_playerEntities.size()
        {
            let entityplayermp: Rc<Entity> = self.m_playerEntities.get(i);
            if entity_player::fields(&entityplayermp).m_dimension.get() == dimension
            {
                entity_player_mp::netServerHandler(&entityplayermp).sendPacket(packet.clonePacket());
            }
            i += 1;
        }
    }

    pub fn getPlayerList(&self) -> String
    {
        let mut s: String = String::new();
        let mut i: i32 = 0;
        while i < self.m_playerEntities.size()
        {
            if i > 0
            {
                s.push_str(", ");
            }
            s.push_str(&JString::display(
                entity_player::fields(&self.m_playerEntities.get(i)).m_username.get().as_ref(),
            ));
            i += 1;
        }
        s
    }

    pub fn banPlayer(&self, name: &JString)
    {
        self.m_bannedPlayers.update(|set| set.add(name.toLowerCase()));
        self.writeBannedPlayers();
    }

    pub fn pardonPlayer(&self, name: &JString)
    {
        self.m_bannedPlayers.update(|set| set.remove(&name.toLowerCase()));
        self.writeBannedPlayers();
    }

    fn readBannedPlayers(&self)
    {
        self.m_bannedPlayers.update(JHashSet::clear);
        match readAllLines(&self.m_bannedPlayersFile)
        {
            Ok(lines) =>
            {
                for s in lines
                {
                    self.m_bannedPlayers.update(|set| set.add(s.trim().toLowerCase()));
                }
            }
            Err(exception) => logger().warning(&format!("Failed to load ban list: {exception}")),
        }
    }

    fn writeBannedPlayers(&self)
    {
        if let Err(exception) = writeAllLines(&self.m_bannedPlayersFile, &self.m_bannedPlayers.with(JHashSet::items))
        {
            logger().warning(&format!("Failed to save ban list: {exception}"));
        }
    }

    pub fn banIP(&self, ip: &JString)
    {
        self.m_bannedIPs.update(|set| set.add(ip.toLowerCase()));
        self.saveBannedList();
    }

    pub fn pardonIP(&self, ip: &JString)
    {
        self.m_bannedIPs.update(|set| set.remove(&ip.toLowerCase()));
        self.saveBannedList();
    }

    fn loadBannedList(&self)
    {
        self.m_bannedIPs.update(JHashSet::clear);
        match readAllLines(&self.m_ipBanFile)
        {
            Ok(lines) =>
            {
                for s in lines
                {
                    self.m_bannedIPs.update(|set| set.add(s.trim().toLowerCase()));
                }
            }
            Err(exception) => logger().warning(&format!("Failed to load ip ban list: {exception}")),
        }
    }

    fn saveBannedList(&self)
    {
        if let Err(exception) = writeAllLines(&self.m_ipBanFile, &self.m_bannedIPs.with(JHashSet::items))
        {
            logger().warning(&format!("Failed to save ip ban list: {exception}"));
        }
    }

    pub fn opPlayer(&self, name: &JString)
    {
        self.m_ops.update(|set| set.add(name.toLowerCase()));
        self.saveOps();
    }

    pub fn deopPlayer(&self, name: &JString)
    {
        self.m_ops.update(|set| set.remove(&name.toLowerCase()));
        self.saveOps();
    }

    fn loadOps(&self)
    {
        self.m_ops.update(JHashSet::clear);
        match readAllLines(&self.m_opFile)
        {
            Ok(lines) =>
            {
                for s in lines
                {
                    self.m_ops.update(|set| set.add(s.trim().toLowerCase()));
                }
            }
            Err(exception) => logger().warning(&format!("Failed to load ip ban list: {exception}")),
        }
    }

    fn saveOps(&self)
    {
        if let Err(exception) = writeAllLines(&self.m_opFile, &self.m_ops.with(JHashSet::items))
        {
            logger().warning(&format!("Failed to save ip ban list: {exception}"));
        }
    }

    fn loadWhiteList(&self)
    {
        self.m_whiteListedIPs.update(JHashSet::clear);
        match readAllLines(&self.m_whitelistPlayersFile)
        {
            Ok(lines) =>
            {
                for s in lines
                {
                    self.m_whiteListedIPs.update(|set| set.add(s.trim().toLowerCase()));
                }
            }
            Err(exception) => logger().warning(&format!("Failed to load white-list: {exception}")),
        }
    }

    fn saveWhiteList(&self)
    {
        if let Err(exception) =
            writeAllLines(&self.m_whitelistPlayersFile, &self.m_whiteListedIPs.with(JHashSet::items))
        {
            logger().warning(&format!("Failed to save white-list: {exception}"));
        }
    }

    pub fn isAllowedToLogin(&self, name: &JString) -> bool
    {
        let name: JString = name.trim().toLowerCase();
        !self.m_whiteListEnforced.get()
            || self.m_ops.with(|set| set.contains(&name))
            || self.m_whiteListedIPs.with(|set| set.contains(&name))
    }

    pub fn isOp(&self, name: Option<&JString>) -> bool
    {
        self.m_ops.with(|set| set.contains(&name.orNpe().trim().toLowerCase()))
    }

    pub fn getPlayerEntity(&self, name: &JString) -> Option<Rc<Entity>>
    {
        let mut i: i32 = 0;
        while i < self.m_playerEntities.size()
        {
            let entityplayermp: Rc<Entity> = self.m_playerEntities.get(i);
            if entity_player::fields(&entityplayermp).m_username.get().orNpe().equalsIgnoreCase(Some(name))
            {
                return Some(entityplayermp);
            }
            i += 1;
        }
        None
    }

    pub fn sendChatMessageToPlayer(&self, name: &JString, message: &JString)
    {
        if let Some(entityplayermp) = self.getPlayerEntity(name)
        {
            entity_player_mp::netServerHandler(&entityplayermp).sendPacket(Box::new(Packet3Chat::new(message.clone())));
        }
    }

    pub fn sendPacketToPlayersAroundPoint(
        &self,
        x: f64,
        y: f64,
        z: f64,
        range: f64,
        dimension: i32,
        packet: Box<dyn Packet>,
    )
    {
        self.sendPacket(None, x, y, z, range, dimension, packet);
    }

    pub fn sendPacket(
        &self,
        source: Option<&Rc<Entity>>,
        x: f64,
        y: f64,
        z: f64,
        range: f64,
        dimension: i32,
        packet: Box<dyn Packet>,
    )
    {
        let mut i: i32 = 0;
        while i < self.m_playerEntities.size()
        {
            let entityplayermp: Rc<Entity> = self.m_playerEntities.get(i);
            let isSource: bool = source.is_some_and(|source| Rc::ptr_eq(source, &entityplayermp));
            if !isSource && entity_player::fields(&entityplayermp).m_dimension.get() == dimension
            {
                let d0: f64 = x - entityplayermp.m_posX.get();
                let d1: f64 = y - entityplayermp.m_posY.get();
                let d2: f64 = z - entityplayermp.m_posZ.get();
                if d0 * d0 + d1 * d1 + d2 * d2 < range * range
                {
                    entity_player_mp::netServerHandler(&entityplayermp).sendPacket(packet.clonePacket());
                }
            }
            i += 1;
        }
    }

    pub fn sendChatMessageToAllOps(&self, message: &JString)
    {
        let packet3chat: Packet3Chat = Packet3Chat::new(message.clone());
        let mut i: i32 = 0;
        while i < self.m_playerEntities.size()
        {
            let entityplayermp: Rc<Entity> = self.m_playerEntities.get(i);
            if self.isOp(entity_player::fields(&entityplayermp).m_username.get().as_ref())
            {
                entity_player_mp::netServerHandler(&entityplayermp).sendPacket(Box::new(packet3chat.clone()));
            }
            i += 1;
        }
    }

    pub fn sendPacketToPlayer(&self, player: &JString, packet: Box<dyn Packet>) -> bool
    {
        match self.getPlayerEntity(player)
        {
            Some(entityplayermp) =>
            {
                entity_player_mp::netServerHandler(&entityplayermp).sendPacket(packet);
                true
            }
            None => false,
        }
    }

    pub fn savePlayerStates(&self)
    {
        let mut i: i32 = 0;
        while i < self.m_playerEntities.size()
        {
            self.playerNBTManagerObj().writePlayerData(&self.m_playerEntities.get(i));
            i += 1;
        }
    }

    pub fn sentTileEntityToPlayer(&self, _x: i32, _y: i32, _z: i32, _blockEntity: &Rc<TileEntity>) {}

    pub fn addToWhiteList(&self, name: &JString)
    {
        self.m_whiteListedIPs.update(|set| set.add(name.clone()));
        self.saveWhiteList();
    }

    pub fn removeFromWhiteList(&self, name: &JString)
    {
        self.m_whiteListedIPs.update(|set| set.remove(name));
        self.saveWhiteList();
    }

    pub fn getWhiteListedIPs(&self) -> Vec<JString> { self.m_whiteListedIPs.with(JHashSet::items) }

    pub fn reloadWhiteList(&self) { self.loadWhiteList(); }

    pub fn sendWorldInfo(&self, player: &Rc<Entity>, world: &Rc<World>)
    {
        let handler: Rc<NetServerHandler> = entity_player_mp::netServerHandler(player);
        handler.sendPacket(Box::new(Packet4UpdateTime::new(world.getWorldTime())));
        if world.isRaining()
        {
            handler.sendPacket(Box::new(Packet70Bed::new(1)));
        }
    }

    pub fn sendPlayerInfo(&self, player: &Rc<Entity>)
    {
        entity_player_mp::setMenu(player, &entity_player::fields(player).m_inventorySlots.get().orNpe());
        entity_player_mp::markHealthDirty(player);
    }
}
