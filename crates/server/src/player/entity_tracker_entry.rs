use crate::player::entity_player_mp;
use game::block::Block;
use game::entity::{DataWatcher, Entity, entity_arrow, entity_falling_sand, entity_fireball, entity_minecart};
use game::item::ItemStack;
use game::math::math_helper;
use game::network::packet::Packet;
use game::network::packets::{
    Packet5PlayerInventory, Packet17Sleep, Packet20NamedEntitySpawn, Packet21PickupSpawn, Packet23VehicleSpawn,
    Packet24MobSpawn, Packet25EntityPainting, Packet28EntityVelocity, Packet29DestroyEntity, Packet31RelEntityMove,
    Packet32EntityLook, Packet33RelEntityMoveLook, Packet34EntityTeleport, Packet40EntityMetadata,
};
use jvm::field::Field;
use jvm::hash_map::JHashSet;
use jvm::object::JavaObject;
use jvm::throwable::OrNpe;
use std::cell::Cell;
use std::rc::Rc;

pub struct EntityTrackerEntry
{
    pub m_trackedEntity: Rc<Entity>,
    pub m_trackingDistanceThreshold: i32,
    pub m_updateInterval: i32,
    pub m_encodedPosX: Cell<i32>,
    pub m_encodedPosY: Cell<i32>,
    pub m_encodedPosZ: Cell<i32>,
    pub m_encodedRotationYaw: Cell<i32>,
    pub m_encodedRotationPitch: Cell<i32>,
    pub m_lastTrackedEntityMotionX: Cell<f64>,
    pub m_lastTrackedEntityMotionY: Cell<f64>,
    pub m_lastTrackedEntityMotionZ: Cell<f64>,
    pub m_updateCounter: Cell<i32>,
    m_lastTrackedEntityPosX: Cell<f64>,
    m_lastTrackedEntityPosY: Cell<f64>,
    m_lastTrackedEntityPosZ: Cell<f64>,
    m_firstUpdateDone: Cell<bool>,
    m_shouldSendMotionUpdates: bool,
    m_ticksWithoutVehicle: Cell<i32>,
    pub m_playerEntitiesUpdated: Cell<bool>,
    pub m_trackedPlayers: Field<JHashSet<Rc<Entity>>>,
}

impl JavaObject for EntityTrackerEntry
{
    fn javaEquals(&self, other: &Self) -> bool
    {
        other.m_trackedEntity.m_entityId.get() == self.m_trackedEntity.m_entityId.get()
    }

    fn javaHash(&self) -> i32 { self.m_trackedEntity.m_entityId.get() }
}

impl EntityTrackerEntry
{
    pub fn new(
        entity: &Rc<Entity>,
        trackingRange: i32,
        updateInterval: i32,
        syncVelocity: bool,
    ) -> Rc<EntityTrackerEntry>
    {
        Rc::new(EntityTrackerEntry {
            m_updateCounter: Cell::new(0),
            m_firstUpdateDone: Cell::new(false),
            m_ticksWithoutVehicle: Cell::new(0),
            m_playerEntitiesUpdated: Cell::new(false),
            m_trackedPlayers: Field::new(JHashSet::new()),
            m_trackedEntity: Rc::clone(entity),
            m_trackingDistanceThreshold: trackingRange,
            m_updateInterval: updateInterval,
            m_shouldSendMotionUpdates: syncVelocity,
            m_encodedPosX: Cell::new(math_helper::floor_double(entity.m_posX.get() * 32.0)),
            m_encodedPosY: Cell::new(math_helper::floor_double(entity.m_posY.get() * 32.0)),
            m_encodedPosZ: Cell::new(math_helper::floor_double(entity.m_posZ.get() * 32.0)),
            m_encodedRotationYaw: Cell::new(math_helper::floor_float(entity.m_rotationYaw.get() * 256.0 / 360.0)),
            m_encodedRotationPitch: Cell::new(math_helper::floor_float(entity.m_rotationPitch.get() * 256.0 / 360.0)),
            m_lastTrackedEntityMotionX: Cell::new(0.0),
            m_lastTrackedEntityMotionY: Cell::new(0.0),
            m_lastTrackedEntityMotionZ: Cell::new(0.0),
            m_lastTrackedEntityPosX: Cell::new(0.0),
            m_lastTrackedEntityPosY: Cell::new(0.0),
            m_lastTrackedEntityPosZ: Cell::new(0.0),
        })
    }

