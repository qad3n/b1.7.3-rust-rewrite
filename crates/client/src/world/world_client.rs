use crate::network::net_client_handler::NetClientHandler;
use crate::world::chunk_provider_client::ChunkProviderClient;
use game::entity::Entity;
use game::network::packets::Packet255KickDisconnect;
use game::pathfinding::MCHash;
use game::storage::SaveHandlerMP;
use game::world::{ChunkCoordinates, IChunkProvider, WORLD, World, WorldClass, WorldProvider};
use jvm::array_list::JArrayList;
use jvm::field::Field;
use jvm::hash_map::{HashCursor, JHashSet};
use jvm::string::JString;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

pub(crate) struct WorldBlockPositionType
{
    m_x: i32,
    m_y: i32,
    m_z: i32,
    m_delay: Cell<i32>,
    m_block: i32,
    m_metadata: i32,
}

impl WorldBlockPositionType
{
    fn new(x: i32, y: i32, z: i32, block: i32, metadata: i32) -> Rc<WorldBlockPositionType>
    {
        Rc::new(WorldBlockPositionType {
            m_x: x,
            m_y: y,
            m_z: z,
            m_delay: Cell::new(80),
            m_block: block,
            m_metadata: metadata,
        })
    }
}

pub struct WorldClientExt
{
    pub(crate) m_blockResets: JArrayList<Rc<WorldBlockPositionType>>,
    m_sendQueue: Rc<NetClientHandler>,
    m_chunkCache: Field<Option<Rc<ChunkProviderClient>>>,
    pub(crate) m_entitiesByNetworkId: RefCell<MCHash<Rc<Entity>>>,
    pub(crate) m_forcedEntities: Field<JHashSet<Rc<Entity>>>,
    pub(crate) m_pendingEntities: Field<JHashSet<Rc<Entity>>>,
}

fn fields(this: &World) -> &WorldClientExt { this.ext::<WorldClientExt>() }

fn sendQueue(this: &World) -> Rc<NetClientHandler> { Rc::clone(&fields(this).m_sendQueue) }

fn chunkCache(this: &World) -> Rc<ChunkProviderClient> { jvm::throwable::OrNpe::orNpe(fields(this).m_chunkCache.get()) }

pub fn new(networkHandler: &Rc<NetClientHandler>, seed: i64, dimension: i32) -> Rc<World>
{
    let ext: WorldClientExt = WorldClientExt {
        m_blockResets: JArrayList::new(),
        m_sendQueue: Rc::clone(networkHandler),
        m_chunkCache: Field::new(None),
        m_entitiesByNetworkId: RefCell::new(MCHash::new()),
        m_forcedEntities: Field::new(JHashSet::new()),
        m_pendingEntities: Field::new(JHashSet::new()),
    };
    let this: Rc<World> = Rc::new(World::alloc(&CLASS, Box::new(ext)));
    World::initWithProvider(
        &this,
        Rc::new(SaveHandlerMP),
        "MpServer",
        WorldProvider::getProviderForDimension(dimension),
        seed,
    );
    this.setSpawnPoint(&ChunkCoordinates::new(8, 64, 8));
    this.m_savedDataStorage.set(Some(Rc::clone(&networkHandler.m_savedDataStorage)));
    this
}

fn firstPending(this: &World) -> Rc<Entity>
{
    fields(this).m_pendingEntities.with(|set| {
        let mut cursor: HashCursor = set.cursor();
        set.nextKey(&mut cursor)
    })
}

fn tick(this: &Rc<World>)
{
    this.setWorldTime(this.getWorldTime() + 1);
    let i: i32 = this.calculateSkylightSubtracted(1.0);
    if i != this.m_skylightSubtracted.get()
    {
        this.m_skylightSubtracted.set(i);
        let mut j: i32 = 0;
        while j < this.m_worldAccesses.size()
        {
            this.m_worldAccesses.get(j).updateAllRenderers();
            j += 1;
        }
    }
    let ext: &WorldClientExt = fields(this);
    let mut k: i32 = 0;
    while k < 10 && !ext.m_pendingEntities.with(JHashSet::isEmpty)
    {
        let entity: Rc<Entity> = firstPending(this);
        if !this.m_loadedEntityList.contains(&entity)
        {
            this.entityJoinedWorld(&entity);
        }
        k += 1;
    }
    sendQueue(this).processReadPackets();
    let mut l: i32 = 0;
    while l < ext.m_blockResets.size()
    {
        let worldblockpositiontype: Rc<WorldBlockPositionType> = ext.m_blockResets.get(l);
        worldblockpositiontype.m_delay.set(worldblockpositiontype.m_delay.get() - 1);
        if worldblockpositiontype.m_delay.get() == 0
        {
            (WORLD.setBlockAndMetadata)(
                this,
                worldblockpositiontype.m_x,
                worldblockpositiontype.m_y,
                worldblockpositiontype.m_z,
                worldblockpositiontype.m_block,
                worldblockpositiontype.m_metadata,
            );
            this.markBlockNeedsUpdate(
                worldblockpositiontype.m_x,
                worldblockpositiontype.m_y,
                worldblockpositiontype.m_z,
            );
            ext.m_blockResets.removeAt(l);
            l -= 1;
        }
        l += 1;
    }
}

