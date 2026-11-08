use crate::player::entity_player_mp;
use crate::server::{ICommandListener, MinecraftServer};
use crate::world::world_server;
use game::container::{Container, Slot};
use game::entity::{Entity, entity_living, entity_player};
use game::inventory::{IInventory, InventoryPlayer};
use game::item::ItemStack;
use game::locale::ChatAllowedCharacters;
use game::math::{AxisAlignedBB, math_helper};
use game::network::net_handler::NetHandler;
use game::network::network_manager::NetworkManager;
use game::network::packet::Packet;
use game::network::packets::{
    Packet0KeepAlive, Packet3Chat, Packet7UseEntity, Packet9Respawn, Packet10Flying, Packet13PlayerLookMove,
    Packet14BlockDig, Packet15Place, Packet16BlockItemSwitch, Packet18Animation, Packet19EntityAction,
    Packet27Position, Packet53BlockChange, Packet101CloseWindow, Packet102WindowClick, Packet103SetSlot,
    Packet106Transaction, Packet130UpdateSign, Packet255KickDisconnect,
};
use game::port::packet_item_stack::PacketItemStack;
use game::tileentity::{TileEntity, tile_entity_sign};
use game::world::{ChunkCoordinates, World};
use jvm::field::Field;
use jvm::floating_decimal::doubleToString;
use jvm::logging::Logger;
use jvm::string::JString;
use jvm::throwable::OrNpe;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::{Rc, Weak};

fn logger() -> &'static Logger { Logger::getLogger("Minecraft") }

pub struct NetServerHandler
{
    m_self: Weak<NetServerHandler>,
    pub m_netManager: Rc<NetworkManager>,
    pub m_connectionClosed: Cell<bool>,
    m_mcServer: Rc<MinecraftServer>,
    m_playerEntity: Field<Rc<Entity>>,
    m_ticks: Cell<i32>,
    m_lastKeepAliveTime: Cell<i32>,
    m_playerInAirTime: Cell<i32>,
    m_moved: Cell<bool>,
    m_lastPosX: Cell<f64>,
    m_lastPosY: Cell<f64>,
    m_lastPosZ: Cell<f64>,
    m_hasMoved: Cell<bool>,
    m_transactions: RefCell<HashMap<i32, i16>>,
}

impl NetServerHandler
{
    pub fn new(
        server: &Rc<MinecraftServer>,
        connection: &Rc<NetworkManager>,
        player: &Rc<Entity>,
    ) -> Rc<NetServerHandler>
    {
        let handler: Rc<NetServerHandler> = Rc::new_cyclic(|this| NetServerHandler {
            m_self: this.clone(),
            m_connectionClosed: Cell::new(false),
            m_ticks: Cell::new(0),
            m_lastKeepAliveTime: Cell::new(0),
            m_playerInAirTime: Cell::new(0),
            m_moved: Cell::new(false),
            m_lastPosX: Cell::new(0.0),
            m_lastPosY: Cell::new(0.0),
            m_lastPosZ: Cell::new(0.0),
            m_hasMoved: Cell::new(true),
            m_transactions: RefCell::new(HashMap::new()),
            m_mcServer: Rc::clone(server),
            m_netManager: Rc::clone(connection),
            m_playerEntity: Field::new(Rc::clone(player)),
        });
        let listener: Weak<dyn game::network::net_handler::NetHandler> = handler.m_self.clone();
        connection.setNetHandler(listener);
        entity_player_mp::fields(player).m_playerNetServerHandler.set(Some(Rc::clone(&handler)));
        handler
    }

    fn player(&self) -> Rc<Entity> { self.m_playerEntity.get() }

    fn username(&self) -> String { JString::display(entity_player::fields(&self.player()).m_username.get().as_ref()) }

    fn dimension(&self) -> i32 { entity_player::fields(&self.player()).m_dimension.get() }

    fn craftingInventory(&self) -> Rc<Container>
    {
        entity_player::fields(&self.player()).m_craftingInventory.get().orNpe()
    }

