use crate::player::EntityTrackerEntry;
use crate::server::MinecraftServer;
use game::entity::Entity;
use game::network::packet::Packet;
use game::pathfinding::MCHash;
use jvm::field::Field;
use jvm::hash_map::JHashSet;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

pub struct EntityTracker
{
    m_trackedEntitySet: Field<JHashSet<Rc<EntityTrackerEntry>>>,
    m_trackedEntityHashTable: RefCell<MCHash<Rc<EntityTrackerEntry>>>,
    m_mcServer: Rc<MinecraftServer>,
    m_maxTrackingDistanceThreshold: Cell<i32>,
    m_dimension: i32,
}

impl EntityTracker
{
    pub fn new(server: &Rc<MinecraftServer>, dimension: i32) -> Rc<EntityTracker>
    {
        Rc::new(EntityTracker {
            m_trackedEntitySet: Field::new(JHashSet::new()),
            m_trackedEntityHashTable: RefCell::new(MCHash::new()),
            m_mcServer: Rc::clone(server),
            m_dimension: dimension,
            m_maxTrackingDistanceThreshold: Cell::new(server.configManager().getMaxTrackingDistance()),
        })
    }

    fn entries(&self) -> Vec<Rc<EntityTrackerEntry>> { self.m_trackedEntitySet.with(JHashSet::items) }

    fn players(&self) -> Vec<Rc<Entity>> { self.m_mcServer.getWorldManager(self.m_dimension).m_playerEntities.toVec() }

    pub fn trackEntity(&self, entity: &Rc<Entity>)
    {
        if entity.instanceOf("EntityPlayerMP")
        {
            self.trackEntityWith(entity, 512, 2);
            for entitytrackerentry in self.entries()
            {
                if !Rc::ptr_eq(&entitytrackerentry.m_trackedEntity, entity)
                {
                    entitytrackerentry.updatePlayerEntity(entity);
                }
            }
        }
        else if entity.instanceOf("EntityFish")
        {
            self.trackEntityWithVelocity(entity, 64, 5, true);
        }
        else if entity.instanceOf("EntityArrow")
        {
            self.trackEntityWithVelocity(entity, 64, 20, false);
        }
        else if entity.instanceOf("EntityFireball")
        {
            self.trackEntityWithVelocity(entity, 64, 10, false);
        }
        else if entity.instanceOf("EntitySnowball")
        {
            self.trackEntityWithVelocity(entity, 64, 10, true);
        }
        else if entity.instanceOf("EntityEgg")
        {
            self.trackEntityWithVelocity(entity, 64, 10, true);
        }
        else if entity.instanceOf("EntityItem")
        {
            self.trackEntityWithVelocity(entity, 64, 20, true);
        }
        else if entity.instanceOf("EntityMinecart")
        {
            self.trackEntityWithVelocity(entity, 160, 5, true);
        }
        else if entity.instanceOf("EntityBoat")
        {
            self.trackEntityWithVelocity(entity, 160, 5, true);
        }
        else if entity.instanceOf("EntitySquid")
        {
            self.trackEntityWithVelocity(entity, 160, 3, true);
        }
        else if entity.instanceOf("IAnimals")
        {
            self.trackEntityWith(entity, 160, 3);
        }
        else if entity.instanceOf("EntityTNTPrimed")
        {
            self.trackEntityWithVelocity(entity, 160, 10, true);
        }
        else if entity.instanceOf("EntityFallingSand")
        {
            self.trackEntityWithVelocity(entity, 160, 20, true);
        }
        else if entity.instanceOf("EntityPainting")
        {
            self.trackEntityWithVelocity(entity, 160, i32::MAX, false);
        }
    }

    pub fn trackEntityWith(&self, entity: &Rc<Entity>, trackingRange: i32, updateInterval: i32)
    {
        self.trackEntityWithVelocity(entity, trackingRange, updateInterval, false);
    }

