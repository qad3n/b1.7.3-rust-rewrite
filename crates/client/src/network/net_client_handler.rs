use crate::client::Minecraft;
use crate::controller::player_controller_mp;
use crate::entity::{entity_client_player_mp, entity_other_player_mp};
use crate::gui::{GuiConnectFailed, GuiDownloadTerrain};
// the java (EntityPlayer) cast is checked in port
use crate::port::network::castToPlayer;
use crate::render::particle::entity_pickup_fx;
use crate::world::world_client;
use game::block::Block;
use game::container::Container;
use game::entity::{
    Entity, EntityList, entity_arrow, entity_boat, entity_egg, entity_falling_sand, entity_fireball, entity_fish,
    entity_item, entity_lightning_bolt, entity_living, entity_minecart, entity_painting, entity_player,
    entity_snowball, entity_tnt_primed,
};
use game::inventory::{IInventory, InventoryBasic, InventoryPlayer};
use game::item::{Item, ItemStack, item_map};
use game::math::math_helper;
use game::network::net_handler::NetHandler;
use game::network::network_manager::NetworkManager;
use game::network::packet::Packet;
use game::network::packets::*;
// item stacks cross the network threads as plain port data
use game::port::packet_item_stack::PacketItemStack;
use game::stats::StatList;
use game::storage::SaveHandlerMP;
use game::tileentity::{TileEntity, tile_entity_dispenser, tile_entity_furnace, tile_entity_sign};
use game::world::{ChunkCoordinates, Explosion, MapDataBase, MapStorage, World};
use jvm::field::Field;
use jvm::io::{IoResult, ioError};
use jvm::random::Random;
use jvm::string::JString;
use jvm::throwable::{EXCEPTION, IO, OrNpe, Throwable, ThrowableClass, catchThrowable};
use std::cell::Cell;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::rc::{Rc, Weak};

pub static UNKNOWN_HOST: ThrowableClass =
    ThrowableClass { m_name: "java.net.UnknownHostException", m_parent: Some(&IO) };
pub static CONNECT: ThrowableClass = ThrowableClass { m_name: "java.net.ConnectException", m_parent: Some(&IO) };

pub struct NetClientHandler
{
    m_self: Weak<NetClientHandler>,
    m_disconnected: Cell<bool>,
    m_netManager: Field<Option<Rc<NetworkManager>>>,
    pub m_message: Field<Option<JString>>,
    m_mc: Rc<Minecraft>,
    m_worldClient: Field<Option<Rc<World>>>,
    m_started: Cell<bool>,
    pub m_savedDataStorage: Rc<MapStorage>,
    m_rand: Random,
}

impl NetClientHandler
{
    pub fn new(minecraft: &Rc<Minecraft>, address: &str, port: i32) -> IoResult<Rc<NetClientHandler>>
    {
        Self::fromSocket(minecraft, Self::connect(address, port)?)
    }

    pub fn connect(address: &str, port: i32) -> IoResult<TcpStream>
    {
        let host: SocketAddr = (address, port as u16)
            .to_socket_addrs()
            .ok()
            .and_then(|mut addresses| addresses.next())
            .ok_or_else(|| ioError(&UNKNOWN_HOST, address))?;
        TcpStream::connect(host).map_err(|error| match error.kind()
        {
            std::io::ErrorKind::ConnectionRefused => ioError(&CONNECT, "Connection refused: connect"),
            _ => jvm::io::fromStd(&error),
        })
    }

    pub fn fromSocket(minecraft: &Rc<Minecraft>, socket: TcpStream) -> IoResult<Rc<NetClientHandler>>
    {
        let this: Rc<NetClientHandler> = Rc::new_cyclic(|this| NetClientHandler {
            m_self: Weak::clone(this),
            m_disconnected: Cell::new(false),
            m_netManager: Field::new(None),
            m_message: Field::new(None),
            m_mc: Rc::clone(minecraft),
            m_worldClient: Field::new(None),
            m_started: Cell::new(false),
            m_savedDataStorage: Rc::new(MapStorage::new(Rc::new(SaveHandlerMP))),
            m_rand: Random::new(),
        });
        let handler: Weak<dyn NetHandler> = Rc::downgrade(&this) as Weak<dyn NetHandler>;
        this.m_netManager.set(Some(NetworkManager::new(socket, "Client", handler, false)?));
        Ok(this)
    }

    fn netManager(&self) -> Rc<NetworkManager> { self.m_netManager.get().orNpe() }

    fn worldClient(&self) -> Rc<World> { self.m_worldClient.get().orNpe() }

    fn thePlayer(&self) -> Rc<Entity> { self.m_mc.m_thePlayer.get().orNpe() }

    fn theWorld(&self) -> Rc<World> { self.m_mc.m_theWorld.get().orNpe() }

    fn rc(&self) -> Rc<NetClientHandler> { self.m_self.upgrade().expect("a live net client handler is owned by an Rc") }