    pub fn handlePackets(&self)
    {
        self.m_moved.set(false);
        self.m_netManager.processReadPackets();
        if self.m_ticks.get() - self.m_lastKeepAliveTime.get() > 20
        {
            self.sendPacket(Box::new(Packet0KeepAlive::default()));
        }
    }

    pub fn kickPlayer(&self, reason: &str)
    {
        let player: Rc<Entity> = self.player();
        entity_player_mp::onDisconnect(&player);
        self.sendPacket(Box::new(Packet255KickDisconnect::new(JString::from(reason))));
        self.m_netManager.close();
        self.m_mcServer.configManager().sendPacketToAllPlayers(Box::new(Packet3Chat::new(JString::from(
            format!("\u{a7}e{} left the game.", self.username()).as_str(),
        ))));
        self.m_mcServer.configManager().playerLoggedOut(&player);
        self.m_connectionClosed.set(true);
    }

    pub fn teleportTo(&self, x: f64, y: f64, z: f64, yaw: f32, pitch: f32)
    {
        self.m_hasMoved.set(false);
        self.m_lastPosX.set(x);
        self.m_lastPosY.set(y);
        self.m_lastPosZ.set(z);
        let player: Rc<Entity> = self.player();
        player.setPositionAndRotation(x, y, z, yaw, pitch);
        entity_player_mp::netServerHandler(&player).sendPacket(Box::new(Packet13PlayerLookMove::new(
            x,
            y + 1.62f32 as f64,
            y,
            z,
            yaw,
            pitch,
            false,
        )));
    }

    pub fn sendPacket(&self, packet: Box<dyn Packet>)
    {
        self.m_netManager.addToSendQueue(packet);
        self.m_lastKeepAliveTime.set(self.m_ticks.get());
    }

    fn isAllowed(text: &JString) -> bool
    {
        let allowed: Vec<u16> = ChatAllowedCharacters::allowedCharacters().encode_utf16().collect();
        (0..text.length()).all(|i| allowed.contains(&text.charAt(i)))
    }

    fn handleSlashCommand(&self, command: &JString)
    {
        let mut command: JString = command.clone();
        if command.toLowerCase().startsWith("/me ")
        {
            command = JString::from(
                format!("* {} {}", self.username(), command.substringFrom(command.indexOf(b' ' as u16)).trim())
                    .as_str(),
            );
            logger().info(&command.to_string());
            self.m_mcServer.configManager().sendPacketToAllPlayers(Box::new(Packet3Chat::new(command)));
        }
        else if command.toLowerCase().startsWith("/kill")
        {
            self.player().attackEntityFrom(None, 1000);
        }
        else if command.toLowerCase().startsWith("/tell ")
        {
            let astring: Vec<JString> = game::port::text::splitUnits(command.chars(), b' ' as u16)
                .into_iter()
                .map(JString::fromUtf16)
                .collect();
            if astring.len() >= 3
            {
                command = command.substringFrom(command.indexOf(b' ' as u16)).trim();
                command = command.substringFrom(command.indexOf(b' ' as u16)).trim();
                command = JString::from(format!("\u{a7}7{} whispers {command}", self.username()).as_str());
                logger().info(&format!("{command} to {}", astring[1]));
                if !self.m_mcServer.configManager().sendPacketToPlayer(&astring[1], Box::new(Packet3Chat::new(command)))
                {
                    self.sendPacket(Box::new(Packet3Chat::new(JString::from(
                        "\u{a7}cThere's no player by that name online.",
                    ))));
                }
            }
        }
        else if self.m_mcServer.configManager().isOp(entity_player::fields(&self.player()).m_username.get().as_ref())
        {
            let s: JString = command.substringFrom(1);
            logger().info(&format!("{} issued server command: {s}", self.username()));
            let listener: Rc<dyn ICommandListener> = self.m_self.upgrade().orNpe();
            self.m_mcServer.addCommand(&s.to_string(), listener);
        }
        else
        {
            let s1: JString = command.substringFrom(1);
            logger().info(&format!("{} tried command: {s1}", self.username()));
        }
    }

