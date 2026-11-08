use crate::entity::Entity;
use common::nbt::{NBTBase, NBTTagCompound};
use jvm::field::Field;
use jvm::string::JString;
use std::cell::Cell;
use std::rc::Rc;

#[derive(Default)]
pub struct WorldInfo
{
    m_randomSeed: Cell<i64>,
    m_spawnX: Cell<i32>,
    m_spawnY: Cell<i32>,
    m_spawnZ: Cell<i32>,
    m_worldTime: Cell<i64>,
    m_lastTimePlayed: Cell<i64>,
    m_sizeOnDisk: Cell<i64>,
    m_playerTag: Field<Option<Rc<NBTBase>>>,
    m_dimension: Cell<i32>,
    m_levelName: Field<Option<JString>>,
    m_saveVersion: Cell<i32>,
    m_raining: Cell<bool>,
    m_rainTime: Cell<i32>,
    m_thundering: Cell<bool>,
    m_thunderTime: Cell<i32>,
}

impl WorldInfo
{
    pub fn fromNBT(nbt: &NBTTagCompound) -> Self
    {
        let info: Self = Self::default();
        info.m_randomSeed.set(nbt.getLong("RandomSeed"));
        info.m_spawnX.set(nbt.getInteger("SpawnX"));
        info.m_spawnY.set(nbt.getInteger("SpawnY"));
        info.m_spawnZ.set(nbt.getInteger("SpawnZ"));
        info.m_worldTime.set(nbt.getLong("Time"));
        info.m_lastTimePlayed.set(nbt.getLong("LastPlayed"));
        info.m_sizeOnDisk.set(nbt.getLong("SizeOnDisk"));
        info.m_levelName.set(nbt.getString("LevelName"));
        info.m_saveVersion.set(nbt.getInteger("version"));
        info.m_rainTime.set(nbt.getInteger("rainTime"));
        info.m_raining.set(nbt.getBoolean("raining"));
        info.m_thunderTime.set(nbt.getInteger("thunderTime"));
        info.m_thundering.set(nbt.getBoolean("thundering"));
        if nbt.hasKey("Player")
        {
            let player: Rc<NBTBase> = nbt.getCompoundTag("Player");
            info.m_dimension.set(player.compound().getInteger("Dimension"));
            info.m_playerTag.set(Some(player));
        }
        info
    }

    pub fn withSeed(seed: i64, worldName: &str) -> Self
    {
        let info: Self = Self::default();
        info.m_randomSeed.set(seed);
        info.m_levelName.set(Some(JString::from(worldName)));
        info
    }

    pub fn withCopy(data: &WorldInfo) -> Self
    {
        let info: Self = Self::default();
        info.m_randomSeed.set(data.m_randomSeed.get());
        info.m_spawnX.set(data.m_spawnX.get());
        info.m_spawnY.set(data.m_spawnY.get());
        info.m_spawnZ.set(data.m_spawnZ.get());
        info.m_worldTime.set(data.m_worldTime.get());
        info.m_lastTimePlayed.set(data.m_lastTimePlayed.get());
        info.m_sizeOnDisk.set(data.m_sizeOnDisk.get());
        info.m_playerTag.set(data.m_playerTag.get());
        info.m_dimension.set(data.m_dimension.get());
        info.m_levelName.set(data.m_levelName.get());
        info.m_saveVersion.set(data.m_saveVersion.get());
        info.m_rainTime.set(data.m_rainTime.get());
        info.m_raining.set(data.m_raining.get());
        info.m_thunderTime.set(data.m_thunderTime.get());
        info.m_thundering.set(data.m_thundering.get());
        info
    }

    pub fn getNBTTagCompound(&self) -> Rc<NBTBase>
    {
        let nbt: Rc<NBTBase> = NBTTagCompound::create();
        self.updateTagCompound(nbt.compound(), self.m_playerTag.get());
        nbt
    }

    pub fn getNBTTagCompoundWithPlayer(&self, players: &[Rc<Entity>]) -> Rc<NBTBase>
    {
        let nbt: Rc<NBTBase> = NBTTagCompound::create();
        let entityPlayer: Option<&Rc<Entity>> = players.first();
        let mut playerData: Option<Rc<NBTBase>> = None;
        if let Some(player) = entityPlayer
        {
            let data: Rc<NBTBase> = NBTTagCompound::create();
            player.writeToNBT(data.compound());
            playerData = Some(data);
        }
        self.updateTagCompound(nbt.compound(), playerData);
        nbt
    }

