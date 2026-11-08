mod blocks;
mod class;
mod entities;
mod light;
mod power;
mod raytrace;
mod ticks;
mod weather;

use crate::entity::Entity;
use crate::math::AxisAlignedBB;
use crate::tileentity::TileEntity;
use crate::world::{
    IChunkProvider, ISaveHandler, IWorldAccess, MapStorage, MetadataChunkBlock, WorldInfo, WorldProvider,
};
pub use class::{WORLD, WorldClass};
use common::world::{ChunkCoordIntPair, NextTickListEntry};
use jvm::array_list::JArrayList;
use jvm::field::Field;
use jvm::hash_map::JHashSet;
use jvm::random::Random;
use jvm::throwable::OrNpe;
use std::any::Any;
use std::cell::Cell;
use std::collections::BTreeSet;
use std::rc::Rc;

thread_local! {
    pub(crate) static LIGHTING_UPDATES_SCHEDULED: Cell<i32> = const { Cell::new(0) };
}

pub struct World
{
    pub m_class: &'static WorldClass,
    pub m_ext: Box<dyn Any>,
    pub m_scheduledUpdatesAreImmediate: Cell<bool>,
    pub m_lightingToUpdate: JArrayList<Rc<MetadataChunkBlock>>,
    pub m_loadedEntityList: Rc<JArrayList<Rc<Entity>>>,
    pub m_unloadedEntityList: JArrayList<Rc<Entity>>,
    pub m_scheduledTickTreeSet: Field<BTreeSet<NextTickListEntry>>,
    pub m_scheduledTickSet: Field<JHashSet<NextTickListEntry>>,
    pub m_loadedTileEntityList: JArrayList<Rc<TileEntity>>,
    pub m_pendingBlockEntities: JArrayList<Rc<TileEntity>>,
    pub m_playerEntities: JArrayList<Rc<Entity>>,
    pub m_weatherEffects: JArrayList<Rc<Entity>>,
    pub m_cloudColor: Cell<i64>,
    pub m_skylightSubtracted: Cell<i32>,
    pub m_randomTickLCG: Cell<i32>,
    pub m_prevRainingStrength: Cell<f32>,
    pub m_rainingStrength: Cell<f32>,
    pub m_prevThunderingStrength: Cell<f32>,
    pub m_thunderingStrength: Cell<f32>,
    pub m_thunderCooldown: Cell<i32>,
    pub m_lightningCooldown: Cell<i32>,
    pub m_editingBlocks: Cell<bool>,
    pub m_lockTimestamp: Cell<i64>,
    pub m_autosavePeriod: Cell<i32>,
    pub m_difficultySetting: Cell<i32>,
    pub m_rand: Random,
    pub m_isNewWorld: Cell<bool>,
    pub m_worldProvider: Field<Option<Rc<WorldProvider>>>,
    pub m_worldAccesses: JArrayList<Rc<dyn IWorldAccess>>,
    pub m_chunkProvider: Field<Option<Rc<dyn IChunkProvider>>>,
    pub m_saveHandler: Field<Option<Rc<dyn ISaveHandler>>>,
    pub m_worldInfo: Field<Option<Rc<WorldInfo>>>,
    pub m_findingSpawnPoint: Cell<bool>,
    pub m_allPlayersSleeping: Cell<bool>,
    pub m_savedDataStorage: Field<Option<Rc<MapStorage>>>,
    pub m_collidingBoundingBoxes: Rc<JArrayList<Rc<AxisAlignedBB>>>,
    pub m_isTickingBlockEntities: Cell<bool>,
    pub m_lightingUpdatesCounter: Cell<i32>,
    pub m_spawnHostileMobs: Cell<bool>,
    pub m_spawnPeacefulMobs: Cell<bool>,
    pub m_positionsToUpdate: Field<JHashSet<ChunkCoordIntPair>>,
    pub m_soundCounter: Cell<i32>,
    pub m_entitiesInArea: Rc<JArrayList<Rc<Entity>>>,
    pub m_multiplayerWorld: Cell<bool>,
}