    pub fn processReadPackets(&self)
    {
        if !self.m_disconnected.get()
        {
            self.netManager().processReadPackets();
        }
        self.netManager().wakeThreads();
    }

    pub fn sendPacketAndDisconnect(&self, packet: Box<dyn Packet>)
    {
        if self.m_disconnected.get()
        {
            return;
        }
        self.netManager().addToSendQueue(packet);
        self.netManager().close();
    }

    pub fn addToSendQueue(&self, packet: Box<dyn Packet>)
    {
        if self.m_disconnected.get()
        {
            return;
        }
        self.netManager().addToSendQueue(packet);
    }

    pub fn disconnect(&self)
    {
        self.m_disconnected.set(true);
        self.netManager().wakeThreads();
        self.netManager().networkShutdown("disconnect.closed", Vec::new());
    }

    fn getEntityByID(&self, networkId: i32) -> Option<Rc<Entity>>
    {
        let thePlayer: Rc<Entity> = self.thePlayer();
        if networkId == thePlayer.m_entityId.get()
        {
            return Some(thePlayer);
        }
        world_client::getEntity(&self.worldClient(), networkId)
    }

    fn changeToWorldClient(self: &Rc<Self>, seed: i64, dimension: i8)
    {
        let worldClient: Rc<World> = world_client::new(self, seed, dimension as i32);
        worldClient.m_multiplayerWorld.set(true);
        self.m_worldClient.set(Some(Rc::clone(&worldClient)));
        self.m_mc.changeWorld1(Some(worldClient));
        entity_player::fields(&self.thePlayer()).m_dimension.set(dimension as i32);
        self.m_mc.displayGuiScreen(Some(GuiDownloadTerrain::new(self)));
    }

    fn joinServer(&self, serverId: &JString)
    {
        let session: Rc<crate::client::Session> = self.m_mc.m_session.get().orNpe();
        let result: Result<IoResult<(JString, bool)>, Throwable> = catchThrowable(&EXCEPTION, || {
            let url: String = format!(
                "http://www.minecraft.net/game/joinserver.jsp?user={}&sessionId={}&serverId={serverId}",
                session.m_username.get().unwrap_or_else(|| "null".to_string()),
                session.m_sessionId.get().unwrap_or_else(|| "null".to_string()),
            );
            let body: Vec<u8> = jvm::net::openStream(&url)?;
            let text: String = String::from_utf8_lossy(&body).into_owned();
            let line: Option<JString> =
                (!text.is_empty()).then(|| JString::from(text.split(['\n', '\r']).next().unwrap_or("")));
            let string: JString = line.orNpe();
            let ok: bool = string.equalsIgnoreCase(Some(&JString::from("ok")));
            Ok((string, ok))
        });
        match result.and_then(|line| line)
        {
            Ok((_, true)) =>
            {
                let username: Option<String> = session.m_username.get();
                self.addToSendQueue(Box::new(Packet1Login::new(JString::from(username.orNpe().as_str()), 14)));
            }
            Ok((string, false)) => self.netManager().networkShutdown("disconnect.loginFailedInfo", vec![string]),
            Err(exception) =>
            {
                exception.printStackTrace();
                self.netManager().networkShutdown(
                    "disconnect.genericReason",
                    vec![JString::from(format!("Internal client error: {exception}").as_str())],
                );
            }
        }
    }

    fn openWindow(&self, windowId: i32)
    {
        let craftingInventory: Rc<Container> =
            entity_player::fields(&self.thePlayer()).m_craftingInventory.get().orNpe();
        craftingInventory.m_windowId.set(windowId);
    }
}

impl NetHandler for NetClientHandler
{
    fn isServerHandler(&self) -> bool { false }

    fn handleLogin(&self, packet: &Packet1Login)
    {
        let this: Rc<NetClientHandler> = self.rc();
        let mc: &Rc<Minecraft> = &self.m_mc;
        mc.m_playerController.set(Some(player_controller_mp::new(mc, &this)));
        mc.m_statFileWriter.get().orNpe().readStat(&StatList::joinMultiplayerStat().orNpe(), 1);
        this.changeToWorldClient(packet.m_mapSeed, packet.m_dimension);
        self.thePlayer().m_entityId.set(packet.m_protocolVersion);
    }

    fn handlePickupSpawn(&self, packet: &Packet21PickupSpawn)
    {
        let d: f64 = packet.m_xPosition as f64 / 32.0;
        let d2: f64 = packet.m_yPosition as f64 / 32.0;
        let d3: f64 = packet.m_zPosition as f64 / 32.0;
        let entityItem: Rc<Entity> = entity_item::create(
            &self.worldClient(),
            d,
            d2,
            d3,
            ItemStack::withDamage(packet.m_itemID, packet.m_count, packet.m_itemDamage),
        );
        entityItem.m_motionX.set(packet.m_rotation as f64 / 128.0);
        entityItem.m_motionY.set(packet.m_pitch as f64 / 128.0);
        entityItem.m_motionZ.set(packet.m_roll as f64 / 128.0);
        entityItem.m_serverPosX.set(packet.m_xPosition);
        entityItem.m_serverPosY.set(packet.m_yPosition);
        entityItem.m_serverPosZ.set(packet.m_zPosition);
        world_client::forceEntity(&self.worldClient(), packet.m_entityId, &entityItem);
    }