    pub fn getNumChunkDataPackets(&self) -> i32 { self.m_netManager.getNumChunkDataPackets() }
}

impl NetHandler for NetServerHandler
{
    fn isServerHandler(&self) -> bool { true }

    fn handlePlayerInput(&self, packet: &Packet27Position)
    {
        entity_player_mp::setMovementType(
            &self.player(),
            packet.getSidewaysSpeed(),
            packet.getForwardSpeed(),
            packet.getJumping(),
            packet.getSneaking(),
            packet.getPitch(),
            packet.getYaw(),
        );
    }

    fn handleFlying(&self, packet: &Packet10Flying)
    {
        let worldserver: Rc<World> = self.m_mcServer.getWorldManager(self.dimension());
        let player: Rc<Entity> = self.player();
        self.m_moved.set(true);
        if !self.m_hasMoved.get()
        {
            let d0: f64 = packet.m_yPosition - self.m_lastPosY.get();
            if packet.m_xPosition == self.m_lastPosX.get()
                && d0 * d0 < 0.01
                && packet.m_zPosition == self.m_lastPosZ.get()
            {
                self.m_hasMoved.set(true);
            }
        }
        if !self.m_hasMoved.get()
        {
            return;
        }
        if let Some(riding) = player.m_ridingEntity.get()
        {
            let mut f4: f32 = player.m_rotationYaw.get();
            let mut f: f32 = player.m_rotationPitch.get();
            riding.updateRiderPosition();
            let d9: f64 = player.m_posX.get();
            let d10: f64 = player.m_posY.get();
            let d11: f64 = player.m_posZ.get();
            let mut d12: f64 = 0.0;
            let mut d14: f64 = 0.0;
            if packet.m_rotating
            {
                f4 = packet.m_yaw;
                f = packet.m_pitch;
            }
            if packet.m_moving && packet.m_yPosition == -999.0 && packet.m_stance == -999.0
            {
                d12 = packet.m_xPosition;
                d14 = packet.m_zPosition;
            }
            player.m_onGround.set(packet.m_onGround);
            entity_player_mp::onUpdateEntity(&player, true);
            player.moveEntity(d12, 0.0, d14);
            player.setPositionAndRotation(d9, d10, d11, f4, f);
            player.m_motionX.set(d12);
            player.m_motionZ.set(d14);
            if let Some(riding) = player.m_ridingEntity.get()
            {
                world_server::tickVehicle(&worldserver, &riding, true);
            }
            if let Some(riding) = player.m_ridingEntity.get()
            {
                riding.updateRiderPosition();
            }
            self.m_mcServer.configManager().r#move(&player);
            self.m_lastPosX.set(player.m_posX.get());
            self.m_lastPosY.set(player.m_posY.get());
            self.m_lastPosZ.set(player.m_posZ.get());
            worldserver.updateEntity(&player);
            return;
        }
        if player.isPlayerSleeping()
        {
            entity_player_mp::onUpdateEntity(&player, true);
            player.setPositionAndRotation(
                self.m_lastPosX.get(),
                self.m_lastPosY.get(),
                self.m_lastPosZ.get(),
                player.m_rotationYaw.get(),
                player.m_rotationPitch.get(),
            );
            worldserver.updateEntity(&player);
            return;
        }
        let d8: f64 = player.m_posY.get();
        self.m_lastPosX.set(player.m_posX.get());
        self.m_lastPosY.set(player.m_posY.get());
        self.m_lastPosZ.set(player.m_posZ.get());
        let mut d1: f64 = player.m_posX.get();
        let mut d2: f64 = player.m_posY.get();
        let mut d3: f64 = player.m_posZ.get();
        let mut f1: f32 = player.m_rotationYaw.get();
        let mut f2: f32 = player.m_rotationPitch.get();
        let mut moving: bool = packet.m_moving;
        if moving && packet.m_yPosition == -999.0 && packet.m_stance == -999.0
        {
            moving = false;
        }
        if moving
        {
            d1 = packet.m_xPosition;
            d2 = packet.m_yPosition;
            d3 = packet.m_zPosition;
            let d4: f64 = packet.m_stance - packet.m_yPosition;
            if !player.isPlayerSleeping() && (d4 > 1.65 || d4 < 0.1)
            {
                self.kickPlayer("Illegal stance");
                logger().warning(&format!("{} had an illegal stance: {}", self.username(), doubleToString(d4)));
                return;
            }
            if packet.m_xPosition.abs() > 3.2E7 || packet.m_zPosition.abs() > 3.2E7
            {
                self.kickPlayer("Illegal position");
                return;
            }
        }
        if packet.m_rotating
        {
            f1 = packet.m_yaw;
            f2 = packet.m_pitch;
        }
        entity_player_mp::onUpdateEntity(&player, true);
        player.m_ySize.set(0.0);
        player.setPositionAndRotation(self.m_lastPosX.get(), self.m_lastPosY.get(), self.m_lastPosZ.get(), f1, f2);
        if !self.m_hasMoved.get()
        {
            return;
        }
        let mut d13: f64 = d1 - player.m_posX.get();
        let mut d5: f64 = d2 - player.m_posY.get();
        let mut d6: f64 = d3 - player.m_posZ.get();
        let mut d7: f64 = d13 * d13 + d5 * d5 + d6 * d6;
        if d7 > 100.0
        {
            logger().warning(&format!("{} moved too quickly!", self.username()));
            self.kickPlayer("You moved too quickly :( (Hacking?)");
            return;
        }
        let f3: f32 = 0.0625;
        let flag: bool = worldserver
            .getCollidingBoundingBoxes(&player, &player.m_boundingBox.copy().contract(f3 as f64, f3 as f64, f3 as f64))
            .size()
            == 0;
        player.moveEntity(d13, d5, d6);
        d13 = d1 - player.m_posX.get();
        d5 = d2 - player.m_posY.get();
        if d5 > -0.5 || d5 < 0.5
        {
            d5 = 0.0;
        }
        d6 = d3 - player.m_posZ.get();
        d7 = d13 * d13 + d5 * d5 + d6 * d6;
        let mut flag1: bool = false;
        if d7 > 0.0625 && !player.isPlayerSleeping()
        {
            flag1 = true;
            logger().warning(&format!("{} moved wrongly!", self.username()));
            jvm::system::println(&format!(
                "Got position {}, {}, {}",
                doubleToString(d1),
                doubleToString(d2),
                doubleToString(d3)
            ));
            jvm::system::println(&format!(
                "Expected {}, {}, {}",
                doubleToString(player.m_posX.get()),
                doubleToString(player.m_posY.get()),
                doubleToString(player.m_posZ.get())
            ));
        }
        player.setPositionAndRotation(d1, d2, d3, f1, f2);
        let flag2: bool = worldserver
            .getCollidingBoundingBoxes(&player, &player.m_boundingBox.copy().contract(f3 as f64, f3 as f64, f3 as f64))
            .size()
            == 0;
        if flag && (flag1 || !flag2) && !player.isPlayerSleeping()
        {
            self.teleportTo(self.m_lastPosX.get(), self.m_lastPosY.get(), self.m_lastPosZ.get(), f1, f2);
            return;
        }
        let axisalignedbb: Rc<AxisAlignedBB> =
            player.m_boundingBox.copy().expand(f3 as f64, f3 as f64, f3 as f64).addCoord(0.0, -0.55, 0.0);
        if self.m_mcServer.m_allowFlight.get() || worldserver.containsNonAir(&axisalignedbb)
        {
            self.m_playerInAirTime.set(0);
        }
        else if d5 >= -0.03125
        {
            self.m_playerInAirTime.set(self.m_playerInAirTime.get() + 1);
            if self.m_playerInAirTime.get() > 80
            {
                logger().warning(&format!("{} was kicked for floating too long!", self.username()));
                self.kickPlayer("Flying is not enabled on this server");
                return;
            }
        }
        player.m_onGround.set(packet.m_onGround);
        self.m_mcServer.configManager().r#move(&player);
        entity_player_mp::handleFalling(&player, player.m_posY.get() - d8, packet.m_onGround);
    }