    pub fn updatePlayerList(&self, players: &[Rc<Entity>])
    {
        let tracked: &Rc<Entity> = &self.m_trackedEntity;
        self.m_playerEntitiesUpdated.set(false);
        if !self.m_firstUpdateDone.get()
            || tracked.getDistanceSq(
                self.m_lastTrackedEntityPosX.get(),
                self.m_lastTrackedEntityPosY.get(),
                self.m_lastTrackedEntityPosZ.get(),
            ) > 16.0
        {
            self.m_lastTrackedEntityPosX.set(tracked.m_posX.get());
            self.m_lastTrackedEntityPosY.set(tracked.m_posY.get());
            self.m_lastTrackedEntityPosZ.set(tracked.m_posZ.get());
            self.m_firstUpdateDone.set(true);
            self.m_playerEntitiesUpdated.set(true);
            self.updatePlayerEntities(players);
        }
        self.m_ticksWithoutVehicle.set(self.m_ticksWithoutVehicle.get() + 1);
        self.m_updateCounter.set(self.m_updateCounter.get() + 1);
        if self.m_updateCounter.get() % self.m_updateInterval == 0
        {
            let i: i32 = math_helper::floor_double(tracked.m_posX.get() * 32.0);
            let j: i32 = math_helper::floor_double(tracked.m_posY.get() * 32.0);
            let k: i32 = math_helper::floor_double(tracked.m_posZ.get() * 32.0);
            let l: i32 = math_helper::floor_float(tracked.m_rotationYaw.get() * 256.0 / 360.0);
            let i1: i32 = math_helper::floor_float(tracked.m_rotationPitch.get() * 256.0 / 360.0);
            let j1: i32 = i - self.m_encodedPosX.get();
            let k1: i32 = j - self.m_encodedPosY.get();
            let l1: i32 = k - self.m_encodedPosZ.get();
            let mut packet: Option<Box<dyn Packet>> = None;
            let flag: bool = i.wrapping_abs() >= 8 || j.wrapping_abs() >= 8 || k.wrapping_abs() >= 8;
            let flag1: bool = (l - self.m_encodedRotationYaw.get()).wrapping_abs() >= 8
                || (i1 - self.m_encodedRotationPitch.get()).wrapping_abs() >= 8;
            if j1 < -128
                || j1 >= 128
                || k1 < -128
                || k1 >= 128
                || l1 < -128
                || l1 >= 128
                || self.m_ticksWithoutVehicle.get() > 400
            {
                self.m_ticksWithoutVehicle.set(0);
                tracked.m_posX.set(i as f64 / 32.0);
                tracked.m_posY.set(j as f64 / 32.0);
                tracked.m_posZ.set(k as f64 / 32.0);
                packet = Some(Box::new(Packet34EntityTeleport::withPosition(
                    tracked.m_entityId.get(),
                    i,
                    j,
                    k,
                    l as i8,
                    i1 as i8,
                )));
            }
            else if flag && flag1
            {
                packet = Some(Box::new(Packet33RelEntityMoveLook::new(
                    tracked.m_entityId.get(),
                    j1 as i8,
                    k1 as i8,
                    l1 as i8,
                    l as i8,
                    i1 as i8,
                )));
            }
            else if flag
            {
                packet =
                    Some(Box::new(Packet31RelEntityMove::new(tracked.m_entityId.get(), j1 as i8, k1 as i8, l1 as i8)));
            }
            else if flag1
            {
                packet = Some(Box::new(Packet32EntityLook::new(tracked.m_entityId.get(), l as i8, i1 as i8)));
            }
            if self.m_shouldSendMotionUpdates
            {
                let d0: f64 = tracked.m_motionX.get() - self.m_lastTrackedEntityMotionX.get();
                let d1: f64 = tracked.m_motionY.get() - self.m_lastTrackedEntityMotionY.get();
                let d2: f64 = tracked.m_motionZ.get() - self.m_lastTrackedEntityMotionZ.get();
                let d3: f64 = 0.02;
                let d4: f64 = d0 * d0 + d1 * d1 + d2 * d2;
                if d4 > d3 * d3
                    || d4 > 0.0
                        && tracked.m_motionX.get() == 0.0
                        && tracked.m_motionY.get() == 0.0
                        && tracked.m_motionZ.get() == 0.0
                {
                    self.m_lastTrackedEntityMotionX.set(tracked.m_motionX.get());
                    self.m_lastTrackedEntityMotionY.set(tracked.m_motionY.get());
                    self.m_lastTrackedEntityMotionZ.set(tracked.m_motionZ.get());
                    self.sendPacketToTrackedPlayers(Box::new(Packet28EntityVelocity::new(
                        tracked.m_entityId.get(),
                        self.m_lastTrackedEntityMotionX.get(),
                        self.m_lastTrackedEntityMotionY.get(),
                        self.m_lastTrackedEntityMotionZ.get(),
                    )));
                }
            }
            if let Some(packet) = packet
            {
                self.sendPacketToTrackedPlayers(packet);
            }
            let datawatcher: &DataWatcher = tracked.getDataWatcher();
            if datawatcher.hasObjectChanged()
            {
                self.sendPacketToTrackedPlayersAndTrackedEntity(Box::new(Packet40EntityMetadata::new(
                    tracked.m_entityId.get(),
                    datawatcher,
                )));
            }
            if flag
            {
                self.m_encodedPosX.set(i);
                self.m_encodedPosY.set(j);
                self.m_encodedPosZ.set(k);
            }
            if flag1
            {
                self.m_encodedRotationYaw.set(l);
                self.m_encodedRotationPitch.set(i1);
            }
        }
        if tracked.m_beenAttacked.get()
        {
            self.sendPacketToTrackedPlayersAndTrackedEntity(Box::new(Packet28EntityVelocity::fromEntity(tracked)));
            tracked.m_beenAttacked.set(false);
        }
    }

