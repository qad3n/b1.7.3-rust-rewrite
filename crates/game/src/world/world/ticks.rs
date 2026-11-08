use crate::block::Block;
use crate::entity::Entity;
use crate::math::math_helper::floor_double;
use crate::world::{Chunk, EnumSkyBlock, World};
use common::world::{ChunkCoordIntPair, NextTickListEntry};
use jvm::hash_map::HashCursor;
use jvm::random::Random;
use jvm::throwable::OrNpe;
use std::rc::Rc;

pub fn scheduleBlockUpdate(this: &Rc<World>, x: i32, y: i32, z: i32, block: i32, delay: i32)
{
    let mut nextticklistentry: NextTickListEntry = NextTickListEntry::new(x, y, z, block);
    let b0: i32 = 8;
    if this.m_scheduledUpdatesAreImmediate.get()
    {
        let entry: &NextTickListEntry = &nextticklistentry;
        if this.checkChunksExist(
            entry.m_xCoord - b0,
            entry.m_yCoord - b0,
            entry.m_zCoord - b0,
            entry.m_xCoord + b0,
            entry.m_yCoord + b0,
            entry.m_zCoord + b0,
        )
        {
            let i: i32 = this.getBlockId(entry.m_xCoord, entry.m_yCoord, entry.m_zCoord);
            if i == entry.m_blockID && i > 0
            {
                Block::byId(i).orNpe().updateTick(this, entry.m_xCoord, entry.m_yCoord, entry.m_zCoord, &this.m_rand);
            }
        }
        return;
    }

    if this.checkChunksExist(x - b0, y - b0, z - b0, x + b0, y + b0, z + b0)
    {
        if block > 0
        {
            nextticklistentry = nextticklistentry.setScheduledTime(delay as i64 + this.worldInfo().getWorldTime());
        }
        if !this.m_scheduledTickSet.with(|set| set.contains(&nextticklistentry))
        {
            this.m_scheduledTickSet.update(|set| set.add(nextticklistentry.clone()));
            this.m_scheduledTickTreeSet.update(|set| set.insert(nextticklistentry));
        }
    }
}

pub fn TickUpdates(this: &Rc<World>, flush: bool) -> bool
{
    let mut i: i32 = this.m_scheduledTickTreeSet.with(|set| set.len()) as i32;
    if i != this.m_scheduledTickSet.with(|set| set.size())
    {
        jvm::throwable::throw(&jvm::throwable::ILLEGAL_STATE, "TickNextTick list out of synch");
    }
    if i > 1000
    {
        i = 1000;
    }

    for _ in 0..i
    {
        let nextticklistentry: NextTickListEntry = match this.m_scheduledTickTreeSet.with(|set| set.first().cloned())
        {
            Some(entry) => entry,
            None => jvm::throwable::throwBare(&jvm::throwable::NO_SUCH_ELEMENT),
        };
        if !flush && nextticklistentry.m_scheduledTime > this.worldInfo().getWorldTime()
        {
            break;
        }

        this.m_scheduledTickTreeSet.update(|set| set.remove(&nextticklistentry));
        this.m_scheduledTickSet.update(|set| set.remove(&nextticklistentry));
        let b0: i32 = 8;
        let entry: &NextTickListEntry = &nextticklistentry;
        if this.checkChunksExist(
            entry.m_xCoord - b0,
            entry.m_yCoord - b0,
            entry.m_zCoord - b0,
            entry.m_xCoord + b0,
            entry.m_yCoord + b0,
            entry.m_zCoord + b0,
        )
        {
            let k: i32 = this.getBlockId(entry.m_xCoord, entry.m_yCoord, entry.m_zCoord);
            if k == entry.m_blockID && k > 0
            {
                Block::byId(k).orNpe().updateTick(this, entry.m_xCoord, entry.m_yCoord, entry.m_zCoord, &this.m_rand);
            }
        }
    }
    this.m_scheduledTickTreeSet.with(|set| !set.is_empty())
}

fn nextLcg(this: &World) -> i32
{
    this.m_randomTickLCG.set(this.m_randomTickLCG.get().wrapping_mul(3).wrapping_add(1013904223));
    this.m_randomTickLCG.get()
}

