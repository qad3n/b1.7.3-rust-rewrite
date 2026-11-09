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

// vanilla resends the full position this often so clients cannot drift
const RESYNC_TICKS: i32 = 400;
// relative move packets carry one signed byte per axis
const RELATIVE_RANGE: std::ops::Range<i32> = -128..128;
// rotation is sent in 1/256 turns and only past this many steps as vanilla
const ROTATION_STEP: i32 = 8;
const VELOCITY_STEP: f64 = 0.02;

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
    m_sinceTeleport: Cell<i32>,
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
            m_sinceTeleport: Cell::new(0),
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

        self.m_sinceTeleport.set(self.m_sinceTeleport.get() + 1);
        self.m_updateCounter.set(self.m_updateCounter.get() + 1);
        if self.m_updateCounter.get() % self.m_updateInterval == 0
        {
            self.sendVelocity();
            self.sendMovement();
            self.sendMetadata();
        }

        if tracked.m_beenAttacked.get()
        {
            self.sendPacketToTrackedPlayersAndTrackedEntity(Box::new(Packet28EntityVelocity::fromEntity(tracked)));
            tracked.m_beenAttacked.set(false);
        }
    }

    // the encoded position only feeds packets and never moves the real entity
    fn sendMovement(&self)
    {
        let tracked: &Rc<Entity> = &self.m_trackedEntity;
        let id: i32 = tracked.m_entityId.get();
        let x: i32 = math_helper::floor_double(tracked.m_posX.get() * 32.0);
        let y: i32 = math_helper::floor_double(tracked.m_posY.get() * 32.0);
        let z: i32 = math_helper::floor_double(tracked.m_posZ.get() * 32.0);
        let yaw: i32 = math_helper::floor_float(tracked.m_rotationYaw.get() * 256.0 / 360.0);
        let pitch: i32 = math_helper::floor_float(tracked.m_rotationPitch.get() * 256.0 / 360.0);
        let dx: i32 = x - self.m_encodedPosX.get();
        let dy: i32 = y - self.m_encodedPosY.get();
        let dz: i32 = z - self.m_encodedPosZ.get();
        let moved: bool = dx != 0 || dy != 0 || dz != 0;
        let turned: bool = (yaw - self.m_encodedRotationYaw.get()).wrapping_abs() >= ROTATION_STEP
            || (pitch - self.m_encodedRotationPitch.get()).wrapping_abs() >= ROTATION_STEP;
        let far: bool = [dx, dy, dz].iter().any(|delta| !RELATIVE_RANGE.contains(delta));

        let packet: Box<dyn Packet> = if far || self.m_sinceTeleport.get() > RESYNC_TICKS
        {
            self.m_sinceTeleport.set(0);
            self.setEncoded(x, y, z);
            self.m_encodedRotationYaw.set(yaw);
            self.m_encodedRotationPitch.set(pitch);
            Box::new(Packet34EntityTeleport::withPosition(id, x, y, z, yaw as i8, pitch as i8))
        }
        else if moved && turned
        {
            self.setEncoded(x, y, z);
            self.m_encodedRotationYaw.set(yaw);
            self.m_encodedRotationPitch.set(pitch);
            Box::new(Packet33RelEntityMoveLook::new(id, dx as i8, dy as i8, dz as i8, yaw as i8, pitch as i8))
        }
        else if moved
        {
            self.setEncoded(x, y, z);
            Box::new(Packet31RelEntityMove::new(id, dx as i8, dy as i8, dz as i8))
        }
        else if turned
        {
            self.m_encodedRotationYaw.set(yaw);
            self.m_encodedRotationPitch.set(pitch);
            Box::new(Packet32EntityLook::new(id, yaw as i8, pitch as i8))
        }
        else
        {
            return;
        };

        self.sendPacketToTrackedPlayers(packet);
    }

    fn setEncoded(&self, x: i32, y: i32, z: i32)
    {
        self.m_encodedPosX.set(x);
        self.m_encodedPosY.set(y);
        self.m_encodedPosZ.set(z);
    }

    fn sendVelocity(&self)
    {
        if !self.m_shouldSendMotionUpdates
        {
            return;
        }

        let tracked: &Rc<Entity> = &self.m_trackedEntity;
        let (motionX, motionY, motionZ): (f64, f64, f64) =
            (tracked.m_motionX.get(), tracked.m_motionY.get(), tracked.m_motionZ.get());
        let dx: f64 = motionX - self.m_lastTrackedEntityMotionX.get();
        let dy: f64 = motionY - self.m_lastTrackedEntityMotionY.get();
        let dz: f64 = motionZ - self.m_lastTrackedEntityMotionZ.get();
        let change: f64 = dx * dx + dy * dy + dz * dz;
        let stopped: bool = motionX == 0.0 && motionY == 0.0 && motionZ == 0.0;
        if change <= VELOCITY_STEP * VELOCITY_STEP && !(change > 0.0 && stopped)
        {
            return;
        }

        self.m_lastTrackedEntityMotionX.set(motionX);
        self.m_lastTrackedEntityMotionY.set(motionY);
        self.m_lastTrackedEntityMotionZ.set(motionZ);
        self.sendPacketToTrackedPlayers(Box::new(Packet28EntityVelocity::new(
            tracked.m_entityId.get(),
            motionX,
            motionY,
            motionZ,
        )));
    }

    fn sendMetadata(&self)
    {
        let tracked: &Rc<Entity> = &self.m_trackedEntity;
        let watcher: &DataWatcher = tracked.getDataWatcher();
        if !watcher.hasObjectChanged()
        {
            return;
        }

        self.sendPacketToTrackedPlayersAndTrackedEntity(Box::new(Packet40EntityMetadata::new(
            tracked.m_entityId.get(),
            watcher,
        )));
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
            // rewrite: a mod entity is not tracked for a player until its mod handshake is done
            if !self.m_trackedPlayers.with(|set| set.contains(player))
                && modloader::entities::canTrack(&self.m_trackedEntity, player)
            {
                self.m_trackedPlayers.update(|set| set.add(Rc::clone(player)));
                let handler: Rc<crate::network::NetServerHandler> = entity_player_mp::netServerHandler(player);
                // mod entities spawn through the loader channel
                if !modloader::entities::sendSpawn(&self.m_trackedEntity, player)
                {
                    handler.sendPacket(self.getSpawnPacket());
                    // rewrite: mod data synced on vanilla entities, as players' equipment, follows their spawn
                    modloader::entities::sendSynced(&self.m_trackedEntity, player);
                }
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
            return Box::new(Packet21PickupSpawn::new(tracked));
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