pub fn clearBlockResets(this: &World, minX: i32, minY: i32, minZ: i32, maxX: i32, maxY: i32, maxZ: i32)
{
    let blockResets: &JArrayList<Rc<WorldBlockPositionType>> = &fields(this).m_blockResets;
    let mut i: i32 = 0;
    while i < blockResets.size()
    {
        let worldblockpositiontype: Rc<WorldBlockPositionType> = blockResets.get(i);
        if worldblockpositiontype.m_x >= minX
            && worldblockpositiontype.m_y >= minY
            && worldblockpositiontype.m_z >= minZ
            && worldblockpositiontype.m_x <= maxX
            && worldblockpositiontype.m_y <= maxY
            && worldblockpositiontype.m_z <= maxZ
        {
            blockResets.removeAt(i);
            i -= 1;
        }
        i += 1;
    }
}

fn getChunkProvider(this: &Rc<World>) -> Rc<dyn IChunkProvider>
{
    let chunkCache: Rc<ChunkProviderClient> = Rc::new(ChunkProviderClient::new(this));
    fields(this).m_chunkCache.set(Some(Rc::clone(&chunkCache)));
    chunkCache
}

pub fn doPreChunk(this: &World, chunkX: i32, chunkZ: i32, load: bool)
{
    if load
    {
        chunkCache(this).prepareChunk(chunkX, chunkZ);
    }
    else
    {
        chunkCache(this).unloadChunk(chunkX, chunkZ);
    }
    if !load
    {
        this.markBlocksDirty(chunkX * 16, 0, chunkZ * 16, chunkX * 16 + 15, 128, chunkZ * 16 + 15);
    }
}

fn entityJoinedWorld(this: &Rc<World>, entity: &Rc<Entity>) -> bool
{
    let flag: bool = (WORLD.entityJoinedWorld)(this, entity);
    let ext: &WorldClientExt = fields(this);
    ext.m_forcedEntities.update(|set| set.add(Rc::clone(entity)));
    if !flag
    {
        ext.m_pendingEntities.update(|set| set.add(Rc::clone(entity)));
    }
    flag
}

fn setEntityDead(this: &Rc<World>, entity: &Rc<Entity>)
{
    (WORLD.setEntityDead)(this, entity);
    fields(this).m_forcedEntities.update(|set| set.remove(entity));
}

fn obtainEntitySkin(this: &Rc<World>, entity: &Rc<Entity>)
{
    (WORLD.obtainEntitySkin)(this, entity);
    let pendingEntities: &Field<JHashSet<Rc<Entity>>> = &fields(this).m_pendingEntities;
    if pendingEntities.with(|set| set.contains(entity))
    {
        pendingEntities.update(|set| set.remove(entity));
    }
}

fn releaseEntitySkin(this: &Rc<World>, entity: &Rc<Entity>)
{
    (WORLD.releaseEntitySkin)(this, entity);
    let ext: &WorldClientExt = fields(this);
    if ext.m_forcedEntities.with(|set| set.contains(entity))
    {
        ext.m_pendingEntities.update(|set| set.add(Rc::clone(entity)));
    }
}

pub fn forceEntity(this: &Rc<World>, networkId: i32, entity: &Rc<Entity>)
{
    if let Some(entityx) = getEntity(this, networkId)
    {
        this.setEntityDead(&entityx);
    }
    let ext: &WorldClientExt = fields(this);
    ext.m_forcedEntities.update(|set| set.add(Rc::clone(entity)));
    entity.m_entityId.set(networkId);
    if !this.entityJoinedWorld(entity)
    {
        ext.m_pendingEntities.update(|set| set.add(Rc::clone(entity)));
    }
    ext.m_entitiesByNetworkId.borrow_mut().addKey(networkId, Rc::clone(entity));
}

pub fn getEntity(this: &World, networkId: i32) -> Option<Rc<Entity>>
{
    fields(this).m_entitiesByNetworkId.borrow().lookup(networkId)
}

