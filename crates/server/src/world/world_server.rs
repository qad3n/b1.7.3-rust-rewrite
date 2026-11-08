use crate::server::MinecraftServer;
use crate::world::ChunkProviderServer;
use game::entity::Entity;
use game::math::math_helper;
use game::network::packets::{
    Packet38EntityStatus, Packet54PlayNoteBlock, Packet60Explosion, Packet70Bed, Packet71Weather,
};
use game::pathfinding::MCHash;
use game::tileentity::TileEntity;
use game::world::{Explosion, IChunkLoader, IChunkProvider, ISaveHandler, WORLD, World, WorldClass, WorldProvider};
use jvm::field::Field;
use jvm::throwable::OrNpe;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

pub struct WorldServerExt
{
    pub m_chunkProviderServer: Field<Option<Rc<ChunkProviderServer>>>,
    pub m_bypassSpawnProtection: Cell<bool>,
    pub m_levelSaving: Cell<bool>,
    m_mcServer: Field<Option<Rc<MinecraftServer>>>,
    m_entitiesById: RefCell<MCHash<Rc<Entity>>>,
}

pub fn fields(this: &World) -> &WorldServerExt { this.ext::<WorldServerExt>() }

fn mcServer(this: &World) -> Rc<MinecraftServer> { fields(this).m_mcServer.get().orNpe() }

pub fn chunkProviderServer(this: &World) -> Rc<ChunkProviderServer> { fields(this).m_chunkProviderServer.get().orNpe() }

