use crate::network::packet::Packet;
use crate::network::packets::*;
use jvm::string::JString;

pub trait NetHandler
{
    fn isServerHandler(&self) -> bool;

    fn handleMapChunk(&self, _packet: &Packet51MapChunk) {}

    fn registerPacket(&self, _packet: &dyn Packet) {}

    fn handleErrorMessage(&self, _reason: &JString, _args: &[JString]) {}

    fn handleKickDisconnect(&self, packet: &Packet255KickDisconnect) { self.registerPacket(packet); }

    fn handleLogin(&self, packet: &Packet1Login) { self.registerPacket(packet); }

    fn handleFlying(&self, packet: &Packet10Flying) { self.registerPacket(packet); }

    fn handleMultiBlockChange(&self, packet: &Packet52MultiBlockChange) { self.registerPacket(packet); }

    fn handleBlockDig(&self, packet: &Packet14BlockDig) { self.registerPacket(packet); }

    fn handleBlockChange(&self, packet: &Packet53BlockChange) { self.registerPacket(packet); }

    fn handlePreChunk(&self, packet: &Packet50PreChunk) { self.registerPacket(packet); }

    fn handleNamedEntitySpawn(&self, packet: &Packet20NamedEntitySpawn) { self.registerPacket(packet); }

    fn handleEntity(&self, packet: &Packet30Entity) { self.registerPacket(packet); }

    fn handleEntityTeleport(&self, packet: &Packet34EntityTeleport) { self.registerPacket(packet); }

    fn handlePlace(&self, packet: &Packet15Place) { self.registerPacket(packet); }

    fn handleBlockItemSwitch(&self, packet: &Packet16BlockItemSwitch) { self.registerPacket(packet); }

    fn handleDestroyEntity(&self, packet: &Packet29DestroyEntity) { self.registerPacket(packet); }

    fn handlePickupSpawn(&self, packet: &Packet21PickupSpawn) { self.registerPacket(packet); }

    fn handleCollect(&self, packet: &Packet22Collect) { self.registerPacket(packet); }

    fn handleChat(&self, packet: &Packet3Chat) { self.registerPacket(packet); }

    fn handleVehicleSpawn(&self, packet: &Packet23VehicleSpawn) { self.registerPacket(packet); }

    fn handleArmAnimation(&self, packet: &Packet18Animation) { self.registerPacket(packet); }

    fn handlePlayerMovementAction(&self, packet: &Packet19EntityAction) { self.registerPacket(packet); }

    fn handleHandshake(&self, packet: &Packet2Handshake) { self.registerPacket(packet); }

    fn handleMobSpawn(&self, packet: &Packet24MobSpawn) { self.registerPacket(packet); }

    fn handleUpdateTime(&self, packet: &Packet4UpdateTime) { self.registerPacket(packet); }

    fn handleSpawnPosition(&self, packet: &Packet6SpawnPosition) { self.registerPacket(packet); }

    fn handleEntityVelocity(&self, packet: &Packet28EntityVelocity) { self.registerPacket(packet); }

    fn handleEntityData(&self, packet: &Packet40EntityMetadata) { self.registerPacket(packet); }

    fn handleRideEntity(&self, packet: &Packet39AttachEntity) { self.registerPacket(packet); }

    fn handleUseEntity(&self, packet: &Packet7UseEntity) { self.registerPacket(packet); }

    fn handleEntityEvent(&self, packet: &Packet38EntityStatus) { self.registerPacket(packet); }

    fn handleHealth(&self, packet: &Packet8UpdateHealth) { self.registerPacket(packet); }

    fn handlePlayerRespawn(&self, packet: &Packet9Respawn) { self.registerPacket(packet); }

    fn handleExplosion(&self, packet: &Packet60Explosion) { self.registerPacket(packet); }

    fn handleOpenInventoryMenu(&self, packet: &Packet100OpenWindow) { self.registerPacket(packet); }

    fn handleCloseInventoryMenu(&self, packet: &Packet101CloseWindow) { self.registerPacket(packet); }

    fn handleInventoryMenuClickSlot(&self, packet: &Packet102WindowClick) { self.registerPacket(packet); }

    fn handleInventoryMenuSlotContent(&self, packet: &Packet103SetSlot) { self.registerPacket(packet); }

    fn handleInventoryMenuContent(&self, packet: &Packet104WindowItems) { self.registerPacket(packet); }

    fn handleSignUpdate(&self, packet: &Packet130UpdateSign) { self.registerPacket(packet); }

    fn handleInventoryMenuData(&self, packet: &Packet105UpdateProgressbar) { self.registerPacket(packet); }

    fn handlePlayerInventory(&self, packet: &Packet5PlayerInventory) { self.registerPacket(packet); }

    fn handleInventoryMenuConfirm(&self, packet: &Packet106Transaction) { self.registerPacket(packet); }

    fn handleAddPainting(&self, packet: &Packet25EntityPainting) { self.registerPacket(packet); }

    fn handleNotePlay(&self, packet: &Packet54PlayNoteBlock) { self.registerPacket(packet); }

    fn handleStat(&self, packet: &Packet200Statistic) { self.registerPacket(packet); }

    fn handlePlayerSleep(&self, packet: &Packet17Sleep) { self.registerPacket(packet); }

    fn handlePlayerInput(&self, packet: &Packet27Position) { self.registerPacket(packet); }

    fn handleGameEvent(&self, packet: &Packet70Bed) { self.registerPacket(packet); }

    fn handleWeather(&self, packet: &Packet71Weather) { self.registerPacket(packet); }

    fn handleMapData(&self, packet: &Packet131MapData) { self.registerPacket(packet); }

    fn handleWorldEvent(&self, packet: &Packet61DoorChange) { self.registerPacket(packet); }
}