    fn handleVehicleSpawn(&self, packet: &Packet23VehicleSpawn)
    {
        let world: Rc<World> = self.worldClient();
        let d0: f64 = packet.m_xPosition as f64 / 32.0;
        let d1: f64 = packet.m_yPosition as f64 / 32.0;
        let d2: f64 = packet.m_zPosition as f64 / 32.0;
        let velocityX: f64 = packet.m_velocityX as f64 / 8000.0;
        let velocityY: f64 = packet.m_velocityY as f64 / 8000.0;
        let velocityZ: f64 = packet.m_velocityZ as f64 / 8000.0;
        let mut data: i32 = packet.m_data;
        let mut entity: Option<Rc<Entity>> = None;
        match packet.m_type
        {
            10 => entity = Some(entity_minecart::createAt(&world, d0, d1, d2, 0)),
            11 => entity = Some(entity_minecart::createAt(&world, d0, d1, d2, 1)),
            12 => entity = Some(entity_minecart::createAt(&world, d0, d1, d2, 2)),
            90 => entity = Some(entity_fish::create(&world, d0, d1, d2)),
            60 => entity = Some(entity_arrow::create(&world, d0, d1, d2)),
            61 => entity = Some(entity_snowball::create(&world, d0, d1, d2)),
            63 =>
            {
                entity = Some(entity_fireball::createAt(&world, d0, d1, d2, velocityX, velocityY, velocityZ));
                data = 0;
            }
            62 => entity = Some(entity_egg::create(&world, d0, d1, d2)),
            1 => entity = Some(entity_boat::createAt(&world, d0, d1, d2)),
            50 => entity = Some(entity_tnt_primed::create(&world, d0, d1, d2)),
            70 => entity = Some(entity_falling_sand::create(&world, d0, d1, d2, Block::sand().m_blockID)),
            71 => entity = Some(entity_falling_sand::create(&world, d0, d1, d2, Block::gravel().m_blockID)),
            _ =>
            {}
        }
        let Some(entity) = entity
        else
        {
            return;
        };
        entity.m_serverPosX.set(packet.m_xPosition);
        entity.m_serverPosY.set(packet.m_yPosition);
        entity.m_serverPosZ.set(packet.m_zPosition);
        entity.m_rotationYaw.set(0.0);
        entity.m_rotationPitch.set(0.0);
        entity.m_entityId.set(packet.m_entityId);
        world_client::forceEntity(&world, packet.m_entityId, &entity);
        if data > 0
        {
            if packet.m_type == 60
                && let Some(entity1) = self.getEntityByID(data)
                && entity1.instanceOf("EntityLiving")
            {
                entity_arrow::fields(&entity).m_owner.set(Some(entity1));
            }
            entity.setVelocity(velocityX, velocityY, velocityZ);
        }
    }

    fn handleWeather(&self, packet: &Packet71Weather)
    {
        let x: f64 = packet.m_x as f64 / 32.0;
        let y: f64 = packet.m_y as f64 / 32.0;
        let z: f64 = packet.m_z as f64 / 32.0;
        if packet.m_type != 1
        {
            return;
        }
        let entity: Rc<Entity> = entity_lightning_bolt::create(&self.worldClient(), x, y, z);
        entity.m_serverPosX.set(packet.m_x);
        entity.m_serverPosY.set(packet.m_y);
        entity.m_serverPosZ.set(packet.m_z);
        entity.m_rotationYaw.set(0.0);
        entity.m_rotationPitch.set(0.0);
        entity.m_entityId.set(packet.m_id);
        self.worldClient().addWeatherEffect(entity);
    }

    fn handleAddPainting(&self, packet: &Packet25EntityPainting)
    {
        let entityPainting: Rc<Entity> = entity_painting::createWithMotive(
            &self.worldClient(),
            packet.m_xPosition,
            packet.m_yPosition,
            packet.m_zPosition,
            packet.m_direction,
            packet.m_title.as_ref().orNpe(),
        );
        world_client::forceEntity(&self.worldClient(), packet.m_entityId, &entityPainting);
    }

    fn handleEntityVelocity(&self, packet: &Packet28EntityVelocity)
    {
        let Some(entity) = self.getEntityByID(packet.m_entityId)
        else
        {
            return;
        };
        entity.setVelocity(
            packet.m_motionX as f64 / 8000.0,
            packet.m_motionY as f64 / 8000.0,
            packet.m_motionZ as f64 / 8000.0,
        );
    }

    fn handleEntityData(&self, packet: &Packet40EntityMetadata)
    {
        if let Some(entity) = self.getEntityByID(packet.m_entityId)
            && let Some(entries) = packet.getDataEntries()
        {
            entity.getDataWatcher().updateWatchedObjectsFromList(&entries);
        }
    }