    fn handleBlockDig(&self, packet: &Packet14BlockDig)
    {
        let worldserver: Rc<World> = self.m_mcServer.getWorldManager(self.dimension());
        let player: Rc<Entity> = self.player();
        if packet.m_status == 4
        {
            player.dropCurrentItem();
            return;
        }
        let flag: bool = worldserver.worldProvider().m_worldType.get() != 0
            || self.m_mcServer.configManager().isOp(entity_player::fields(&player).m_username.get().as_ref());
        world_server::fields(&worldserver).m_bypassSpawnProtection.set(flag);
        let mut flag1: bool = false;
        if packet.m_status == 0
        {
            flag1 = true;
        }
        if packet.m_status == 2
        {
            flag1 = true;
        }
        let i: i32 = packet.m_xPosition;
        let j: i32 = packet.m_yPosition;
        let k: i32 = packet.m_zPosition;
        if flag1
        {
            let d0: f64 = player.m_posX.get() - (i as f64 + 0.5);
            let d1: f64 = player.m_posY.get() - (j as f64 + 0.5);
            let d3: f64 = player.m_posZ.get() - (k as f64 + 0.5);
            let d5: f64 = d0 * d0 + d1 * d1 + d3 * d3;
            if d5 > 36.0
            {
                return;
            }
        }
        let chunkcoordinates: ChunkCoordinates = worldserver.getSpawnPoint();
        let l: i32 = math_helper::abs((i - chunkcoordinates.m_x) as f32) as i32;
        let mut i1: i32 = math_helper::abs((k - chunkcoordinates.m_z) as f32) as i32;
        if l > i1
        {
            i1 = l;
        }
        let handler: Rc<NetServerHandler> = entity_player_mp::netServerHandler(&player);
        if packet.m_status == 0
        {
            if i1 <= 16 && !flag
            {
                handler.sendPacket(Box::new(Packet53BlockChange::new(i, j, k, &worldserver)));
            }
            else
            {
                entity_player_mp::fields(&player).m_itemInWorldManager.get().orNpe().startMiningBlock(
                    i,
                    j,
                    k,
                    packet.m_face,
                );
            }
        }
        else if packet.m_status == 2
        {
            entity_player_mp::fields(&player).m_itemInWorldManager.get().orNpe().finishMiningBlock(i, j, k);
            if worldserver.getBlockId(i, j, k) != 0
            {
                handler.sendPacket(Box::new(Packet53BlockChange::new(i, j, k, &worldserver)));
            }
        }
        else if packet.m_status == 3
        {
            let d2: f64 = player.m_posX.get() - (i as f64 + 0.5);
            let d4: f64 = player.m_posY.get() - (j as f64 + 0.5);
            let d6: f64 = player.m_posZ.get() - (k as f64 + 0.5);
            let d7: f64 = d2 * d2 + d4 * d4 + d6 * d6;
            if d7 < 256.0
            {
                handler.sendPacket(Box::new(Packet53BlockChange::new(i, j, k, &worldserver)));
            }
        }
        world_server::fields(&worldserver).m_bypassSpawnProtection.set(false);
    }