    pub fn sendPacketToTrackedPlayers(&self, packet: Box<dyn Packet>)
    {
        for entityplayermp in self.m_trackedPlayers.with(JHashSet::items)
        {
            entity_player_mp::netServerHandler(&entityplayermp).sendPacket(packet.clonePacket());
        }
    }

    pub fn sendPacketToTrackedPlayersAndTrackedEntity(&self, packet: Box<dyn Packet>)
    {
        self.sendPacketToTrackedPlayers(packet.clonePacket());
        if self.m_trackedEntity.instanceOf("EntityPlayerMP")
        {
            entity_player_mp::netServerHandler(&self.m_trackedEntity).sendPacket(packet);
        }
    }

    pub fn sendDestroyEntityPacketToTrackedPlayers(&self)
    {
        self.sendPacketToTrackedPlayers(Box::new(Packet29DestroyEntity::new(self.m_trackedEntity.m_entityId.get())));
    }

    pub fn removeFromTrackedPlayers(&self, player: &Rc<Entity>)
    {
        if self.m_trackedPlayers.with(|set| set.contains(player))
        {
            self.m_trackedPlayers.update(|set| set.remove(player));
        }
    }

    pub fn updatePlayerEntity(&self, player: &Rc<Entity>)
    {
        if Rc::ptr_eq(player, &self.m_trackedEntity)
        {
            return;
        }
        let d0: f64 = player.m_posX.get() - (self.m_encodedPosX.get() / 32) as f64;
        let d1: f64 = player.m_posZ.get() - (self.m_encodedPosZ.get() / 32) as f64;
        let threshold: f64 = self.m_trackingDistanceThreshold as f64;
        if d0 >= -threshold && d0 <= threshold && d1 >= -threshold && d1 <= threshold
        {
            if !self.m_trackedPlayers.with(|set| set.contains(player))
            {
                self.m_trackedPlayers.update(|set| set.add(Rc::clone(player)));
                let handler: Rc<crate::network::NetServerHandler> = entity_player_mp::netServerHandler(player);
                handler.sendPacket(self.getSpawnPacket());
                let tracked: &Rc<Entity> = &self.m_trackedEntity;
                if self.m_shouldSendMotionUpdates
                {
                    handler.sendPacket(Box::new(Packet28EntityVelocity::new(
                        tracked.m_entityId.get(),
                        tracked.m_motionX.get(),
                        tracked.m_motionY.get(),
                        tracked.m_motionZ.get(),
                    )));
                }
                let aitemstack: Option<Vec<Option<Rc<ItemStack>>>> = tracked.getInventory();
                if let Some(aitemstack) = aitemstack
                {
                    for i in 0..aitemstack.len()
                    {
                        handler.sendPacket(Box::new(Packet5PlayerInventory::new(
                            tracked.m_entityId.get(),
                            i as i32,
                            aitemstack[i].as_ref(),
                        )));
                    }
                }
                if tracked.instanceOf("EntityPlayer") && tracked.isPlayerSleeping()
                {
                    handler.sendPacket(Box::new(Packet17Sleep::new(
                        tracked,
                        0,
                        math_helper::floor_double(tracked.m_posX.get()),
                        math_helper::floor_double(tracked.m_posY.get()),
                        math_helper::floor_double(tracked.m_posZ.get()),
                    )));
                }
            }
        }
        else if self.m_trackedPlayers.with(|set| set.contains(player))
        {
            self.m_trackedPlayers.update(|set| set.remove(player));
            entity_player_mp::netServerHandler(player)
                .sendPacket(Box::new(Packet29DestroyEntity::new(self.m_trackedEntity.m_entityId.get())));
        }
    }

    pub fn updatePlayerEntities(&self, players: &[Rc<Entity>])
    {
        for player in players
        {
            if !player.instanceOf("EntityPlayerMP")
            {
                game::port::classCast(player.className(), "EntityPlayerMP");
            }
            self.updatePlayerEntity(player);
        }
    }

