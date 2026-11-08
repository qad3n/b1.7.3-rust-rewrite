use crate::player::{PlayerHash, entity_player_mp};
use crate::server::MinecraftServer;
use crate::world::world_server;
use common::world::ChunkCoordIntPair;
use game::block::Block;
use game::entity::Entity;
use game::network::packet::Packet;
use game::network::packets::{Packet50PreChunk, Packet51MapChunk, Packet52MultiBlockChange, Packet53BlockChange};
use game::tileentity::TileEntity;
use game::world::World;
use jvm::array_list::JArrayList;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

pub struct PlayerManager
{
    m_self: Weak<PlayerManager>,
    pub m_players: JArrayList<Rc<Entity>>,
    m_playerInstances: PlayerHash<Rc<PlayerInstance>>,
    m_playerInstancesToUpdate: JArrayList<Rc<PlayerInstance>>,
    m_mcServer: Rc<MinecraftServer>,
    m_dimension: i32,
    m_playerViewRadius: i32,
    m_adjacentChunkCoords: [[i32; 2]; 4],
}

fn instanceKey(chunkX: i32, chunkZ: i32) -> i64 { (chunkX as i64 + 2147483647) | (chunkZ as i64 + 2147483647) << 32 }

impl PlayerManager
{
    pub fn new(server: &Rc<MinecraftServer>, dimension: i32, chunkViewDistance: i32) -> Rc<PlayerManager>
    {
        let players: JArrayList<Rc<Entity>> = JArrayList::new();
        let playerInstances: PlayerHash<Rc<PlayerInstance>> = PlayerHash::new();
        let playerInstancesToUpdate: JArrayList<Rc<PlayerInstance>> = JArrayList::new();
        let adjacentChunkCoords: [[i32; 2]; 4] = [[1, 0], [0, 1], [-1, 0], [0, -1]];
        if chunkViewDistance > 15
        {
            jvm::throwable::throw(&jvm::throwable::ILLEGAL_ARGUMENT, "Too big view radius!");
        }
        if chunkViewDistance < 3
        {
            jvm::throwable::throw(&jvm::throwable::ILLEGAL_ARGUMENT, "Too small view radius!");
        }
        Rc::new_cyclic(|this| PlayerManager {
            m_self: this.clone(),
            m_players: players,
            m_playerInstances: playerInstances,
            m_playerInstancesToUpdate: playerInstancesToUpdate,
            m_adjacentChunkCoords: adjacentChunkCoords,
            m_playerViewRadius: chunkViewDistance,
            m_mcServer: Rc::clone(server),
            m_dimension: dimension,
        })
    }

    pub fn getMinecraftServer(&self) -> Rc<World> { self.m_mcServer.getWorldManager(self.m_dimension) }

    pub fn updatePlayerInstances(&self)
    {
        let mut i: i32 = 0;
        while i < self.m_playerInstancesToUpdate.size()
        {
            self.m_playerInstancesToUpdate.get(i).onUpdate();
            i += 1;
        }
        self.m_playerInstancesToUpdate.clear();
    }

    fn getPlayerInstance(&self, chunkX: i32, chunkZ: i32, add: bool) -> Option<Rc<PlayerInstance>>
    {
        let i: i64 = instanceKey(chunkX, chunkZ);
        let mut playermanager_playerinstance: Option<Rc<PlayerInstance>> = self.m_playerInstances.getValueByKey(i);
        if playermanager_playerinstance.is_none() && add
        {
            let instance: Rc<PlayerInstance> = PlayerInstance::new(&self.m_self, chunkX, chunkZ);
            self.m_playerInstances.add(i, Rc::clone(&instance));
            playermanager_playerinstance = Some(instance);
        }
        playermanager_playerinstance
    }

    fn addToInstance(&self, chunkX: i32, chunkZ: i32, player: &Rc<Entity>)
    {
        if let Some(instance) = self.getPlayerInstance(chunkX, chunkZ, true)
        {
            instance.addPlayer(player);
        }
    }

    pub fn markBlockNeedsUpdate(&self, x: i32, y: i32, z: i32)
    {
        let i: i32 = x >> 4;
        let j: i32 = z >> 4;
        if let Some(playermanager_playerinstance) = self.getPlayerInstance(i, j, false)
        {
            playermanager_playerinstance.markBlockNeedsUpdate(x & 15, y, z & 15);
        }
    }