    fn handlePlace(&self, packet: &Packet15Place)
    {
        let worldserver: Rc<World> = self.m_mcServer.getWorldManager(self.dimension());
        let player: Rc<Entity> = self.player();
        let inventory: Rc<InventoryPlayer> = entity_player::inventory(&player);
        let mut itemstack: Option<Rc<ItemStack>> = inventory.getCurrentItem();
        let flag: bool = worldserver.worldProvider().m_worldType.get() != 0
            || self.m_mcServer.configManager().isOp(entity_player::fields(&player).m_username.get().as_ref());
        world_server::fields(&worldserver).m_bypassSpawnProtection.set(flag);
        let handler: Rc<NetServerHandler> = entity_player_mp::netServerHandler(&player);
        if packet.m_direction == 255
        {
            let Some(itemstack) = &itemstack
            else
            {
                return;
            };
            entity_player_mp::fields(&player).m_itemInWorldManager.get().orNpe().useItem(
                &player,
                &worldserver,
                itemstack,
            );
        }
        else
        {
            let mut i: i32 = packet.m_xPosition;
            let mut j: i32 = packet.m_yPosition;
            let mut k: i32 = packet.m_zPosition;
            let l: i32 = packet.m_direction;
            let chunkcoordinates: ChunkCoordinates = worldserver.getSpawnPoint();
            let i1: i32 = math_helper::abs((i - chunkcoordinates.m_x) as f32) as i32;
            let mut j1: i32 = math_helper::abs((k - chunkcoordinates.m_z) as f32) as i32;
            if i1 > j1
            {
                j1 = i1;
            }
            if self.m_hasMoved.get()
                && player.getDistanceSq(i as f64 + 0.5, j as f64 + 0.5, k as f64 + 0.5) < 64.0
                && (j1 > 16 || flag)
            {
                entity_player_mp::fields(&player).m_itemInWorldManager.get().orNpe().activeBlockOrUseItem(
                    &player,
                    &worldserver,
                    itemstack.as_ref(),
                    i,
                    j,
                    k,
                    l,
                );
            }
            handler.sendPacket(Box::new(Packet53BlockChange::new(i, j, k, &worldserver)));
            if l == 0
            {
                j -= 1;
            }
            if l == 1
            {
                j += 1;
            }
            if l == 2
            {
                k -= 1;
            }
            if l == 3
            {
                k += 1;
            }
            if l == 4
            {
                i -= 1;
            }
            if l == 5
            {
                i += 1;
            }
            handler.sendPacket(Box::new(Packet53BlockChange::new(i, j, k, &worldserver)));
        }
        itemstack = inventory.getCurrentItem();
        let currentItem: usize = inventory.m_currentItem.get() as usize;
        if let Some(itemstack) = &itemstack
            && itemstack.m_stackSize.get() == 0
        {
            inventory.m_mainInventory.borrow_mut()[currentItem] = None;
        }
        let ext: &entity_player_mp::PlayerMPExt = entity_player_mp::fields(&player);
        ext.m_isChangingQuantityOnly.set(true);
        let copied: Option<Rc<ItemStack>> =
            ItemStack::copyItemStack(inventory.m_mainInventory.borrow()[currentItem].as_ref());
        inventory.m_mainInventory.borrow_mut()[currentItem] = copied;
        let playerInventory: Rc<dyn IInventory> = Rc::clone(&inventory) as Rc<dyn IInventory>;
        let slot: Option<Rc<Slot>> =
            self.craftingInventory().getSlotFor(&playerInventory, inventory.m_currentItem.get());
        self.craftingInventory().updateCraftingResults();
        ext.m_isChangingQuantityOnly.set(false);
        let packetStack: Option<Rc<ItemStack>> = PacketItemStack::toItemStackOption(packet.m_itemStack);
        if !ItemStack::areItemStacksEqual(inventory.getCurrentItem().as_ref(), packetStack.as_ref())
        {
            self.sendPacket(Box::new(Packet103SetSlot::new(
                self.craftingInventory().m_windowId.get(),
                slot.orNpe().m_slotNumber.get(),
                inventory.getCurrentItem().as_ref(),
            )));
        }
        world_server::fields(&worldserver).m_bypassSpawnProtection.set(false);
    }