    fn handleNamedEntitySpawn(&self, packet: &Packet20NamedEntitySpawn)
    {
        let d0: f64 = packet.m_xPosition as f64 / 32.0;
        let d1: f64 = packet.m_yPosition as f64 / 32.0;
        let d2: f64 = packet.m_zPosition as f64 / 32.0;
        let f: f32 = (packet.m_rotation as i32 * 360) as f32 / 256.0;
        let f1: f32 = (packet.m_pitch as i32 * 360) as f32 / 256.0;
        let entityotherplayermp: Rc<Entity> = entity_other_player_mp::new(&self.theWorld(), packet.m_name.clone());
        entityotherplayermp.m_serverPosX.set(packet.m_xPosition);
        entityotherplayermp.m_lastTickPosX.set(packet.m_xPosition as f64);
        entityotherplayermp.m_prevPosX.set(packet.m_xPosition as f64);
        entityotherplayermp.m_serverPosY.set(packet.m_yPosition);
        entityotherplayermp.m_lastTickPosY.set(packet.m_yPosition as f64);
        entityotherplayermp.m_prevPosY.set(packet.m_yPosition as f64);
        entityotherplayermp.m_serverPosZ.set(packet.m_zPosition);
        entityotherplayermp.m_lastTickPosZ.set(packet.m_zPosition as f64);
        entityotherplayermp.m_prevPosZ.set(packet.m_zPosition as f64);
        let i: i32 = packet.m_currentItem;
        let inventory: Rc<InventoryPlayer> = entity_player::inventory(&entityotherplayermp);
        let currentItem: usize = inventory.m_currentItem.get() as usize;
        inventory.m_mainInventory.borrow_mut()[currentItem] = if i == 0
        {
            None
        }
        else
        {
            Some(ItemStack::withDamage(i, 1, 0))
        };
        entityotherplayermp.setPositionAndRotation(d0, d1, d2, f, f1);
        world_client::forceEntity(&self.worldClient(), packet.m_entityId, &entityotherplayermp);
    }

    fn handleEntityTeleport(&self, packet: &Packet34EntityTeleport)
    {
        let Some(entity) = self.getEntityByID(packet.m_entityId)
        else
        {
            return;
        };
        entity.m_serverPosX.set(packet.m_xPosition);
        entity.m_serverPosY.set(packet.m_yPosition);
        entity.m_serverPosZ.set(packet.m_zPosition);
        let d: f64 = entity.m_serverPosX.get() as f64 / 32.0;
        let d2: f64 = entity.m_serverPosY.get() as f64 / 32.0 + 0.015625;
        let d3: f64 = entity.m_serverPosZ.get() as f64 / 32.0;
        entity.setPositionAndRotation2(
            d,
            d2,
            d3,
            (packet.m_yaw as i32 * 360) as f32 / 256.0,
            (packet.m_pitch as i32 * 360) as f32 / 256.0,
            3,
        );
    }

    fn handleEntity(&self, packet: &Packet30Entity)
    {
        let Some(entity) = self.getEntityByID(packet.m_entityId)
        else
        {
            return;
        };
        entity.m_serverPosX.set(entity.m_serverPosX.get() + packet.m_xPosition as i32);
        entity.m_serverPosY.set(entity.m_serverPosY.get() + packet.m_yPosition as i32);
        entity.m_serverPosZ.set(entity.m_serverPosZ.get() + packet.m_zPosition as i32);
        let d: f64 = entity.m_serverPosX.get() as f64 / 32.0;
        let d2: f64 = entity.m_serverPosY.get() as f64 / 32.0;
        let d3: f64 = entity.m_serverPosZ.get() as f64 / 32.0;
        let f: f32 = if packet.m_rotating
        {
            (packet.m_yaw as i32 * 360) as f32 / 256.0
        }
        else
        {
            entity.m_rotationYaw.get()
        };
        let f2: f32 = if packet.m_rotating
        {
            (packet.m_pitch as i32 * 360) as f32 / 256.0
        }
        else
        {
            entity.m_rotationPitch.get()
        };
        entity.setPositionAndRotation2(d, d2, d3, f, f2, 3);
    }

    fn handleDestroyEntity(&self, packet: &Packet29DestroyEntity)
    {
        world_client::removeEntityFromWorld(&self.worldClient(), packet.m_entityId);
    }