    pub fn addPlayer(&self, player: &Rc<Entity>)
    {
        let i: i32 = player.m_posX.get() as i32 >> 4;
        let j: i32 = player.m_posZ.get() as i32 >> 4;
        let ext: &entity_player_mp::PlayerMPExt = entity_player_mp::fields(player);
        ext.m_trackedX.set(player.m_posX.get());
        ext.m_trackedZ.set(player.m_posZ.get());
        let mut k: i32 = 0;
        let l: i32 = self.m_playerViewRadius;
        let mut i1: i32 = 0;
        let mut j1: i32 = 0;
        self.addToInstance(i, j, player);
        for k1 in 1..=l * 2
        {
            for _ in 0..2
            {
                let aint: [i32; 2] = self.m_adjacentChunkCoords[(k % 4) as usize];
                k += 1;
                for _ in 0..k1
                {
                    i1 += aint[0];
                    j1 += aint[1];
                    self.addToInstance(i + i1, j + j1, player);
                }
            }
        }
        k %= 4;
        for _ in 0..l * 2
        {
            i1 += self.m_adjacentChunkCoords[k as usize][0];
            j1 += self.m_adjacentChunkCoords[k as usize][1];
            self.addToInstance(i + i1, j + j1, player);
        }
        self.m_players.add(Rc::clone(player));
    }

    pub fn removePlayer(&self, player: &Rc<Entity>)
    {
        let ext: &entity_player_mp::PlayerMPExt = entity_player_mp::fields(player);
        let i: i32 = ext.m_trackedX.get() as i32 >> 4;
        let j: i32 = ext.m_trackedZ.get() as i32 >> 4;
        for k in i - self.m_playerViewRadius..=i + self.m_playerViewRadius
        {
            for l in j - self.m_playerViewRadius..=j + self.m_playerViewRadius
            {
                if let Some(playermanager_playerinstance) = self.getPlayerInstance(k, l, false)
                {
                    playermanager_playerinstance.removePlayer(player);
                }
            }
        }
        self.m_players.remove(player);
    }

    fn isChunkWithinView(&self, chunkX: i32, chunkZ: i32, playerChunkX: i32, playerChunkZ: i32) -> bool
    {
        let i: i32 = chunkX - playerChunkX;
        let j: i32 = chunkZ - playerChunkZ;
        if i < -self.m_playerViewRadius || i > self.m_playerViewRadius
        {
            false
        }
        else
        {
            j >= -self.m_playerViewRadius && j <= self.m_playerViewRadius
        }
    }

    pub fn movePlayer(&self, player: &Rc<Entity>)
    {
        let ext: &entity_player_mp::PlayerMPExt = entity_player_mp::fields(player);
        let i: i32 = player.m_posX.get() as i32 >> 4;
        let j: i32 = player.m_posZ.get() as i32 >> 4;
        let d0: f64 = ext.m_trackedX.get() - player.m_posX.get();
        let d1: f64 = ext.m_trackedZ.get() - player.m_posZ.get();
        let d2: f64 = d0 * d0 + d1 * d1;
        if !(d2 < 64.0)
        {
            let k: i32 = ext.m_trackedX.get() as i32 >> 4;
            let l: i32 = ext.m_trackedZ.get() as i32 >> 4;
            let i1: i32 = i - k;
            let j1: i32 = j - l;
            if i1 != 0 || j1 != 0
            {
                for k1 in i - self.m_playerViewRadius..=i + self.m_playerViewRadius
                {
                    for l1 in j - self.m_playerViewRadius..=j + self.m_playerViewRadius
                    {
                        if !self.isChunkWithinView(k1, l1, k, l)
                        {
                            self.addToInstance(k1, l1, player);
                        }
                        if !self.isChunkWithinView(k1 - i1, l1 - j1, i, j)
                            && let Some(playermanager_playerinstance) = self.getPlayerInstance(k1 - i1, l1 - j1, false)
                        {
                            playermanager_playerinstance.removePlayer(player);
                        }
                    }
                }
                ext.m_trackedX.set(player.m_posX.get());
                ext.m_trackedZ.set(player.m_posZ.get());
            }
        }
    }

    pub fn getMaxTrackingDistance(&self) -> i32 { self.m_playerViewRadius * 16 - 16 }
}

pub struct PlayerInstance
{
    m_outer: Weak<PlayerManager>,
    m_players: JArrayList<Rc<Entity>>,
    m_chunkX: i32,
    m_chunkZ: i32,
    m_currentChunk: ChunkCoordIntPair,
    m_blocksToUpdate: RefCell<[i16; 10]>,
    m_numBlocksToUpdate: Cell<i32>,
    m_minX: Cell<i32>,
    m_maxX: Cell<i32>,
    m_minY: Cell<i32>,
    m_maxY: Cell<i32>,
    m_minZ: Cell<i32>,
    m_maxZ: Cell<i32>,
}