impl World
{
    pub fn alloc(class: &'static WorldClass, ext: Box<dyn Any>) -> Self
    {
        let randomTickLCG: i32 = Random::new().nextInt();
        let rand: Random = Random::new();
        let soundCounter: i32 = rand.nextIntBounded(12000);
        Self {
            m_class: class,
            m_ext: ext,
            m_scheduledUpdatesAreImmediate: Cell::new(false),
            m_lightingToUpdate: JArrayList::new(),
            m_loadedEntityList: Rc::new(JArrayList::new()),
            m_unloadedEntityList: JArrayList::new(),
            m_scheduledTickTreeSet: Field::new(BTreeSet::new()),
            m_scheduledTickSet: Field::new(JHashSet::new()),
            m_loadedTileEntityList: JArrayList::new(),
            m_pendingBlockEntities: JArrayList::new(),
            m_playerEntities: JArrayList::new(),
            m_weatherEffects: JArrayList::new(),
            m_cloudColor: Cell::new(16777215),
            m_skylightSubtracted: Cell::new(0),
            m_randomTickLCG: Cell::new(randomTickLCG),
            m_prevRainingStrength: Cell::new(0.0),
            m_rainingStrength: Cell::new(0.0),
            m_prevThunderingStrength: Cell::new(0.0),
            m_thunderingStrength: Cell::new(0.0),
            m_thunderCooldown: Cell::new(0),
            m_lightningCooldown: Cell::new(0),
            m_editingBlocks: Cell::new(false),
            m_lockTimestamp: Cell::new(jvm::system::currentTimeMillis()),
            m_autosavePeriod: Cell::new(40),
            m_difficultySetting: Cell::new(0),
            m_rand: rand,
            m_isNewWorld: Cell::new(false),
            m_worldProvider: Field::new(None),
            m_worldAccesses: JArrayList::new(),
            m_chunkProvider: Field::new(None),
            m_saveHandler: Field::new(None),
            m_worldInfo: Field::new(None),
            m_findingSpawnPoint: Cell::new(false),
            m_allPlayersSleeping: Cell::new(false),
            m_savedDataStorage: Field::new(None),
            m_collidingBoundingBoxes: Rc::new(JArrayList::new()),
            m_isTickingBlockEntities: Cell::new(false),
            m_lightingUpdatesCounter: Cell::new(0),
            m_spawnHostileMobs: Cell::new(true),
            m_spawnPeacefulMobs: Cell::new(true),
            m_positionsToUpdate: Field::new(JHashSet::new()),
            m_soundCounter: Cell::new(soundCounter),
            m_entitiesInArea: Rc::new(JArrayList::new()),
            m_multiplayerWorld: Cell::new(false),
        }
    }

    pub fn ext<T: 'static>(&self) -> &T { self.m_ext.downcast_ref::<T>().expect("world ext matches its class") }

    pub fn worldProvider(&self) -> Rc<WorldProvider> { self.m_worldProvider.get().orNpe() }

    pub fn worldInfo(&self) -> Rc<WorldInfo> { self.m_worldInfo.get().orNpe() }

    pub fn chunkProvider(&self) -> Rc<dyn IChunkProvider> { self.m_chunkProvider.get().orNpe() }

    pub fn saveHandler(&self) -> Rc<dyn ISaveHandler> { self.m_saveHandler.get().orNpe() }

    pub fn getWorldFile(&self) -> Option<Rc<dyn ISaveHandler>> { self.m_saveHandler.get() }

    pub fn getWorldChunkManager(&self) -> Rc<crate::worldgen::WorldChunkManager>
    {
        self.worldProvider().m_worldChunkMgr.get().orNpe()
    }

    pub fn newWithProvider(
        storage: Rc<dyn ISaveHandler>,
        name: &str,
        dimension: Rc<WorldProvider>,
        seed: i64,
    ) -> Rc<Self>
    {
        let this: Rc<Self> = Rc::new(Self::alloc(&WORLD, Box::new(())));
        Self::initWithProvider(&this, storage, name, dimension, seed);
        this
    }

    pub fn initWithProvider(
        this: &Rc<Self>,
        storage: Rc<dyn ISaveHandler>,
        name: &str,
        dimension: Rc<WorldProvider>,
        seed: i64,
    )
    {
        this.m_saveHandler.set(Some(Rc::clone(&storage)));
        this.m_worldInfo.set(Some(Rc::new(WorldInfo::withSeed(seed, name))));
        this.m_worldProvider.set(Some(Rc::clone(&dimension)));
        this.m_savedDataStorage.set(Some(Rc::new(MapStorage::new(storage))));
        dimension.registerWorld(this);
        this.m_chunkProvider.set(Some(this.getChunkProvider()));
        this.calculateInitialSkylight();
        this.initWeather();
    }

    pub fn newFromWorld(world: &Rc<World>, dimension: Rc<WorldProvider>) -> Rc<Self>
    {
        let this: Rc<Self> = Rc::new(Self::alloc(&WORLD, Box::new(())));
        this.m_lockTimestamp.set(world.m_lockTimestamp.get());
        this.m_saveHandler.set(world.m_saveHandler.get());
        this.m_worldInfo.set(Some(Rc::new(WorldInfo::withCopy(&world.worldInfo()))));
        this.m_savedDataStorage.set(Some(Rc::new(MapStorage::new(this.saveHandler()))));
        this.m_worldProvider.set(Some(Rc::clone(&dimension)));
        dimension.registerWorld(&this);
        this.m_chunkProvider.set(Some(this.getChunkProvider()));
        this.calculateInitialSkylight();
        this.initWeather();
        this
    }

    pub fn newFromSave(
        storage: Rc<dyn ISaveHandler>,
        name: &str,
        seed: i64,
        dimension: Option<Rc<WorldProvider>>,
    ) -> Rc<Self>
    {
        let this: Rc<Self> = Rc::new(Self::alloc(&WORLD, Box::new(())));
        Self::initFromSave(&this, storage, name, seed, dimension);
        this
    }

    pub fn initFromSave(
        this: &Rc<Self>,
        storage: Rc<dyn ISaveHandler>,
        name: &str,
        seed: i64,
        dimension: Option<Rc<WorldProvider>>,
    )
    {
        this.m_saveHandler.set(Some(Rc::clone(&storage)));
        this.m_savedDataStorage.set(Some(Rc::new(MapStorage::new(Rc::clone(&storage)))));
        this.m_worldInfo.set(storage.loadWorldInfo());
        this.m_isNewWorld.set(this.m_worldInfo.with(Option::is_none));
        let provider: Rc<WorldProvider> = match dimension
        {
            Some(dimension) => dimension,
            None => match this.m_worldInfo.get()
            {
                Some(info) if info.getDimension() == -1 => WorldProvider::getProviderForDimension(-1),
                _ => WorldProvider::getProviderForDimension(0),
            },
        };
        this.m_worldProvider.set(Some(Rc::clone(&provider)));

        let mut flag: bool = false;
        match this.m_worldInfo.get()
        {
            None =>
            {
                this.m_worldInfo.set(Some(Rc::new(WorldInfo::withSeed(seed, name))));
                flag = true;
            }
            Some(info) => info.setWorldName(name),
        }

        provider.registerWorld(this);
        this.m_chunkProvider.set(Some(this.getChunkProvider()));
        if flag
        {
            this.getInitialSpawnLocation();
        }
        this.calculateInitialSkylight();
        this.initWeather();
    }

    pub fn getInitialSpawnLocation(&self)
    {
        self.m_findingSpawnPoint.set(true);
        let mut i: i32 = 0;
        let b0: i8 = 64;
        let mut j: i32 = 0;
        while !self.worldProvider().canCoordinateBeSpawn(i, j)
        {
            i += self.m_rand.nextIntBounded(64) - self.m_rand.nextIntBounded(64);
            j += self.m_rand.nextIntBounded(64) - self.m_rand.nextIntBounded(64);
        }
        self.worldInfo().setSpawn(i, b0 as i32, j);
        self.m_findingSpawnPoint.set(false);
    }

    pub fn getFirstUncoveredBlock(&self, x: i32, z: i32) -> i32
    {
        let mut i: i32 = 63;
        while !self.isAirBlock(x, i + 1, z)
        {
            i += 1;
        }
        self.getBlockId(x, i, z)
    }

    pub fn emptyMethod1(&self) {}

    pub fn spawnPlayerWithLoadedChunks(self: &Rc<Self>, player: &Rc<Entity>)
    {
        let result: Result<(), jvm::throwable::Throwable> =
            jvm::throwable::catchThrowable(&jvm::throwable::EXCEPTION, || {
                let info: Rc<WorldInfo> = self.worldInfo();
                if let Some(nbttagcompound) = info.getPlayerNBTTagCompound()
                {
                    player.readFromNBT(nbttagcompound.compound());
                    info.setPlayerNBTTagCompound(None);
                }

                if let Some(provider) = self.chunkProvider().asLoadOrGenerate()
                {
                    let i: i32 = crate::math::math_helper::floor_float(player.posX() as i32 as f32) >> 4;
                    let j: i32 = crate::math::math_helper::floor_float(player.posZ() as i32 as f32) >> 4;
                    provider.setCurrentChunkOver(i, j);
                }
                self.entityJoinedWorld(player);
            });
        if let Err(exception) = result
        {
            jvm::system::printStackTrace(&exception);
        }
    }

    pub fn saveWorld(self: &Rc<Self>, saveEntities: bool, progressListener: Option<&dyn common::world::IProgressUpdate>)
    {
        let provider: Rc<dyn IChunkProvider> = self.chunkProvider();
        if !provider.canSave()
        {
            return;
        }
        if let Some(listener) = progressListener
        {
            listener.progressStartNoAbort("Saving level");
        }
        self.saveLevel();
        if let Some(listener) = progressListener
        {
            listener.displayLoadingString("Saving chunks");
        }
        provider.saveChunks(saveEntities, progressListener);
    }

    fn saveLevel(&self)
    {
        self.checkSessionLock();
        self.saveHandler().saveWorldInfoAndPlayer(&self.worldInfo(), &self.m_playerEntities.toVec());
        self.m_savedDataStorage.get().orNpe().saveAllData();
    }

    pub fn saveWhilePaused(&self, step: i32) -> bool
    {
        let provider: Rc<dyn IChunkProvider> = self.chunkProvider();
        if !provider.canSave()
        {
            return true;
        }
        if step == 0
        {
            self.saveLevel();
        }
        provider.saveChunks(false, None)
    }

    pub fn saveWorldIndirectly(self: &Rc<Self>, progressListener: Option<&dyn common::world::IProgressUpdate>)
    {
        self.saveWorld(true, progressListener);
    }

    pub fn getDebugInfo(&self) -> String { format!("All: {}", self.m_loadedEntityList.size()) }

    pub fn getChunkSourceDebugInfo(&self) -> String { self.chunkProvider().makeString() }

    pub fn checkSessionLock(&self) { self.saveHandler().checkSessionLock(); }

    pub fn setWorldTime(&self, time: i64) { self.worldInfo().setWorldTime(time); }

    pub fn getRandomSeed(&self) -> i64 { self.worldInfo().getRandomSeed() }

    pub fn getWorldTime(&self) -> i64 { self.worldInfo().getWorldTime() }

    pub fn getSpawnPoint(&self) -> crate::world::ChunkCoordinates
    {
        let info: Rc<WorldInfo> = self.worldInfo();
        crate::world::ChunkCoordinates::new(info.getSpawnX(), info.getSpawnY(), info.getSpawnZ())
    }

    pub fn setSpawnPoint(&self, pos: &crate::world::ChunkCoordinates)
    {
        self.worldInfo().setSpawn(pos.m_x, pos.m_y, pos.m_z);
    }

    pub fn getIChunkProvider(&self) -> Rc<dyn IChunkProvider> { self.chunkProvider() }

    pub fn getWorldInfo(&self) -> Rc<WorldInfo> { self.worldInfo() }

    pub fn setItemData(&self, id: &str, data: Rc<crate::world::MapDataBase>)
    {
        self.m_savedDataStorage.get().orNpe().setData(id, data);
    }

    pub fn loadItemData(&self, kind: crate::world::MapDataKind, id: &str) -> Option<Rc<crate::world::MapDataBase>>
    {
        self.m_savedDataStorage.get().orNpe().loadData(kind, id)
    }

    pub fn getUniqueDataId(&self, id: &str) -> i32 { self.m_savedDataStorage.get().orNpe().getUniqueDataId(id) }

    pub fn addWorldAccess(&self, listener: Rc<dyn IWorldAccess>) { self.m_worldAccesses.add(listener); }

    pub fn removeWorldAccess(&self, listener: &Rc<dyn IWorldAccess>) { self.m_worldAccesses.remove(listener); }

    pub fn setAllowedMobSpawns(&self, spawnAnimals: bool, spawnMonsters: bool)
    {
        self.m_spawnHostileMobs.set(spawnAnimals);
        self.m_spawnPeacefulMobs.set(spawnMonsters);
    }

    pub fn prepare(&self)
    {
        while self.chunkProvider().unload100OldestChunks()
        {}
    }
}

pub fn canModify(_this: &Rc<World>, _player: &Rc<Entity>, _x: i32, _y: i32, _z: i32) -> bool { true }

pub fn doEntityEvent(_this: &Rc<World>, _entity: &Rc<Entity>, _event: i8) {}