pub fn removeEntityFromWorld(this: &Rc<World>, networkId: i32) -> Option<Rc<Entity>>
{
    let ext: &WorldClientExt = fields(this);
    let entity: Option<Rc<Entity>> = ext.m_entitiesByNetworkId.borrow_mut().removeObject(networkId);
    if let Some(entity) = &entity
    {
        ext.m_forcedEntities.update(|set| set.remove(entity));
        this.setEntityDead(entity);
    }
    entity
}

fn addBlockReset(this: &World, x: i32, y: i32, z: i32, block: i32, metadata: i32)
{
    fields(this).m_blockResets.add(WorldBlockPositionType::new(x, y, z, block, metadata));
}

fn setBlockMetadata(this: &Rc<World>, x: i32, y: i32, z: i32, metadata: i32) -> bool
{
    let i: i32 = this.getBlockId(x, y, z);
    let j: i32 = this.getBlockMetadata(x, y, z);
    if (WORLD.setBlockMetadata)(this, x, y, z, metadata)
    {
        addBlockReset(this, x, y, z, i, j);
        return true;
    }
    false
}

fn setBlockAndMetadata(this: &Rc<World>, x: i32, y: i32, z: i32, block: i32, metadata: i32) -> bool
{
    let i: i32 = this.getBlockId(x, y, z);
    let j: i32 = this.getBlockMetadata(x, y, z);
    if (WORLD.setBlockAndMetadata)(this, x, y, z, block, metadata)
    {
        addBlockReset(this, x, y, z, i, j);
        return true;
    }
    false
}

fn setBlock(this: &Rc<World>, x: i32, y: i32, z: i32, block: i32) -> bool
{
    let i: i32 = this.getBlockId(x, y, z);
    let j: i32 = this.getBlockMetadata(x, y, z);
    if (WORLD.setBlock)(this, x, y, z, block)
    {
        addBlockReset(this, x, y, z, i, j);
        return true;
    }
    false
}

pub fn setBlockWithMetadataFromPacket(this: &Rc<World>, x: i32, y: i32, z: i32, block: i32, metadata: i32) -> bool
{
    clearBlockResets(this, x, y, z, x, y, z);
    if (WORLD.setBlockAndMetadata)(this, x, y, z, block, metadata)
    {
        this.notifyBlockChange(x, y, z, block);
        return true;
    }
    false
}

fn updateWeather(this: &Rc<World>)
{
    if this.worldProvider().m_hasNoSky.get()
    {
        return;
    }
    if this.m_thunderCooldown.get() > 0
    {
        this.m_thunderCooldown.set(this.m_thunderCooldown.get() - 1);
    }
    this.m_prevRainingStrength.set(this.m_rainingStrength.get());
    if this.worldInfo().getRaining()
    {
        this.m_rainingStrength.set((this.m_rainingStrength.get() as f64 + 0.01) as f32);
    }
    else
    {
        this.m_rainingStrength.set((this.m_rainingStrength.get() as f64 - 0.01) as f32);
    }
    if this.m_rainingStrength.get() < 0.0
    {
        this.m_rainingStrength.set(0.0);
    }
    if this.m_rainingStrength.get() > 1.0
    {
        this.m_rainingStrength.set(1.0);
    }
    this.m_prevThunderingStrength.set(this.m_thunderingStrength.get());
    if this.worldInfo().getThundering()
    {
        this.m_thunderingStrength.set((this.m_thunderingStrength.get() as f64 + 0.01) as f32);
    }
    else
    {
        this.m_thunderingStrength.set((this.m_thunderingStrength.get() as f64 - 0.01) as f32);
    }
    if this.m_thunderingStrength.get() < 0.0
    {
        this.m_thunderingStrength.set(0.0);
    }
    if this.m_thunderingStrength.get() > 1.0
    {
        this.m_thunderingStrength.set(1.0);
    }
}

pub const CLASS: WorldClass = WorldClass {
    m_name: "WorldClient",
    m_parent: Some(&WORLD),
    m_interfaces: &[],
    tick,
    getChunkProvider,
    setSpawnLocation: |this| this.setSpawnPoint(&ChunkCoordinates::new(8, 64, 8)),
    updateBlocksAndPlayCaveSounds: |_| {},
    scheduleBlockUpdate: |_, _, _, _, _, _| {},
    TickUpdates: |_, _| false,
    entityJoinedWorld,
    setEntityDead,
    obtainEntitySkin,
    releaseEntitySkin,
    setBlockMetadata,
    setBlockAndMetadata,
    setBlock,
    sendQuittingDisconnectingPacket: |this| {
        sendQueue(this).sendPacketAndDisconnect(Box::new(Packet255KickDisconnect::new(JString::from("Quitting"))));
    },
    updateWeather,
    ..WORLD
};