    fn handleFlying(&self, packet: &Packet10Flying)
    {
        let entityPlayerSP: Rc<Entity> = self.thePlayer();
        let mut d: f64 = entityPlayerSP.posX();
        let mut d2: f64 = entityPlayerSP.posY();
        let mut d3: f64 = entityPlayerSP.posZ();
        let mut f: f32 = entityPlayerSP.m_rotationYaw.get();
        let mut f2: f32 = entityPlayerSP.m_rotationPitch.get();
        if packet.m_moving
        {
            d = packet.m_xPosition;
            d2 = packet.m_yPosition;
            d3 = packet.m_zPosition;
        }
        if packet.m_rotating
        {
            f = packet.m_yaw;
            f2 = packet.m_pitch;
        }
        entityPlayerSP.m_ySize.set(0.0);
        entityPlayerSP.m_motionZ.set(0.0);
        entityPlayerSP.m_motionY.set(0.0);
        entityPlayerSP.m_motionX.set(0.0);
        entityPlayerSP.setPositionAndRotation(d, d2, d3, f, f2);
        let mut reply: Packet10Flying = packet.clone();
        reply.m_xPosition = entityPlayerSP.posX();
        reply.m_yPosition = entityPlayerSP.m_boundingBox.minY();
        reply.m_zPosition = entityPlayerSP.posZ();
        reply.m_stance = entityPlayerSP.posY();
        self.netManager().addToSendQueue(reply.intoPacket());
        if !self.m_started.get()
        {
            let thePlayer: Rc<Entity> = self.thePlayer();
            thePlayer.m_prevPosX.set(thePlayer.posX());
            thePlayer.m_prevPosY.set(thePlayer.posY());
            thePlayer.m_prevPosZ.set(thePlayer.posZ());
            self.m_started.set(true);
            self.m_mc.displayGuiScreen(None);
        }
    }

    fn handlePreChunk(&self, packet: &Packet50PreChunk)
    {
        world_client::doPreChunk(&self.worldClient(), packet.m_xPosition, packet.m_yPosition, packet.m_mode);
    }

    fn handleMultiBlockChange(&self, packet: &Packet52MultiBlockChange)
    {
        let world: Rc<World> = self.worldClient();
        let chunk = world.getChunkFromChunkCoords(packet.m_xPosition, packet.m_zPosition);
        let n: i32 = packet.m_xPosition * 16;
        let n2: i32 = packet.m_zPosition * 16;
        let coordinateArray: &Vec<i16> = packet.m_coordinateArray.as_ref().orNpe();
        let typeArray: &Vec<u8> = packet.m_typeArray.as_ref().orNpe();
        let metadataArray: &Vec<u8> = packet.m_metadataArray.as_ref().orNpe();
        for i in 0..packet.m_size as usize
        {
            let s: i16 = coordinateArray[i];
            let n3: i32 = typeArray[i] as i32 & 0xFF;
            let by: i8 = metadataArray[i] as i8;
            let n4: i32 = (s as i32) >> 12 & 0xF;
            let n5: i32 = (s as i32) >> 8 & 0xF;
            let n6: i32 = s as i32 & 0xFF;
            chunk.setBlockIDWithMetadata(n4, n6, n5, n3, by as i32);
            world_client::clearBlockResets(&world, n4 + n, n6, n5 + n2, n4 + n, n6, n5 + n2);
            world.markBlocksDirty(n4 + n, n6, n5 + n2, n4 + n, n6, n5 + n2);
        }
    }

    fn handleMapChunk(&self, packet: &Packet51MapChunk)
    {
        let world: Rc<World> = self.worldClient();
        world_client::clearBlockResets(
            &world,
            packet.m_xPosition,
            packet.m_yPosition,
            packet.m_zPosition,
            packet.m_xPosition + packet.m_xSize - 1,
            packet.m_yPosition + packet.m_ySize - 1,
            packet.m_zPosition + packet.m_zSize - 1,
        );
        let chunk: Vec<i8> = packet.m_chunk.as_ref().orNpe().iter().map(|value| *value as i8).collect();
        world.setChunkData(
            packet.m_xPosition,
            packet.m_yPosition,
            packet.m_zPosition,
            packet.m_xSize,
            packet.m_ySize,
            packet.m_zSize,
            &chunk,
        );
    }

    fn handleBlockChange(&self, packet: &Packet53BlockChange)
    {
        world_client::setBlockWithMetadataFromPacket(
            &self.worldClient(),
            packet.m_xPosition,
            packet.m_yPosition,
            packet.m_zPosition,
            packet.m_type,
            packet.m_metadata,
        );
    }

    fn handleKickDisconnect(&self, packet: &Packet255KickDisconnect)
    {
        self.netManager().networkShutdown("disconnect.kicked", Vec::new());
        self.m_disconnected.set(true);
        self.m_mc.changeWorld1(None);
        self.m_mc.displayGuiScreen(Some(GuiConnectFailed::new(
            "disconnect.disconnected",
            "disconnect.genericReason",
            &[packet.m_reason.clone().orNpe()],
        )));
    }

    fn handleErrorMessage(&self, reason: &JString, args: &[JString])
    {
        if self.m_disconnected.get()
        {
            return;
        }
        self.m_disconnected.set(true);
        self.m_mc.changeWorld1(None);
        self.m_mc.displayGuiScreen(Some(GuiConnectFailed::new("disconnect.lost", &reason.to_string(), args)));
    }