impl PlayerInstance
{
    fn new(outer: &Weak<PlayerManager>, chunkX: i32, chunkZ: i32) -> Rc<PlayerInstance>
    {
        let instance: Rc<PlayerInstance> = Rc::new(PlayerInstance {
            m_outer: outer.clone(),
            m_players: JArrayList::new(),
            m_blocksToUpdate: RefCell::new([0; 10]),
            m_numBlocksToUpdate: Cell::new(0),
            m_minX: Cell::new(0),
            m_maxX: Cell::new(0),
            m_minY: Cell::new(0),
            m_maxY: Cell::new(0),
            m_minZ: Cell::new(0),
            m_maxZ: Cell::new(0),
            m_chunkX: chunkX,
            m_chunkZ: chunkZ,
            m_currentChunk: ChunkCoordIntPair::new(chunkX, chunkZ),
        });
        world_server::chunkProviderServer(&instance.outer().getMinecraftServer()).loadChunk(chunkX, chunkZ);
        instance
    }

    fn outer(&self) -> Rc<PlayerManager> { self.m_outer.upgrade().expect("player instances live inside their manager") }

    pub fn addPlayer(&self, player: &Rc<Entity>)
    {
        if self.m_players.contains(player)
        {
            jvm::throwable::throw(
                &jvm::throwable::ILLEGAL_STATE,
                &format!(
                    "Failed to add player. net.minecraft.src.{}@{:x} already is in chunk {}, {}",
                    player.className(),
                    player.m_entityId.get(),
                    self.m_chunkX,
                    self.m_chunkZ
                ),
            );
        }
        let ext: &entity_player_mp::PlayerMPExt = entity_player_mp::fields(player);
        ext.m_allLoadedChunks.update(|set| set.add(self.m_currentChunk));
        entity_player_mp::netServerHandler(player).sendPacket(Box::new(Packet50PreChunk::new(
            self.m_currentChunk.m_chunkXPos,
            self.m_currentChunk.m_chunkZPos,
            true,
        )));
        self.m_players.add(Rc::clone(player));
        ext.m_loadedChunks.add(self.m_currentChunk);
    }

    pub fn removePlayer(self: &Rc<Self>, player: &Rc<Entity>)
    {
        if !self.m_players.contains(player)
        {
            return;
        }
        self.m_players.remove(player);
        if self.m_players.size() == 0
        {
            let l: i64 = (self.m_chunkX as i64 + i32::MAX as i64) | (self.m_chunkZ as i64 + i32::MAX as i64) << 32;
            let outer: Rc<PlayerManager> = self.outer();
            outer.m_playerInstances.remove(l);
            if self.m_numBlocksToUpdate.get() > 0
            {
                outer.m_playerInstancesToUpdate.removeIdentity(self);
            }
            world_server::chunkProviderServer(&outer.getMinecraftServer()).unloadChunk(self.m_chunkX, self.m_chunkZ);
        }
        let ext: &entity_player_mp::PlayerMPExt = entity_player_mp::fields(player);
        ext.m_loadedChunks.remove(&self.m_currentChunk);
        if ext.m_allLoadedChunks.with(|set| set.contains(&self.m_currentChunk))
        {
            entity_player_mp::netServerHandler(player).sendPacket(Box::new(Packet50PreChunk::new(
                self.m_chunkX,
                self.m_chunkZ,
                false,
            )));
        }
    }

    pub fn markBlockNeedsUpdate(self: &Rc<Self>, localX: i32, y: i32, localZ: i32)
    {
        if self.m_numBlocksToUpdate.get() == 0
        {
            self.outer().m_playerInstancesToUpdate.add(Rc::clone(self));
            self.m_minX.set(localX);
            self.m_maxX.set(localX);
            self.m_minY.set(y);
            self.m_maxY.set(y);
            self.m_minZ.set(localZ);
            self.m_maxZ.set(localZ);
        }
        if self.m_minX.get() > localX
        {
            self.m_minX.set(localX);
        }
        if self.m_maxX.get() < localX
        {
            self.m_maxX.set(localX);
        }
        if self.m_minY.get() > y
        {
            self.m_minY.set(y);
        }
        if self.m_maxY.get() < y
        {
            self.m_maxY.set(y);
        }
        if self.m_minZ.get() > localZ
        {
            self.m_minZ.set(localZ);
        }
        if self.m_maxZ.get() < localZ
        {
            self.m_maxZ.set(localZ);
        }
        if self.m_numBlocksToUpdate.get() < 10
        {
            let short1: i16 = (localX << 12 | localZ << 8 | y) as i16;
            for i in 0..self.m_numBlocksToUpdate.get()
            {
                if self.m_blocksToUpdate.borrow()[i as usize] == short1
                {
                    return;
                }
            }
            let index: i32 = self.m_numBlocksToUpdate.get();
            self.m_numBlocksToUpdate.set(index + 1);
            self.m_blocksToUpdate.borrow_mut()[index as usize] = short1;
        }
    }