    fn handleErrorMessage(&self, reason: &JString, _args: &[JString])
    {
        let player: Rc<Entity> = self.player();
        logger().info(&format!("{} lost connection: {reason}", self.username()));
        self.m_mcServer.configManager().sendPacketToAllPlayers(Box::new(Packet3Chat::new(JString::from(
            format!("\u{a7}e{} left the game.", self.username()).as_str(),
        ))));
        self.m_mcServer.configManager().playerLoggedOut(&player);
        self.m_connectionClosed.set(true);
    }

    fn registerPacket(&self, packet: &dyn Packet)
    {
        logger().warning(&format!(
            "class net.minecraft.src.NetServerHandler wasn't prepared to deal with a class net.minecraft.src.{}",
            game::network::packet::className(packet)
        ));
        self.kickPlayer("Protocol error, unexpected packet");
    }

    fn handleBlockItemSwitch(&self, packet: &Packet16BlockItemSwitch)
    {
        if packet.m_id < 0 || packet.m_id > InventoryPlayer::getHotbarSize()
        {
            logger().warning(&format!("{} tried to set an invalid carried item", self.username()));
            return;
        }
        entity_player::inventory(&self.player()).m_currentItem.set(packet.m_id);
    }

    fn handleChat(&self, packet: &Packet3Chat)
    {
        let message: JString = packet.m_message.clone().orNpe();
        if message.length() > 100
        {
            self.kickPlayer("Chat message too long");
            return;
        }
        let trim: JString = message.trim();
        if !NetServerHandler::isAllowed(&trim)
        {
            self.kickPlayer("Illegal characters in chat");
            return;
        }
        if trim.startsWith("/")
        {
            self.handleSlashCommand(&trim);
        }
        else
        {
            let string: String = format!("<{}> {trim}", self.username());
            logger().info(&string);
            self.m_mcServer
                .configManager()
                .sendPacketToAllPlayers(Box::new(Packet3Chat::new(JString::from(string.as_str()))));
        }
    }