    fn handleCollect(&self, packet: &Packet22Collect)
    {
        let entity: Option<Rc<Entity>> = self.getEntityByID(packet.m_collectedEntityId);
        let collector: Option<Rc<Entity>> = self.getEntityByID(packet.m_collectorEntityId);
        if let Some(collector) = &collector
            && !collector.instanceOf("EntityLiving")
        {
            jvm::throwable::throw(
                &jvm::throwable::CLASS_CAST,
                &format!(
                    "net.minecraft.src.{} cannot be cast to net.minecraft.src.EntityLiving",
                    collector.className()
                ),
            );
        }
        let entityLiving: Rc<Entity> = collector.unwrap_or_else(|| self.thePlayer());
        if let Some(entity) = entity
        {
            let pitch: f32 = ((self.m_rand.nextFloat() - self.m_rand.nextFloat()) * 0.7 + 1.0) * 2.0;
            self.worldClient().playSoundAtEntity(&entity, "random.pop", 0.2, pitch);
            let fx: Rc<Entity> = entity_pickup_fx::new(&self.theWorld(), &entity, &entityLiving, -0.5);
            self.m_mc.m_effectRenderer.get().orNpe().addEffect(fx);
            world_client::removeEntityFromWorld(&self.worldClient(), packet.m_collectedEntityId);
        }
    }

    fn handleChat(&self, packet: &Packet3Chat)
    {
        self.m_mc.m_ingameGUI.get().orNpe().addChatMessage(&packet.m_message.as_ref().orNpe().to_string());
    }

    fn handleArmAnimation(&self, packet: &Packet18Animation)
    {
        let Some(entity) = self.getEntityByID(packet.m_entityId)
        else
        {
            return;
        };
        match packet.m_animate
        {
            1 =>
            {
                castToPlayer(&entity);
                entity.swingItem();
            }
            2 => entity.performHurtAnimation(),
            3 =>
            {
                castToPlayer(&entity);
                entity.wakeUpPlayer(false, false, false);
            }
            4 =>
            {
                castToPlayer(&entity);
                entity.addRespawnParticles();
            }
            _ =>
            {}
        }
    }

    fn handlePlayerSleep(&self, packet: &Packet17Sleep)
    {
        let Some(entity) = self.getEntityByID(packet.m_id)
        else
        {
            return;
        };
        if packet.m_action == 0
        {
            castToPlayer(&entity);
            entity.sleepInBedAt(packet.m_x, packet.m_y, packet.m_z);
        }
    }

    fn handleHandshake(&self, packet: &Packet2Handshake)
    {
        let serverId: &JString = packet.m_username.as_ref().orNpe();
        if *serverId == JString::from("-")
        {
            let username: Option<String> = self.m_mc.m_session.get().orNpe().m_username.get();
            self.addToSendQueue(Box::new(Packet1Login::new(JString::from(username.orNpe().as_str()), 14)));
        }
        else
        {
            self.joinServer(serverId);
        }
    }

    fn handleMobSpawn(&self, packet: &Packet24MobSpawn)
    {
        let d: f64 = packet.m_xPosition as f64 / 32.0;
        let d2: f64 = packet.m_yPosition as f64 / 32.0;
        let d3: f64 = packet.m_zPosition as f64 / 32.0;
        let f: f32 = (packet.m_yaw as i32 * 360) as f32 / 256.0;
        let f2: f32 = (packet.m_pitch as i32 * 360) as f32 / 256.0;
        let entityLiving: Rc<Entity> = EntityList::createEntity(packet.m_type as i32, &self.theWorld()).orNpe();
        entity_living::fields(&entityLiving);
        entityLiving.m_serverPosX.set(packet.m_xPosition);
        entityLiving.m_serverPosY.set(packet.m_yPosition);
        entityLiving.m_serverPosZ.set(packet.m_zPosition);
        entityLiving.m_entityId.set(packet.m_entityId);
        entityLiving.setPositionAndRotation(d, d2, d3, f, f2);
        entity_living::fields(&entityLiving).m_isMultiplayerEntity.set(true);
        world_client::forceEntity(&self.worldClient(), packet.m_entityId, &entityLiving);
        if let Some(list) = packet.getMetadata()
        {
            entityLiving.getDataWatcher().updateWatchedObjectsFromList(&list);
        }
    }

    fn handleUpdateTime(&self, packet: &Packet4UpdateTime) { self.theWorld().setWorldTime(packet.m_time); }

    fn handleSpawnPosition(&self, packet: &Packet6SpawnPosition)
    {
        entity_player::setPlayerSpawnCoordinate(
            &self.thePlayer(),
            Some(ChunkCoordinates::new(packet.m_xPosition, packet.m_yPosition, packet.m_zPosition)),
        );
        self.theWorld().worldInfo().setSpawn(packet.m_xPosition, packet.m_yPosition, packet.m_zPosition);
    }