pub fn updateBlocksAndPlayCaveSounds(this: &Rc<World>)
{
    this.m_positionsToUpdate.update(|set| set.clear());
    let mut i: i32 = 0;
    while i < this.m_playerEntities.size()
    {
        let entityplayer: Rc<Entity> = this.m_playerEntities.get(i);
        let j: i32 = floor_double(entityplayer.posX() / 16.0);
        let k: i32 = floor_double(entityplayer.posZ() / 16.0);
        let b0: i32 = 9;
        for l in -b0..=b0
        {
            for i1 in -b0..=b0
            {
                this.m_positionsToUpdate.update(|set| set.add(ChunkCoordIntPair::new(l + j, i1 + k)));
            }
        }
        i += 1;
    }

    if this.m_soundCounter.get() > 0
    {
        this.m_soundCounter.set(this.m_soundCounter.get() - 1);
    }

    let mut cursor: HashCursor = this.m_positionsToUpdate.with(|set| set.cursor());
    while cursor.hasNext()
    {
        let chunkcoordintpair: ChunkCoordIntPair = this.m_positionsToUpdate.with(|set| set.nextKey(&mut cursor));
        let i2: i32 = chunkcoordintpair.m_chunkXPos * 16;
        let j2: i32 = chunkcoordintpair.m_chunkZPos * 16;
        let chunk: Rc<Chunk> =
            this.getChunkFromChunkCoords(chunkcoordintpair.m_chunkXPos, chunkcoordintpair.m_chunkZPos);
        if this.m_soundCounter.get() == 0
        {
            let k2: i32 = nextLcg(this) >> 2;
            let mut k3: i32 = k2 & 15;
            let mut j1: i32 = k2 >> 8 & 15;
            let k1: i32 = k2 >> 16 & 127;
            let l1: i32 = chunk.getBlockID(k3, k1, j1);
            k3 += i2;
            j1 += j2;
            if l1 == 0
                && this.getFullBlockLightValue(k3, k1, j1) <= this.m_rand.nextIntBounded(8)
                && this.getSavedLightValue(EnumSkyBlock::Sky, k3, k1, j1) <= 0
            {
                let (x, y, z): (f64, f64, f64) = (k3 as f64 + 0.5, k1 as f64 + 0.5, j1 as f64 + 0.5);
                if let Some(player) = this.getClosestPlayer(x, y, z, 8.0)
                    && player.getDistanceSq(x, y, z) > 4.0
                {
                    let pitch: f32 = 0.8 + this.m_rand.nextFloat() * 0.2;
                    this.playSoundEffect(x, y, z, "ambient.cave.cave", 0.7, pitch);
                    this.m_soundCounter.set(this.m_rand.nextIntBounded(12000) + 6000);
                }
            }
        }

        if this.m_rand.nextIntBounded(100000) == 0 && this.isRaining() && this.isThundering()
        {
            let l2: i32 = nextLcg(this) >> 2;
            let l3: i32 = i2 + (l2 & 15);
            let k4: i32 = j2 + (l2 >> 8 & 15);
            let j5: i32 = this.findTopSolidBlock(l3, k4);
            if this.canBlockBeRainedOn(l3, j5, k4)
            {
                this.addWeatherEffect(crate::entity::entity_lightning_bolt::create(
                    this, l3 as f64, j5 as f64, k4 as f64,
                ));
                this.m_thunderCooldown.set(2);
            }
        }

        if this.m_rand.nextIntBounded(16) == 0
        {
            let i3: i32 = nextLcg(this) >> 2;
            let i4: i32 = i3 & 15;
            let l4: i32 = i3 >> 8 & 15;
            let k5: i32 = this.findTopSolidBlock(i4 + i2, l4 + j2);
            if this.getWorldChunkManager().getBiomeGenAt(i4 + i2, l4 + j2).getEnableSnow()
                && k5 >= 0
                && k5 < 128
                && chunk.getSavedLightValue(EnumSkyBlock::Block, i4, k5, l4) < 10
            {
                let i6: i32 = chunk.getBlockID(i4, k5 - 1, l4);
                let k6: i32 = chunk.getBlockID(i4, k5, l4);
                if this.isRaining()
                    && k6 == 0
                    && Block::snow().canPlaceBlockAt(this, i4 + i2, k5, l4 + j2)
                    && i6 != 0
                    && i6 != Block::ice().m_blockID
                    && Block::byId(i6).orNpe().m_blockMaterial.getIsSolid()
                {
                    this.setBlockWithNotify(i4 + i2, k5, l4 + j2, Block::snow().m_blockID);
                }
                if i6 == Block::waterStill().m_blockID && chunk.getBlockMetadata(i4, k5 - 1, l4) == 0
                {
                    this.setBlockWithNotify(i4 + i2, k5 - 1, l4 + j2, Block::ice().m_blockID);
                }
            }
        }

        for _ in 0..80
        {
            let j4: i32 = nextLcg(this) >> 2;
            let i5: i32 = j4 & 15;
            let l5: i32 = j4 >> 8 & 15;
            let j6: i32 = j4 >> 16 & 127;
            let l6: i32 = chunk.rawBlock(i5 << 11 | l5 << 7 | j6) as i32 & 255;
            if Block::tickOnLoad(l6)
            {
                Block::byId(l6).orNpe().updateTick(this, i5 + i2, j6, l5 + j2, &this.m_rand);
            }
        }
    }
}

impl World
{
    pub fn randomDisplayUpdates(self: &Rc<Self>, x: i32, y: i32, z: i32)
    {
        let b0: i32 = 16;
        let random: Random = Random::new();
        for _ in 0..1000
        {
            let j: i32 = x + self.m_rand.nextIntBounded(b0) - self.m_rand.nextIntBounded(b0);
            let k: i32 = y + self.m_rand.nextIntBounded(b0) - self.m_rand.nextIntBounded(b0);
            let l: i32 = z + self.m_rand.nextIntBounded(b0) - self.m_rand.nextIntBounded(b0);
            let i1: i32 = self.getBlockId(j, k, l);
            if i1 > 0
            {
                Block::byId(i1).orNpe().randomDisplayTick(self, j, k, l, &random);
            }
        }
    }
}

impl World
{
    pub fn changeTime(&self, time: i64)
    {
        let i: i64 = time - self.worldInfo().getWorldTime();
        // the tree set holds the entries java shifts through the hash set, the hash set copies only key by position
        self.m_scheduledTickTreeSet.update(|set| {
            *set = std::mem::take(set)
                .into_iter()
                .map(|mut nextticklistentry| {
                    nextticklistentry.m_scheduledTime += i;
                    nextticklistentry
                })
                .collect();
        });
        self.setWorldTime(time);
    }
}