    fn handleArmAnimation(&self, packet: &Packet18Animation)
    {
        if packet.m_animate == 1
        {
            self.player().swingItem();
        }
    }

    fn handlePlayerMovementAction(&self, packet: &Packet19EntityAction)
    {
        if packet.m_state == 1
        {
            self.player().setSneaking(true);
        }
        else if packet.m_state == 2
        {
            self.player().setSneaking(false);
        }
        else if packet.m_state == 3
        {
            self.player().wakeUpPlayer(false, true, true);
            self.m_hasMoved.set(false);
        }
    }

    fn handleKickDisconnect(&self, _packet: &Packet255KickDisconnect)
    {
        self.m_netManager.networkShutdown("disconnect.quitting", Vec::new());
    }

    fn handleUseEntity(&self, packet: &Packet7UseEntity)
    {
        let worldserver: Rc<World> = self.m_mcServer.getWorldManager(self.dimension());
        let player: Rc<Entity> = self.player();
        let entity: Option<Rc<Entity>> = world_server::getEntity(&worldserver, packet.m_targetEntity);
        if let Some(entity) = entity
            && entity_living::canEntityBeSeen(&player, &entity)
            && player.getDistanceSqToEntity(&entity) < 36.0
        {
            if packet.m_isLeftClick == 0
            {
                entity_player::useCurrentItemOnEntity(&player, &entity);
            }
            else if packet.m_isLeftClick == 1
            {
                entity_player::attackTargetEntityWithCurrentItem(&player, &entity);
            }
        }
    }

    fn handlePlayerRespawn(&self, _packet: &Packet9Respawn)
    {
        let player: Rc<Entity> = self.player();
        if entity_living::fields(&player).m_health.get() > 0
        {
            return;
        }
        self.m_playerEntity.set(self.m_mcServer.configManager().recreatePlayerEntity(&player, 0));
    }

    fn handleCloseInventoryMenu(&self, _packet: &Packet101CloseWindow)
    {
        entity_player_mp::closeCraftingGui(&self.player());
    }