pub fn alloc(class: &'static WorldClass) -> Rc<World>
{
    let ext: WorldServerExt = WorldServerExt {
        m_chunkProviderServer: Field::new(None),
        m_bypassSpawnProtection: Cell::new(false),
        m_levelSaving: Cell::new(false),
        m_mcServer: Field::new(None),
        m_entitiesById: RefCell::new(MCHash::new()),
    };
    Rc::new(World::alloc(class, Box::new(ext)))
}

pub fn init(
    this: &Rc<World>,
    server: &Rc<MinecraftServer>,
    storage: Rc<dyn ISaveHandler>,
    name: &str,
    dimension: i32,
    seed: i64,
)
{
    World::initFromSave(this, storage, name, seed, Some(WorldProvider::getProviderForDimension(dimension)));
    fields(this).m_mcServer.set(Some(Rc::clone(server)));
}

pub fn new(
    server: &Rc<MinecraftServer>,
    storage: Rc<dyn ISaveHandler>,
    name: &str,
    dimension: i32,
    seed: i64,
) -> Rc<World>
{
    let this: Rc<World> = alloc(&CLASS);
    init(&this, server, storage, name, dimension, seed);
    this
}

fn updateEntityWithOptionalForce(this: &Rc<World>, entity: &Rc<Entity>, requireLoaded: bool)
{
    if !mcServer(this).m_spawnPeacefulMobs.get()
        && (entity.instanceOf("EntityAnimal") || entity.instanceOf("EntityWaterMob"))
    {
        entity.setEntityDead();
    }
    match entity.m_riddenByEntity.get()
    {
        Some(rider) if rider.instanceOf("EntityPlayer") =>
        {}
        _ => (WORLD.updateEntityWithOptionalForce)(this, entity, requireLoaded),
    }
}

pub fn tickVehicle(this: &Rc<World>, vehicle: &Rc<Entity>, requireLoaded: bool)
{
    (WORLD.updateEntityWithOptionalForce)(this, vehicle, requireLoaded);
}

fn getChunkProvider(this: &Rc<World>) -> Rc<dyn IChunkProvider>
{
    let provider: Rc<WorldProvider> = this.worldProvider();
    let ichunkloader: Rc<dyn IChunkLoader> = this.saveHandler().getChunkLoader(&provider);
    let chunkProviderServer: Rc<ChunkProviderServer> =
        ChunkProviderServer::new(this, ichunkloader, provider.getChunkProvider());
    fields(this).m_chunkProviderServer.set(Some(Rc::clone(&chunkProviderServer)));
    chunkProviderServer
}

pub fn getTileEntityList(
    this: &World,
    minX: i32,
    minY: i32,
    minZ: i32,
    maxX: i32,
    maxY: i32,
    maxZ: i32,
) -> Vec<Rc<TileEntity>>
{
    let mut arraylist: Vec<Rc<TileEntity>> = Vec::new();
    let mut i: i32 = 0;
    while i < this.m_loadedTileEntityList.size()
    {
        let tileentity: Rc<TileEntity> = this.m_loadedTileEntityList.get(i);
        if tileentity.m_xCoord.get() >= minX
            && tileentity.m_yCoord.get() >= minY
            && tileentity.m_zCoord.get() >= minZ
            && tileentity.m_xCoord.get() < maxX
            && tileentity.m_yCoord.get() < maxY
            && tileentity.m_zCoord.get() < maxZ
        {
            arraylist.push(tileentity);
        }
        i += 1;
    }
    arraylist
}

fn canModify(this: &Rc<World>, player: &Rc<Entity>, x: i32, _y: i32, z: i32) -> bool
{
    let i: i32 = math_helper::abs((x - this.worldInfo().getSpawnX()) as f32) as i32;
    let mut j: i32 = math_helper::abs((z - this.worldInfo().getSpawnZ()) as f32) as i32;
    if i > j
    {
        j = i;
    }
    j > 16 || mcServer(this).configManager().isOp(game::entity::entity_player::fields(player).m_username.get().as_ref())
}

fn obtainEntitySkin(this: &Rc<World>, entity: &Rc<Entity>)
{
    (WORLD.obtainEntitySkin)(this, entity);
    fields(this).m_entitiesById.borrow_mut().addKey(entity.m_entityId.get(), Rc::clone(entity));
}

fn releaseEntitySkin(this: &Rc<World>, entity: &Rc<Entity>)
{
    (WORLD.releaseEntitySkin)(this, entity);
    fields(this).m_entitiesById.borrow_mut().removeObject(entity.m_entityId.get());
}

pub fn getEntity(this: &World, networkId: i32) -> Option<Rc<Entity>>
{
    fields(this).m_entitiesById.borrow().lookup(networkId)
}

fn addWeatherEffect(this: &Rc<World>, entity: Rc<Entity>) -> bool
{
    if (WORLD.addWeatherEffect)(this, Rc::clone(&entity))
    {
        mcServer(this).configManager().sendPacketToPlayersAroundPoint(
            entity.m_posX.get(),
            entity.m_posY.get(),
            entity.m_posZ.get(),
            512.0,
            this.worldProvider().m_worldType.get(),
            Box::new(Packet71Weather::new(&entity)),
        );
        return true;
    }
    false
}

fn doEntityEvent(this: &Rc<World>, entity: &Rc<Entity>, event: i8)
{
    let packet38entitystatus: Packet38EntityStatus = Packet38EntityStatus::new(entity.m_entityId.get(), event);
    mcServer(this)
        .getEntityTracker(this.worldProvider().m_worldType.get())
        .sendPacketToTrackedPlayersAndTrackedEntity(entity, Box::new(packet38entitystatus));
}

fn newExplosion(
    this: &Rc<World>,
    source: Option<&Rc<Entity>>,
    x: f64,
    y: f64,
    z: f64,
    power: f32,
    createFire: bool,
) -> Rc<Explosion>
{
    let explosion: Rc<Explosion> = Explosion::new(this, source, x, y, z, power);
    explosion.m_isFlaming.set(createFire);
    explosion.doExplosionA();
    explosion.doExplosionB(false);
    let packet: Packet60Explosion =
        explosion.m_destroyedBlockPositions.with(|positions| Packet60Explosion::new(x, y, z, power, positions));
    mcServer(this).configManager().sendPacketToPlayersAroundPoint(
        x,
        y,
        z,
        64.0,
        this.worldProvider().m_worldType.get(),
        Box::new(packet),
    );
    explosion
}

fn playNoteAt(this: &Rc<World>, x: i32, y: i32, z: i32, kind: i32, data: i32)
{
    (WORLD.playNoteAt)(this, x, y, z, kind, data);
    mcServer(this).configManager().sendPacketToPlayersAroundPoint(
        x as f64,
        y as f64,
        z as f64,
        64.0,
        this.worldProvider().m_worldType.get(),
        Box::new(Packet54PlayNoteBlock::new(x, y, z, kind, data)),
    );
}

pub fn forceSave(this: &World) { this.saveHandler().forceSave(); }

fn updateWeather(this: &Rc<World>)
{
    let flag: bool = this.isRaining();
    (WORLD.updateWeather)(this);
    if flag != this.isRaining()
    {
        if flag
        {
            mcServer(this).configManager().sendPacketToAllPlayers(Box::new(Packet70Bed::new(2)));
        }
        else
        {
            mcServer(this).configManager().sendPacketToAllPlayers(Box::new(Packet70Bed::new(1)));
        }
    }
}

pub const CLASS: WorldClass = WorldClass {
    m_name: "WorldServer",
    m_parent: Some(&WORLD),
    updateEntityWithOptionalForce,
    getChunkProvider,
    canModify,
    obtainEntitySkin,
    releaseEntitySkin,
    addWeatherEffect,
    doEntityEvent,
    newExplosion,
    playNoteAt,
    updateWeather,
    ..WORLD
};