    pub fn trackEntityWithVelocity(
        &self,
        entity: &Rc<Entity>,
        trackingRange: i32,
        updateInterval: i32,
        syncVelocity: bool,
    )
    {
        let mut trackingRange: i32 = trackingRange;
        if trackingRange > self.m_maxTrackingDistanceThreshold.get()
        {
            trackingRange = self.m_maxTrackingDistanceThreshold.get();
        }
        if self.m_trackedEntityHashTable.borrow().containsItem(entity.m_entityId.get())
        {
            jvm::throwable::throw(&jvm::throwable::ILLEGAL_STATE, "Entity is already tracked!");
        }
        let entitytrackerentry: Rc<EntityTrackerEntry> =
            EntityTrackerEntry::new(entity, trackingRange, updateInterval, syncVelocity);
        self.m_trackedEntitySet.update(|set| set.add(Rc::clone(&entitytrackerentry)));
        self.m_trackedEntityHashTable.borrow_mut().addKey(entity.m_entityId.get(), Rc::clone(&entitytrackerentry));
        entitytrackerentry.updatePlayerEntities(&self.players());
    }

    pub fn untrackEntity(&self, entity: &Rc<Entity>)
    {
        if entity.instanceOf("EntityPlayerMP")
        {
            for entitytrackerentry in self.entries()
            {
                entitytrackerentry.removeFromTrackedPlayers(entity);
            }
        }
        let entitytrackerentry1: Option<Rc<EntityTrackerEntry>> =
            self.m_trackedEntityHashTable.borrow_mut().removeObject(entity.m_entityId.get());
        if let Some(entitytrackerentry1) = entitytrackerentry1
        {
            self.m_trackedEntitySet.update(|set| set.remove(&entitytrackerentry1));
            entitytrackerentry1.sendDestroyEntityPacketToTrackedPlayers();
        }
    }

    pub fn updateTrackedEntities(&self)
    {
        let mut arraylist: Vec<Rc<Entity>> = Vec::new();
        for entitytrackerentry in self.entries()
        {
            entitytrackerentry.updatePlayerList(&self.players());
            if entitytrackerentry.m_playerEntitiesUpdated.get()
                && entitytrackerentry.m_trackedEntity.instanceOf("EntityPlayerMP")
            {
                arraylist.push(Rc::clone(&entitytrackerentry.m_trackedEntity));
            }
        }
        for entityplayermp in &arraylist
        {
            for entitytrackerentry1 in self.entries()
            {
                if !Rc::ptr_eq(&entitytrackerentry1.m_trackedEntity, entityplayermp)
                {
                    entitytrackerentry1.updatePlayerEntity(entityplayermp);
                }
            }
        }
    }

    pub fn sendPacketToTrackedPlayers(&self, entity: &Entity, packet: Box<dyn Packet>)
    {
        let entitytrackerentry: Option<Rc<EntityTrackerEntry>> =
            self.m_trackedEntityHashTable.borrow().lookup(entity.m_entityId.get());
        if let Some(entitytrackerentry) = entitytrackerentry
        {
            entitytrackerentry.sendPacketToTrackedPlayers(packet);
        }
    }

    pub fn sendPacketToTrackedPlayersAndTrackedEntity(&self, entity: &Entity, packet: Box<dyn Packet>)
    {
        let entitytrackerentry: Option<Rc<EntityTrackerEntry>> =
            self.m_trackedEntityHashTable.borrow().lookup(entity.m_entityId.get());
        if let Some(entitytrackerentry) = entitytrackerentry
        {
            entitytrackerentry.sendPacketToTrackedPlayersAndTrackedEntity(packet);
        }
    }

    pub fn removeTrackedPlayerSymmetric(&self, player: &Rc<Entity>)
    {
        for entitytrackerentry in self.entries()
        {
            entitytrackerentry.removeTrackedPlayerSymmetric(player);
        }
    }
}