    fn handleInventoryMenuClickSlot(&self, packet: &Packet102WindowClick)
    {
        let player: Rc<Entity> = self.player();
        if self.craftingInventory().m_windowId.get() == packet.m_window_Id
            && self.craftingInventory().getCanCraft(&player)
        {
            let itemstack: Option<Rc<ItemStack>> = self.craftingInventory().onClickSlot(
                packet.m_inventorySlot,
                packet.m_mouseClick,
                packet.m_quickMove,
                &player,
            );
            let packetStack: Option<Rc<ItemStack>> = PacketItemStack::toItemStackOption(packet.m_itemStack);
            if ItemStack::areItemStacksEqual(packetStack.as_ref(), itemstack.as_ref())
            {
                entity_player_mp::netServerHandler(&player).sendPacket(Box::new(Packet106Transaction::new(
                    packet.m_window_Id,
                    packet.m_action,
                    true,
                )));
                let ext: &entity_player_mp::PlayerMPExt = entity_player_mp::fields(&player);
                ext.m_isChangingQuantityOnly.set(true);
                self.craftingInventory().updateCraftingResults();
                entity_player_mp::updateHeldItem(&player);
                ext.m_isChangingQuantityOnly.set(false);
            }
            else
            {
                self.m_transactions.borrow_mut().insert(self.craftingInventory().m_windowId.get(), packet.m_action);
                entity_player_mp::netServerHandler(&player).sendPacket(Box::new(Packet106Transaction::new(
                    packet.m_window_Id,
                    packet.m_action,
                    false,
                )));
                self.craftingInventory().setCanCraft(&player, false);
                let mut arraylist: Vec<Option<Rc<ItemStack>>> = Vec::new();
                let mut i: i32 = 0;
                while i < self.craftingInventory().m_slots.size()
                {
                    arraylist.push(self.craftingInventory().m_slots.get(i).getStack());
                    i += 1;
                }
                entity_player_mp::updateCraftingInventory(&player, &self.craftingInventory(), &arraylist);
            }
        }
    }

    fn handleInventoryMenuConfirm(&self, packet: &Packet106Transaction)
    {
        let player: Rc<Entity> = self.player();
        let oshort: Option<i16> = self.m_transactions.borrow().get(&self.craftingInventory().m_windowId.get()).copied();
        if let Some(oshort) = oshort
            && packet.m_interactionId == oshort
            && self.craftingInventory().m_windowId.get() == packet.m_windowId
            && !self.craftingInventory().getCanCraft(&player)
        {
            self.craftingInventory().setCanCraft(&player, true);
        }
    }

    fn handleSignUpdate(&self, packet: &Packet130UpdateSign)
    {
        let worldserver: Rc<World> = self.m_mcServer.getWorldManager(self.dimension());
        if !worldserver.blockExists(packet.m_xPosition, packet.m_yPosition, packet.m_zPosition)
        {
            return;
        }
        let tileentity: Option<Rc<TileEntity>> =
            worldserver.getBlockTileEntity(packet.m_xPosition, packet.m_yPosition, packet.m_zPosition);
        if let Some(tileentity) = &tileentity
            && tileentity.instanceOf("TileEntitySign")
            && !tile_entity_sign::getIsEditAble(tileentity)
        {
            self.m_mcServer.logWarning(&format!("Player {} just tried to change non-editable sign", self.username()));
            return;
        }
        let mut signLines: Vec<Option<JString>> = packet.m_signLines.clone().orNpe();
        for k in 0..4
        {
            let mut flag: bool = true;
            let line: &JString = signLines[k].as_ref().orNpe();
            if line.length() > 15
            {
                flag = false;
            }
            else if !NetServerHandler::isAllowed(line)
            {
                flag = false;
            }
            if !flag
            {
                signLines[k] = Some(JString::from("!?"));
            }
        }
        if let Some(tileentity) = tileentity
            && tileentity.instanceOf("TileEntitySign")
        {
            let l: i32 = packet.m_xPosition;
            let i1: i32 = packet.m_yPosition;
            let j1: i32 = packet.m_zPosition;
            let sign: &tile_entity_sign::SignExt = tile_entity_sign::fields(&tileentity);
            for j in 0..4
            {
                let line: Option<JString> = signLines[j].clone();
                sign.m_signText.update(|text| text[j] = line);
            }
            tile_entity_sign::setEditable(&tileentity, false);
            tileentity.onInventoryChanged();
            worldserver.markBlockNeedsUpdate(l, i1, j1);
        }
    }
}

impl ICommandListener for NetServerHandler
{
    fn log(&self, message: &str)
    {
        self.sendPacket(Box::new(Packet3Chat::new(JString::from(format!("\u{a7}7{message}").as_str()))));
    }

    fn getUsername(&self) -> String { self.username() }
}