    pub fn sendPacketToPlayersInInstance(&self, packet: Box<dyn Packet>)
    {
        let mut i: i32 = 0;
        while i < self.m_players.size()
        {
            let entityplayermp: Rc<Entity> = self.m_players.get(i);
            if entity_player_mp::fields(&entityplayermp)
                .m_allLoadedChunks
                .with(|set| set.contains(&self.m_currentChunk))
            {
                entity_player_mp::netServerHandler(&entityplayermp).sendPacket(packet.clonePacket());
            }
            i += 1;
        }
    }

    pub fn onUpdate(&self)
    {
        let minecraftServer: Rc<World> = self.outer().getMinecraftServer();
        if self.m_numBlocksToUpdate.get() == 0
        {
            return;
        }
        if self.m_numBlocksToUpdate.get() == 1
        {
            let x: i32 = self.m_chunkX * 16 + self.m_minX.get();
            let minY: i32 = self.m_minY.get();
            let z: i32 = self.m_chunkZ * 16 + self.m_minZ.get();
            self.sendPacketToPlayersInInstance(Box::new(Packet53BlockChange::new(x, minY, z, &minecraftServer)));
            if Block::isBlockContainer(minecraftServer.getBlockId(x, minY, z))
            {
                self.updateTileEntity(minecraftServer.getBlockTileEntity(x, minY, z).as_ref());
            }
        }
        else if self.m_numBlocksToUpdate.get() == 10
        {
            self.m_minY.set(self.m_minY.get() / 2 * 2);
            self.m_maxY.set((self.m_maxY.get() / 2 + 1) * 2);
            let n: i32 = self.m_minX.get() + self.m_chunkX * 16;
            let minY2: i32 = self.m_minY.get();
            let n2: i32 = self.m_minZ.get() + self.m_chunkZ * 16;
            let widthX: i32 = self.m_maxX.get() - self.m_minX.get() + 1;
            let height: i32 = self.m_maxY.get() - self.m_minY.get() + 2;
            let widthZ: i32 = self.m_maxZ.get() - self.m_minZ.get() + 1;
            self.sendPacketToPlayersInInstance(Box::new(Packet51MapChunk::new(
                n,
                minY2,
                n2,
                widthX,
                height,
                widthZ,
                &minecraftServer,
            )));
            let tileEntityList: Vec<Rc<TileEntity>> = world_server::getTileEntityList(
                &minecraftServer,
                n,
                minY2,
                n2,
                n + widthX,
                minY2 + height,
                n2 + widthZ,
            );
            for tileentity in &tileEntityList
            {
                self.updateTileEntity(Some(tileentity));
            }
        }
        else
        {
            let blocksToUpdate: [i16; 10] = *self.m_blocksToUpdate.borrow();
            self.sendPacketToPlayersInInstance(Box::new(Packet52MultiBlockChange::new(
                self.m_chunkX,
                self.m_chunkZ,
                &blocksToUpdate,
                self.m_numBlocksToUpdate.get(),
                &minecraftServer,
            )));
            for _ in 0..self.m_numBlocksToUpdate.get()
            {
                let numBlocksToUpdate: i32 = self.m_numBlocksToUpdate.get();
                let n3: i32 = self.m_chunkX * 16 + (numBlocksToUpdate >> 12 & 0xF);
                let n4: i32 = numBlocksToUpdate & 0xFF;
                let n5: i32 = self.m_chunkZ * 16 + (numBlocksToUpdate >> 8 & 0xF);
                if Block::isBlockContainer(minecraftServer.getBlockId(n3, n4, n5))
                {
                    jvm::system::println("Sending!");
                    self.updateTileEntity(minecraftServer.getBlockTileEntity(n3, n4, n5).as_ref());
                }
            }
        }
        self.m_numBlocksToUpdate.set(0);
    }

    fn updateTileEntity(&self, blockEntity: Option<&Rc<TileEntity>>)
    {
        if let Some(blockEntity) = blockEntity
        {
            let packet: Option<Box<dyn Packet>> = blockEntity.getDescriptionPacket();
            if let Some(packet) = packet
            {
                self.sendPacketToPlayersInInstance(packet);
            }
        }
    }
}