    fn handleRideEntity(&self, packet: &Packet39AttachEntity)
    {
        let mut entity: Option<Rc<Entity>> = self.getEntityByID(packet.m_entityId);
        let entity2: Option<Rc<Entity>> = self.getEntityByID(packet.m_vehicleEntityId);
        if packet.m_entityId == self.thePlayer().m_entityId.get()
        {
            entity = Some(self.thePlayer());
        }
        let Some(entity) = entity
        else
        {
            return;
        };
        entity.mountEntity(entity2.as_ref());
    }

    fn handleEntityEvent(&self, packet: &Packet38EntityStatus)
    {
        if let Some(entity) = self.getEntityByID(packet.m_entityId)
        {
            entity.handleHealthUpdate(packet.m_entityStatus);
        }
    }

    fn handleHealth(&self, packet: &Packet8UpdateHealth) { self.thePlayer().setHealth(packet.m_healthMP); }

    fn handlePlayerRespawn(&self, packet: &Packet9Respawn)
    {
        if packet.m_dimension as i32 != entity_player::fields(&self.thePlayer()).m_dimension.get()
        {
            self.m_started.set(false);
            let seed: i64 = self.worldClient().worldInfo().getRandomSeed();
            self.rc().changeToWorldClient(seed, packet.m_dimension);
        }
        self.m_mc.respawn(true, packet.m_dimension as i32);
    }

    fn handleExplosion(&self, packet: &Packet60Explosion)
    {
        let explosion: Rc<Explosion> = Explosion::new(
            &self.theWorld(),
            None,
            packet.m_explosionX,
            packet.m_explosionY,
            packet.m_explosionZ,
            packet.m_explosionSize,
        );
        explosion.m_destroyedBlockPositions.set(packet.m_destroyedBlockPositions.take().orNpe());
        explosion.doExplosionB(true);
    }

    fn handleOpenInventoryMenu(&self, packet: &Packet100OpenWindow)
    {
        let thePlayer: Rc<Entity> = self.thePlayer();
        match packet.m_inventoryType
        {
            0 =>
            {
                let inventoryBasic: Rc<dyn IInventory> =
                    Rc::new(InventoryBasic::new(packet.m_windowTitle.clone().orNpe(), packet.m_slotsCount));
                thePlayer.displayGUIChest(Some(&inventoryBasic));
                self.openWindow(packet.m_windowId);
            }
            2 =>
            {
                let tileEntityFurnace: Rc<TileEntity> = tile_entity_furnace::create();
                thePlayer.displayGUIFurnace(Some(&tileEntityFurnace));
                self.openWindow(packet.m_windowId);
            }
            3 =>
            {
                let tileEntityDispenser: Rc<TileEntity> = tile_entity_dispenser::create();
                thePlayer.displayGUIDispenser(Some(&tileEntityDispenser));
                self.openWindow(packet.m_windowId);
            }
            1 =>
            {
                thePlayer.displayWorkbenchGUI(
                    math_helper::floor_double(thePlayer.posX()),
                    math_helper::floor_double(thePlayer.posY()),
                    math_helper::floor_double(thePlayer.posZ()),
                );
                self.openWindow(packet.m_windowId);
            }
            _ =>
            {}
        }
    }

    fn handleInventoryMenuSlotContent(&self, packet: &Packet103SetSlot)
    {
        let thePlayer: Rc<Entity> = self.thePlayer();
        let player: &entity_player::PlayerFields = entity_player::fields(&thePlayer);
        let myItemStack: Option<Rc<ItemStack>> = PacketItemStack::toItemStackOption(packet.m_myItemStack);
        if packet.m_windowId == -1
        {
            entity_player::inventory(&thePlayer).setItemStack(myItemStack);
        }
        else if packet.m_windowId == 0 && packet.m_itemSlot >= 36 && packet.m_itemSlot < 45
        {
            let inventorySlots: Rc<Container> = player.m_inventorySlots.get().orNpe();
            let itemStack: Option<Rc<ItemStack>> = inventorySlots.getSlot(packet.m_itemSlot).getStack();
            if let Some(myItemStack) = &myItemStack
                && itemStack.is_none_or(|stack| stack.m_stackSize.get() < myItemStack.m_stackSize.get())
            {
                myItemStack.m_animationsToGo.set(5);
            }
            inventorySlots.putStackInSlot(packet.m_itemSlot, myItemStack);
        }
        else
        {
            let craftingInventory: Rc<Container> = player.m_craftingInventory.get().orNpe();
            if packet.m_windowId == craftingInventory.m_windowId.get()
            {
                craftingInventory.putStackInSlot(packet.m_itemSlot, myItemStack);
            }
        }
    }