    fn updateTagCompound(&self, nbt: &NBTTagCompound, playerData: Option<Rc<NBTBase>>)
    {
        nbt.setLong("RandomSeed", self.m_randomSeed.get());
        nbt.setInteger("SpawnX", self.m_spawnX.get());
        nbt.setInteger("SpawnY", self.m_spawnY.get());
        nbt.setInteger("SpawnZ", self.m_spawnZ.get());
        nbt.setLong("Time", self.m_worldTime.get());
        nbt.setLong("SizeOnDisk", self.m_sizeOnDisk.get());
        nbt.setLong("LastPlayed", jvm::system::currentTimeMillis());
        nbt.setString("LevelName", self.m_levelName.get());
        nbt.setInteger("version", self.m_saveVersion.get());
        nbt.setInteger("rainTime", self.m_rainTime.get());
        nbt.setBoolean("raining", self.m_raining.get());
        nbt.setInteger("thunderTime", self.m_thunderTime.get());
        nbt.setBoolean("thundering", self.m_thundering.get());
        if let Some(playerData) = playerData
        {
            nbt.setCompoundTag("Player", &playerData);
        }
    }

    pub fn getRandomSeed(&self) -> i64 { self.m_randomSeed.get() }

    pub fn getSpawnX(&self) -> i32 { self.m_spawnX.get() }

    pub fn getSpawnY(&self) -> i32 { self.m_spawnY.get() }

    pub fn getSpawnZ(&self) -> i32 { self.m_spawnZ.get() }

    pub fn getWorldTime(&self) -> i64 { self.m_worldTime.get() }

    pub fn getSizeOnDisk(&self) -> i64 { self.m_sizeOnDisk.get() }

    pub fn getPlayerNBTTagCompound(&self) -> Option<Rc<NBTBase>> { self.m_playerTag.get() }

    pub fn getDimension(&self) -> i32 { self.m_dimension.get() }

    pub fn setSpawnX(&self, x: i32) { self.m_spawnX.set(x); }

    pub fn setSpawnY(&self, y: i32) { self.m_spawnY.set(y); }

    pub fn setSpawnZ(&self, z: i32) { self.m_spawnZ.set(z); }

    pub fn setWorldTime(&self, time: i64) { self.m_worldTime.set(time); }

    pub fn setSizeOnDisk(&self, size: i64) { self.m_sizeOnDisk.set(size); }

    pub fn setPlayerNBTTagCompound(&self, playerData: Option<Rc<NBTBase>>) { self.m_playerTag.set(playerData); }

    pub fn setSpawn(&self, x: i32, y: i32, z: i32)
    {
        self.m_spawnX.set(x);
        self.m_spawnY.set(y);
        self.m_spawnZ.set(z);
    }

    pub fn getWorldName(&self) -> Option<JString> { self.m_levelName.get() }

    pub fn setWorldName(&self, name: &str) { self.m_levelName.set(Some(JString::from(name))); }

    pub fn getSaveVersion(&self) -> i32 { self.m_saveVersion.get() }

    pub fn setSaveVersion(&self, version: i32) { self.m_saveVersion.set(version); }

    pub fn getLastTimePlayed(&self) -> i64 { self.m_lastTimePlayed.get() }

    pub fn getThundering(&self) -> bool { self.m_thundering.get() }

    pub fn setThundering(&self, thundering: bool) { self.m_thundering.set(thundering); }

    pub fn getThunderTime(&self) -> i32 { self.m_thunderTime.get() }

    pub fn setThunderTime(&self, thunderTime: i32) { self.m_thunderTime.set(thunderTime); }

    pub fn getRaining(&self) -> bool { self.m_raining.get() }

    pub fn setRaining(&self, raining: bool) { self.m_raining.set(raining); }

    pub fn getRainTime(&self) -> i32 { self.m_rainTime.get() }

    pub fn setRainTime(&self, rainTime: i32) { self.m_rainTime.set(rainTime); }
}