    fn getSpawnPacket(&self) -> Box<dyn Packet>
    {
        let tracked: &Rc<Entity> = &self.m_trackedEntity;
        if tracked.instanceOf("EntityItem")
        {
            let packet21pickupspawn: Packet21PickupSpawn = Packet21PickupSpawn::new(tracked);
            tracked.m_posX.set(packet21pickupspawn.m_xPosition as f64 / 32.0);
            tracked.m_posY.set(packet21pickupspawn.m_yPosition as f64 / 32.0);
            tracked.m_posZ.set(packet21pickupspawn.m_zPosition as f64 / 32.0);
            return Box::new(packet21pickupspawn);
        }
        if tracked.instanceOf("EntityPlayerMP")
        {
            return Box::new(Packet20NamedEntitySpawn::new(tracked));
        }
        if tracked.instanceOf("EntityMinecart")
        {
            let minecartType: i32 = entity_minecart::fields(tracked).m_minecartType.get();
            if minecartType == 0
            {
                return Box::new(Packet23VehicleSpawn::new(tracked, 10));
            }
            if minecartType == 1
            {
                return Box::new(Packet23VehicleSpawn::new(tracked, 11));
            }
            if minecartType == 2
            {
                return Box::new(Packet23VehicleSpawn::new(tracked, 12));
            }
        }
        if tracked.instanceOf("EntityBoat")
        {
            return Box::new(Packet23VehicleSpawn::new(tracked, 1));
        }
        if tracked.instanceOf("IAnimals")
        {
            return Box::new(Packet24MobSpawn::new(tracked));
        }
        if tracked.instanceOf("EntityFish")
        {
            return Box::new(Packet23VehicleSpawn::new(tracked, 90));
        }
        if tracked.instanceOf("EntityArrow")
        {
            let entityliving: Option<Rc<Entity>> = entity_arrow::fields(tracked).m_owner.get();
            let data: i32 = match entityliving
            {
                Some(entityliving) => entityliving.m_entityId.get(),
                None => tracked.m_entityId.get(),
            };
            return Box::new(Packet23VehicleSpawn::withData(tracked, 60, data));
        }
        if tracked.instanceOf("EntitySnowball")
        {
            return Box::new(Packet23VehicleSpawn::new(tracked, 61));
        }
        if tracked.instanceOf("EntityFireball")
        {
            let entityfireball: &entity_fireball::FireballFields = entity_fireball::fields(tracked);
            let owner: Rc<Entity> = entityfireball.m_throwable.m_thrower.get().orNpe();
            let mut packet23vehiclespawn: Packet23VehicleSpawn =
                Packet23VehicleSpawn::withData(tracked, 63, owner.m_entityId.get());
            packet23vehiclespawn.m_velocityX = (entityfireball.m_accelerationX.get() * 8000.0) as i32;
            packet23vehiclespawn.m_velocityY = (entityfireball.m_accelerationY.get() * 8000.0) as i32;
            packet23vehiclespawn.m_velocityZ = (entityfireball.m_accelerationZ.get() * 8000.0) as i32;
            return Box::new(packet23vehiclespawn);
        }
        if tracked.instanceOf("EntityEgg")
        {
            return Box::new(Packet23VehicleSpawn::new(tracked, 62));
        }
        if tracked.instanceOf("EntityTNTPrimed")
        {
            return Box::new(Packet23VehicleSpawn::new(tracked, 50));
        }
        if tracked.instanceOf("EntityFallingSand")
        {
            let blockID: i32 = entity_falling_sand::fields(tracked).m_blockID.get();
            if blockID == Block::sand().m_blockID
            {
                return Box::new(Packet23VehicleSpawn::new(tracked, 70));
            }
            if blockID == Block::gravel().m_blockID
            {
                return Box::new(Packet23VehicleSpawn::new(tracked, 71));
            }
        }
        if tracked.instanceOf("EntityPainting")
        {
            return Box::new(Packet25EntityPainting::new(tracked));
        }
        jvm::throwable::throw(
            &jvm::throwable::ILLEGAL_ARGUMENT,
            &format!("Don't know how to add class net.minecraft.src.{}!", tracked.className()),
        )
    }

    pub fn removeTrackedPlayerSymmetric(&self, player: &Rc<Entity>)
    {
        if self.m_trackedPlayers.with(|set| set.contains(player))
        {
            self.m_trackedPlayers.update(|set| set.remove(player));
            entity_player_mp::netServerHandler(player)
                .sendPacket(Box::new(Packet29DestroyEntity::new(self.m_trackedEntity.m_entityId.get())));
        }
    }
}