    fn handleInventoryMenuConfirm(&self, packet: &Packet106Transaction)
    {
        let thePlayer: Rc<Entity> = self.thePlayer();
        let player: &entity_player::PlayerFields = entity_player::fields(&thePlayer);
        let mut container: Option<Rc<Container>> = None;
        if packet.m_windowId == 0
        {
            container = player.m_inventorySlots.get();
        }
        else if packet.m_windowId == player.m_craftingInventory.get().orNpe().m_windowId.get()
        {
            container = player.m_craftingInventory.get();
        }
        if let Some(container) = container
        {
            if packet.m_accepted
            {
                container.deleteBackup(packet.m_interactionId);
            }
            else
            {
                container.rollbackToBackup(packet.m_interactionId);
                self.addToSendQueue(Box::new(Packet106Transaction::new(
                    packet.m_windowId,
                    packet.m_interactionId,
                    true,
                )));
            }
        }
    }

    fn handleInventoryMenuContent(&self, packet: &Packet104WindowItems)
    {
        let thePlayer: Rc<Entity> = self.thePlayer();
        let player: &entity_player::PlayerFields = entity_player::fields(&thePlayer);
        let items: Vec<Option<Rc<ItemStack>>> =
            packet.m_itemStack.as_ref().orNpe().iter().map(|item| PacketItemStack::toItemStackOption(*item)).collect();
        if packet.m_windowId == 0
        {
            player.m_inventorySlots.get().orNpe().putStacksInSlots(&items);
        }
        else
        {
            let craftingInventory: Rc<Container> = player.m_craftingInventory.get().orNpe();
            if packet.m_windowId == craftingInventory.m_windowId.get()
            {
                craftingInventory.putStacksInSlots(&items);
            }
        }
    }

    fn handleSignUpdate(&self, packet: &Packet130UpdateSign)
    {
        let world: Rc<World> = self.theWorld();
        if !world.blockExists(packet.m_xPosition, packet.m_yPosition, packet.m_zPosition)
        {
            return;
        }
        if let Some(tileentity) = world.getBlockTileEntity(packet.m_xPosition, packet.m_yPosition, packet.m_zPosition)
            && tileentity.instanceOf("TileEntitySign")
        {
            let signLines: &Vec<Option<JString>> = packet.m_signLines.as_ref().orNpe();
            tile_entity_sign::fields(&tileentity)
                .m_signText
                .update(|signText| signText.clone_from_slice(&signLines[..4]));
            tileentity.onInventoryChanged();
        }
    }

    fn handleInventoryMenuData(&self, packet: &Packet105UpdateProgressbar)
    {
        self.registerPacket(packet);
        if let Some(craftingInventory) = entity_player::fields(&self.thePlayer()).m_craftingInventory.get()
            && craftingInventory.m_windowId.get() == packet.m_windowId
        {
            craftingInventory.setData(packet.m_progressBar, packet.m_progressBarValue);
        }
    }

    fn handlePlayerInventory(&self, packet: &Packet5PlayerInventory)
    {
        if let Some(entity) = self.getEntityByID(packet.m_entityID)
        {
            entity.outfitWithItem(packet.m_slot, packet.m_itemID, packet.m_itemDamage);
        }
    }

    fn handleCloseInventoryMenu(&self, _packet: &Packet101CloseWindow) { self.thePlayer().closeScreen(); }

    fn handleNotePlay(&self, packet: &Packet54PlayNoteBlock)
    {
        self.theWorld().playNoteAt(
            packet.m_xLocation,
            packet.m_yLocation,
            packet.m_zLocation,
            packet.m_instrumentType,
            packet.m_pitch,
        );
    }

    fn handleGameEvent(&self, packet: &Packet70Bed)
    {
        let n: i32 = packet.m_event;
        if n >= 0
            && n < packet70_bed::EVENT_MESSAGES.len() as i32
            && let Some(message) = packet70_bed::EVENT_MESSAGES[n as usize]
        {
            self.thePlayer().addChatMessage(message);
        }
        match n
        {
            1 =>
            {
                self.worldClient().worldInfo().setRaining(true);
                self.worldClient().setRain(1.0);
            }
            2 =>
            {
                self.worldClient().worldInfo().setRaining(false);
                self.worldClient().setRain(0.0);
            }
            _ =>
            {}
        }
    }

    fn handleMapData(&self, packet: &Packet131MapData)
    {
        if packet.m_type as i32 == Item::mapItem().m_shiftedIndex
        {
            let mapData: Rc<MapDataBase> = item_map::getMapData(packet.m_id, &self.theWorld());
            let data: Vec<i8> = packet.m_data.as_ref().orNpe().iter().map(|value| *value as i8).collect();
            mapData.m_map.update(&mapData, &data);
        }
        else
        {
            jvm::system::println(&format!("Unknown itemid: {}", packet.m_id));
        }
    }

    fn handleWorldEvent(&self, packet: &Packet61DoorChange)
    {
        self.theWorld().doEvent(packet.m_event, packet.m_x, packet.m_y, packet.m_z, packet.m_data);
    }

    fn handleStat(&self, packet: &Packet200Statistic)
    {
        let thePlayer: Rc<Entity> = self.thePlayer();
        entity_client_player_mp::incrementRemoteStat(
            &thePlayer,
            StatList::byKey(packet.m_statId).as_ref(),
            packet.m_value,
        );
    }
}
