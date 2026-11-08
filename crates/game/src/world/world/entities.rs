use crate::block::{Block, Material, block_fluid};
use crate::entity::Entity;
use crate::math::math_helper::floor_double;
use crate::math::{AxisAlignedBB, Vec3D};
use crate::pathfinding::{PathEntity, Pathfinder};
use crate::tileentity::TileEntity;
use crate::world::{Chunk, ChunkCache, Explosion, World};
use jvm::array_list::JArrayList;
use std::rc::Rc;

fn isPlayer(entity: &Entity) -> bool { entity.instanceOf("EntityPlayer") }

pub fn entityJoinedWorld(this: &Rc<World>, entity: &Rc<Entity>) -> bool
{
    let i: i32 = floor_double(entity.posX() / 16.0);
    let j: i32 = floor_double(entity.posZ() / 16.0);
    let flag: bool = isPlayer(entity);
    if !flag && !this.chunkExists(i, j)
    {
        return false;
    }
    if isPlayer(entity)
    {
        this.m_playerEntities.add(Rc::clone(entity));
        this.updateAllPlayersSleepingFlag();
    }
    this.getChunkFromChunkCoords(i, j).addEntity(entity);
    this.m_loadedEntityList.add(Rc::clone(entity));
    this.obtainEntitySkin(entity);
    true
}

pub fn obtainEntitySkin(this: &Rc<World>, entity: &Rc<Entity>)
{
    let mut i: i32 = 0;
    while i < this.m_worldAccesses.size()
    {
        this.m_worldAccesses.get(i).obtainEntitySkin(entity);
        i += 1;
    }
}

pub fn releaseEntitySkin(this: &Rc<World>, entity: &Rc<Entity>)
{
    let mut i: i32 = 0;
    while i < this.m_worldAccesses.size()
    {
        this.m_worldAccesses.get(i).releaseEntitySkin(entity);
        i += 1;
    }
}

pub fn setEntityDead(this: &Rc<World>, entity: &Rc<Entity>)
{
    if let Some(rider) = entity.m_riddenByEntity.get()
    {
        rider.mountEntity(None);
    }
    if entity.m_ridingEntity.with(Option::is_some)
    {
        entity.mountEntity(None);
    }
    entity.setEntityDead();
    if isPlayer(entity)
    {
        this.m_playerEntities.remove(entity);
        this.updateAllPlayersSleepingFlag();
    }
}

impl World
{
    pub fn removePlayer(self: &Rc<Self>, entity: &Rc<Entity>)
    {
        let this: &Rc<World> = self;
        entity.setEntityDead();
        if isPlayer(entity)
        {
            this.m_playerEntities.remove(entity);
            this.updateAllPlayersSleepingFlag();
        }
        let i: i32 = entity.m_chunkCoordX.get();
        let j: i32 = entity.m_chunkCoordZ.get();
        if entity.m_addedToChunk.get() && this.chunkExists(i, j)
        {
            this.getChunkFromChunkCoords(i, j).removeEntity(entity);
        }
        this.m_loadedEntityList.remove(entity);
        this.releaseEntitySkin(entity);
    }
}

impl World
{
    pub fn playSoundAtEntity(&self, source: &Rc<Entity>, sound: &str, volume: f32, pitch: f32)
    {
        let mut i: i32 = 0;
        while i < self.m_worldAccesses.size()
        {
            self.m_worldAccesses.get(i).playSound(
                sound,
                source.posX(),
                source.posY() - source.m_yOffset.get() as f64,
                source.posZ(),
                volume,
                pitch,
            );
            i += 1;
        }
    }

    pub fn playSoundEffect(&self, x: f64, y: f64, z: f64, sound: &str, volume: f32, pitch: f32)
    {
        let mut i: i32 = 0;
        while i < self.m_worldAccesses.size()
        {
            self.m_worldAccesses.get(i).playSound(sound, x, y, z, volume, pitch);
            i += 1;
        }
    }

    pub fn playRecord(&self, record: Option<&str>, x: i32, y: i32, z: i32)
    {
        let mut i: i32 = 0;
        while i < self.m_worldAccesses.size()
        {
            self.m_worldAccesses.get(i).playRecord(record, x, y, z);
            i += 1;
        }
    }

    pub fn spawnParticle(&self, kind: &str, x: f64, y: f64, z: f64, velocityX: f64, velocityY: f64, velocityZ: f64)
    {
        let mut i: i32 = 0;
        while i < self.m_worldAccesses.size()
        {
            self.m_worldAccesses.get(i).spawnParticle(kind, x, y, z, velocityX, velocityY, velocityZ);
            i += 1;
        }
    }

    pub fn getEntityOfSubtype(&self, _class: &str) -> Option<Rc<Entity>> { None }

    pub fn doEvent(&self, kind: i32, x: i32, y: i32, z: i32, data: i32) { self.doEventFrom(None, kind, x, y, z, data); }

    pub fn doEventFrom(&self, source: Option<&Rc<Entity>>, kind: i32, x: i32, y: i32, z: i32, data: i32)
    {
        let mut i: i32 = 0;
        while i < self.m_worldAccesses.size()
        {
            self.m_worldAccesses.get(i).doEvent(source, kind, x, y, z, data);
            i += 1;
        }
    }

    pub fn getCollidingBoundingBoxes(
        self: &Rc<Self>,
        entity: &Rc<Entity>,
        shape: &AxisAlignedBB,
    ) -> Rc<JArrayList<Rc<AxisAlignedBB>>>
    {
        let boxes: Rc<JArrayList<Rc<AxisAlignedBB>>> = Rc::clone(&self.m_collidingBoundingBoxes);
        boxes.clear();
        let i: i32 = floor_double(shape.minX());
        let j: i32 = floor_double(shape.maxX() + 1.0);
        let k: i32 = floor_double(shape.minY());
        let l: i32 = floor_double(shape.maxY() + 1.0);
        let i1: i32 = floor_double(shape.minZ());
        let j1: i32 = floor_double(shape.maxZ() + 1.0);
        for k1 in i..j
        {
            for l1 in i1..j1
            {
                if self.blockExists(k1, 64, l1)
                {
                    for i2 in (k - 1)..l
                    {
                        if let Some(block) = Block::byId(self.getBlockId(k1, i2, l1))
                        {
                            block.getCollidingBoundingBoxes(self, k1, i2, l1, shape, &boxes);
                        }
                    }
                }
            }
        }

        let d0: f64 = 0.25;
        let list: Rc<JArrayList<Rc<Entity>>> =
            self.getEntitiesWithinAABBExcludingEntity(Some(entity), &shape.expand(d0, d0, d0));
        let mut j2: i32 = 0;
        while j2 < list.size()
        {
            if let Some(bb) = list.get(j2).getBoundingBox()
                && bb.intersectsWith(shape)
            {
                boxes.add(bb);
            }
            if let Some(bb) = entity.getCollisionBox(&list.get(j2))
                && bb.intersectsWith(shape)
            {
                boxes.add(bb);
            }
            j2 += 1;
        }
        boxes
    }

    pub fn updateEntities(self: &Rc<Self>)
    {
        let mut i: i32 = 0;
        while i < self.m_weatherEffects.size()
        {
            let entity: Rc<Entity> = self.m_weatherEffects.get(i);
            entity.onUpdate();
            if entity.m_isDead.get()
            {
                self.m_weatherEffects.removeAt(i);
                i -= 1;
            }
            i += 1;
        }

        self.m_loadedEntityList.removeAll(&self.m_unloadedEntityList);
        let mut l: i32 = 0;
        while l < self.m_unloadedEntityList.size()
        {
            let entity1: Rc<Entity> = self.m_unloadedEntityList.get(l);
            let j: i32 = entity1.m_chunkCoordX.get();
            let k: i32 = entity1.m_chunkCoordZ.get();
            if entity1.m_addedToChunk.get() && self.chunkExists(j, k)
            {
                self.getChunkFromChunkCoords(j, k).removeEntity(&entity1);
            }
            l += 1;
        }
        let mut i1: i32 = 0;
        while i1 < self.m_unloadedEntityList.size()
        {
            self.releaseEntitySkin(&self.m_unloadedEntityList.get(i1));
            i1 += 1;
        }
        self.m_unloadedEntityList.clear();

        let mut j1: i32 = 0;
        while j1 < self.m_loadedEntityList.size()
        {
            let entity2: Rc<Entity> = self.m_loadedEntityList.get(j1);
            if let Some(riding) = entity2.m_ridingEntity.get()
            {
                let ridden: Option<Rc<Entity>> = riding.m_riddenByEntity.get();
                if !riding.m_isDead.get() && ridden.as_ref().is_some_and(|ridden| Rc::ptr_eq(ridden, &entity2))
                {
                    j1 += 1;
                    continue;
                }
                riding.m_riddenByEntity.set(None);
                entity2.m_ridingEntity.set(None);
            }

            if !entity2.m_isDead.get()
            {
                self.updateEntity(&entity2);
            }

            if entity2.m_isDead.get()
            {
                let k1: i32 = entity2.m_chunkCoordX.get();
                let l1: i32 = entity2.m_chunkCoordZ.get();
                if entity2.m_addedToChunk.get() && self.chunkExists(k1, l1)
                {
                    self.getChunkFromChunkCoords(k1, l1).removeEntity(&entity2);
                }
                self.m_loadedEntityList.removeAt(j1);
                j1 -= 1;
                self.releaseEntitySkin(&entity2);
            }
            j1 += 1;
        }

        self.m_isTickingBlockEntities.set(true);
        let mut iterator: jvm::array_list::ListIterator = self.m_loadedTileEntityList.iterator();
        while iterator.hasNext(&self.m_loadedTileEntityList)
        {
            let tileentity: Rc<TileEntity> = iterator.next(&self.m_loadedTileEntityList);
            if !tileentity.isRemoved()
            {
                tileentity.updateEntity();
            }
            if tileentity.isRemoved()
            {
                iterator.remove(&self.m_loadedTileEntityList);
                let chunk: Rc<Chunk> =
                    self.getChunkFromChunkCoords(tileentity.m_xCoord.get() >> 4, tileentity.m_zCoord.get() >> 4);
                chunk.removeChunkBlockTileEntity(
                    tileentity.m_xCoord.get() & 15,
                    tileentity.m_yCoord.get(),
                    tileentity.m_zCoord.get() & 15,
                );
            }
        }
        self.m_isTickingBlockEntities.set(false);

        if !self.m_pendingBlockEntities.isEmpty()
        {
            for tileentity1 in self.m_pendingBlockEntities.toVec()
            {
                if tileentity1.isRemoved()
                {
                    continue;
                }
                if !self.m_loadedTileEntityList.containsIdentity(&tileentity1)
                {
                    self.m_loadedTileEntityList.add(Rc::clone(&tileentity1));
                }
                let chunk1: Rc<Chunk> =
                    self.getChunkFromChunkCoords(tileentity1.m_xCoord.get() >> 4, tileentity1.m_zCoord.get() >> 4);
                chunk1.setChunkBlockTileEntity(
                    tileentity1.m_xCoord.get() & 15,
                    tileentity1.m_yCoord.get(),
                    tileentity1.m_zCoord.get() & 15,
                    &tileentity1,
                );
                self.markBlockNeedsUpdate(
                    tileentity1.m_xCoord.get(),
                    tileentity1.m_yCoord.get(),
                    tileentity1.m_zCoord.get(),
                );
            }
            self.m_pendingBlockEntities.clear();
        }
    }

    pub fn loadBlockEntities(&self, blockEntities: &[Rc<TileEntity>])
    {
        if self.m_isTickingBlockEntities.get()
        {
            self.m_pendingBlockEntities.addAll(blockEntities);
        }
        else
        {
            self.m_loadedTileEntityList.addAll(blockEntities);
        }
    }

    pub fn updateEntity(self: &Rc<Self>, entity: &Rc<Entity>) { self.updateEntityWithOptionalForce(entity, true); }

    pub fn checkIfAABBIsClear(&self, bounds: &AxisAlignedBB) -> bool
    {
        let list: Rc<JArrayList<Rc<Entity>>> = self.getEntitiesWithinAABBExcludingEntity(None, bounds);
        let mut i: i32 = 0;
        while i < list.size()
        {
            let entity: Rc<Entity> = list.get(i);
            if !entity.m_isDead.get() && entity.m_preventEntitySpawning.get()
            {
                return false;
            }
            i += 1;
        }
        true
    }

    fn blockRange(bounds: &AxisAlignedBB) -> (i32, i32, i32, i32, i32, i32)
    {
        (
            floor_double(bounds.minX()),
            floor_double(bounds.maxX() + 1.0),
            floor_double(bounds.minY()),
            floor_double(bounds.maxY() + 1.0),
            floor_double(bounds.minZ()),
            floor_double(bounds.maxZ() + 1.0),
        )
    }

    pub fn getIsAnyLiquid(&self, bounds: &AxisAlignedBB) -> bool
    {
        let (mut i, j, mut k, l, mut i1, j1): (i32, i32, i32, i32, i32, i32) = Self::blockRange(bounds);
        if bounds.minX() < 0.0
        {
            i -= 1;
        }
        if bounds.minY() < 0.0
        {
            k -= 1;
        }
        if bounds.minZ() < 0.0
        {
            i1 -= 1;
        }
        for k1 in i..j
        {
            for l1 in k..l
            {
                for i2 in i1..j1
                {
                    if let Some(block) = Block::byId(self.getBlockId(k1, l1, i2))
                        && block.m_blockMaterial.getIsLiquid()
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn isBoundingBoxBurning(&self, bounds: &AxisAlignedBB) -> bool
    {
        let (i, j, k, l, i1, j1): (i32, i32, i32, i32, i32, i32) = Self::blockRange(bounds);
        if self.checkChunksExist(i, k, i1, j, l, j1)
        {
            for k1 in i..j
            {
                for l1 in k..l
                {
                    for i2 in i1..j1
                    {
                        let j2: i32 = self.getBlockId(k1, l1, i2);
                        if j2 == Block::fire().m_blockID
                            || j2 == Block::lavaMoving().m_blockID
                            || j2 == Block::lavaStill().m_blockID
                        {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    pub fn handleMaterialAcceleration(
        self: &Rc<Self>,
        bounds: &AxisAlignedBB,
        material: Rc<Material>,
        entity: &Rc<Entity>,
    ) -> bool
    {
        let (i, j, k, l, i1, j1): (i32, i32, i32, i32, i32, i32) = Self::blockRange(bounds);
        if !self.checkChunksExist(i, k, i1, j, l, j1)
        {
            return false;
        }

        let mut flag: bool = false;
        let mut vec3d: Rc<Vec3D> = Vec3D::createVector(0.0, 0.0, 0.0);
        for k1 in i..j
        {
            for l1 in k..l
            {
                for i2 in i1..j1
                {
                    let Some(block) = Block::byId(self.getBlockId(k1, l1, i2))
                    else
                    {
                        continue;
                    };
                    if Rc::ptr_eq(&block.m_blockMaterial, &material)
                    {
                        let d0: f64 =
                            ((l1 + 1) as f32 - block_fluid::getPercentAir(self.getBlockMetadata(k1, l1, i2))) as f64;
                        if l as f64 >= d0
                        {
                            flag = true;
                            block.velocityToAddToEntity(self, k1, l1, i2, entity, &vec3d);
                        }
                    }
                }
            }
        }

        if vec3d.lengthVector() > 0.0
        {
            vec3d = vec3d.normalize();
            let d1: f64 = 0.014;
            entity.m_motionX.set(entity.m_motionX.get() + vec3d.x() * d1);
            entity.m_motionY.set(entity.m_motionY.get() + vec3d.y() * d1);
            entity.m_motionZ.set(entity.m_motionZ.get() + vec3d.z() * d1);
        }
        flag
    }

    pub fn isMaterialInBB(&self, bounds: &AxisAlignedBB, material: Rc<Material>) -> bool
    {
        let (i, j, k, l, i1, j1): (i32, i32, i32, i32, i32, i32) = Self::blockRange(bounds);
        for k1 in i..j
        {
            for l1 in k..l
            {
                for i2 in i1..j1
                {
                    if let Some(block) = Block::byId(self.getBlockId(k1, l1, i2))
                        && Rc::ptr_eq(&block.m_blockMaterial, &material)
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn isAABBInMaterial(&self, bounds: &AxisAlignedBB, material: Rc<Material>) -> bool
    {
        let (i, j, k, l, i1, j1): (i32, i32, i32, i32, i32, i32) = Self::blockRange(bounds);
        for k1 in i..j
        {
            for l1 in k..l
            {
                for i2 in i1..j1
                {
                    let Some(block) = Block::byId(self.getBlockId(k1, l1, i2))
                    else
                    {
                        continue;
                    };
                    if Rc::ptr_eq(&block.m_blockMaterial, &material)
                    {
                        let j2: i32 = self.getBlockMetadata(k1, l1, i2);
                        let mut d0: f64 = (l1 + 1) as f64;
                        if j2 < 8
                        {
                            d0 = (l1 + 1) as f64 - j2 as f64 / 8.0;
                        }
                        if d0 >= bounds.minY()
                        {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    pub fn createExplosion(
        self: &Rc<Self>,
        source: Option<&Rc<Entity>>,
        x: f64,
        y: f64,
        z: f64,
        power: f32,
    ) -> Rc<Explosion>
    {
        self.newExplosion(source, x, y, z, power, false)
    }

    pub fn getBlockDensity(self: &Rc<Self>, pos: &Vec3D, bounds: &AxisAlignedBB) -> f32
    {
        let d0: f64 = 1.0 / ((bounds.maxX() - bounds.minX()) * 2.0 + 1.0);
        let d1: f64 = 1.0 / ((bounds.maxY() - bounds.minY()) * 2.0 + 1.0);
        let d2: f64 = 1.0 / ((bounds.maxZ() - bounds.minZ()) * 2.0 + 1.0);
        let mut i: i32 = 0;
        let mut j: i32 = 0;
        let mut f: f32 = 0.0;
        while f <= 1.0
        {
            let mut f1: f32 = 0.0;
            while f1 <= 1.0
            {
                let mut f2: f32 = 0.0;
                while f2 <= 1.0
                {
                    let d3: f64 = bounds.minX() + (bounds.maxX() - bounds.minX()) * f as f64;
                    let d4: f64 = bounds.minY() + (bounds.maxY() - bounds.minY()) * f1 as f64;
                    let d5: f64 = bounds.minZ() + (bounds.maxZ() - bounds.minZ()) * f2 as f64;
                    if self.rayTraceBlocks(&Vec3D::createVector(d3, d4, d5), pos).is_none()
                    {
                        i += 1;
                    }
                    j += 1;
                    f2 = (f2 as f64 + d2) as f32;
                }
                f1 = (f1 as f64 + d1) as f32;
            }
            f = (f as f64 + d0) as f32;
        }
        i as f32 / j as f32
    }

    pub fn onBlockHit(self: &Rc<Self>, player: Option<&Rc<Entity>>, x: i32, y: i32, z: i32, face: i32)
    {
        let (mut x, mut y, mut z): (i32, i32, i32) = (x, y, z);
        match face
        {
            0 => y -= 1,
            1 => y += 1,
            2 => z -= 1,
            3 => z += 1,
            4 => x -= 1,
            5 => x += 1,
            _ => (),
        }
        if self.getBlockId(x, y, z) == Block::fire().m_blockID
        {
            self.doEventFrom(player, 1004, x, y, z, 0);
            self.setBlockWithNotify(x, y, z, 0);
        }
    }

    pub fn getEntitiesWithinAABBExcludingEntity(
        &self,
        exclude: Option<&Rc<Entity>>,
        bounds: &AxisAlignedBB,
    ) -> Rc<JArrayList<Rc<Entity>>>
    {
        let area: Rc<JArrayList<Rc<Entity>>> = Rc::clone(&self.m_entitiesInArea);
        area.clear();
        let i: i32 = floor_double((bounds.minX() - 2.0) / 16.0);
        let j: i32 = floor_double((bounds.maxX() + 2.0) / 16.0);
        let k: i32 = floor_double((bounds.minZ() - 2.0) / 16.0);
        let l: i32 = floor_double((bounds.maxZ() + 2.0) / 16.0);
        for i1 in i..=j
        {
            for j1 in k..=l
            {
                if self.chunkExists(i1, j1)
                {
                    self.getChunkFromChunkCoords(i1, j1).getEntitiesWithinAABBForEntity(exclude, bounds, &area);
                }
            }
        }
        area
    }

    pub fn getEntitiesWithinAABB(&self, kind: &str, bounds: &AxisAlignedBB) -> Vec<Rc<Entity>>
    {
        let i: i32 = floor_double((bounds.minX() - 2.0) / 16.0);
        let j: i32 = floor_double((bounds.maxX() + 2.0) / 16.0);
        let k: i32 = floor_double((bounds.minZ() - 2.0) / 16.0);
        let l: i32 = floor_double((bounds.maxZ() + 2.0) / 16.0);
        let mut arraylist: Vec<Rc<Entity>> = Vec::new();
        for i1 in i..=j
        {
            for j1 in k..=l
            {
                if self.chunkExists(i1, j1)
                {
                    self.getChunkFromChunkCoords(i1, j1).getEntitiesOfTypeWithinAAAB(kind, bounds, &mut arraylist);
                }
            }
        }
        arraylist
    }

    pub fn getLoadedEntityList(&self) -> Rc<JArrayList<Rc<Entity>>> { Rc::clone(&self.m_loadedEntityList) }

    pub fn countEntities(&self, kind: &str) -> i32
    {
        let mut i: i32 = 0;
        let mut j: i32 = 0;
        while j < self.m_loadedEntityList.size()
        {
            if self.m_loadedEntityList.get(j).instanceOf(kind)
            {
                i += 1;
            }
            j += 1;
        }
        i
    }

    pub fn loadEntities(self: &Rc<Self>, entities: &[Rc<Entity>])
    {
        self.m_loadedEntityList.addAll(entities);
        for entity in entities
        {
            self.obtainEntitySkin(entity);
        }
    }

    pub fn unloadEntities(&self, entities: &[Rc<Entity>]) { self.m_unloadedEntityList.addAll(entities); }

    pub fn getPathToEntity(
        self: &Rc<Self>,
        entity: &Rc<Entity>,
        target: &Rc<Entity>,
        range: f32,
    ) -> Option<Rc<PathEntity>>
    {
        let i: i32 = floor_double(entity.posX());
        let j: i32 = floor_double(entity.posY());
        let k: i32 = floor_double(entity.posZ());
        let l: i32 = (range + 16.0) as i32;
        let chunkcache: Rc<ChunkCache> = ChunkCache::new(self, i - l, j - l, k - l, i + l, j + l, k + l);
        Pathfinder::new(chunkcache).createEntityPathTo(entity, target, range)
    }

    pub fn getEntityPathToXYZ(
        self: &Rc<Self>,
        entity: &Rc<Entity>,
        x: i32,
        y: i32,
        z: i32,
        range: f32,
    ) -> Option<Rc<PathEntity>>
    {
        let i: i32 = floor_double(entity.posX());
        let j: i32 = floor_double(entity.posY());
        let k: i32 = floor_double(entity.posZ());
        let l: i32 = (range + 8.0) as i32;
        let chunkcache: Rc<ChunkCache> = ChunkCache::new(self, i - l, j - l, k - l, i + l, j + l, k + l);
        Pathfinder::new(chunkcache).createEntityPathToXYZ(entity, x, y, z, range)
    }

    pub fn getClosestPlayerToEntity(&self, entity: &Entity, range: f64) -> Option<Rc<Entity>>
    {
        self.getClosestPlayer(entity.posX(), entity.posY(), entity.posZ(), range)
    }

    pub fn getClosestPlayer(&self, x: f64, y: f64, z: f64, range: f64) -> Option<Rc<Entity>>
    {
        let mut d0: f64 = -1.0;
        let mut entityplayer: Option<Rc<Entity>> = None;
        let mut i: i32 = 0;
        while i < self.m_playerEntities.size()
        {
            let entityplayer1: Rc<Entity> = self.m_playerEntities.get(i);
            let d1: f64 = entityplayer1.getDistanceSq(x, y, z);
            if (range < 0.0 || d1 < range * range) && (d0 == -1.0 || d1 < d0)
            {
                d0 = d1;
                entityplayer = Some(entityplayer1);
            }
            i += 1;
        }
        entityplayer
    }

    pub fn getPlayerEntityByName(&self, name: &jvm::string::JString) -> Option<Rc<Entity>>
    {
        let mut i: i32 = 0;
        while i < self.m_playerEntities.size()
        {
            let player: Rc<Entity> = self.m_playerEntities.get(i);
            if crate::entity::entity_player::fields(&player).m_username.get().as_ref() == Some(name)
            {
                return Some(player);
            }
            i += 1;
        }
        None
    }

    pub fn joinEntityInSurroundings(&self, entity: &Rc<Entity>)
    {
        let i: i32 = floor_double(entity.posX() / 16.0);
        let j: i32 = floor_double(entity.posZ() / 16.0);
        let b0: i32 = 2;
        for k in (i - b0)..=(i + b0)
        {
            for l in (j - b0)..=(j + b0)
            {
                self.getChunkFromChunkCoords(k, l);
            }
        }
        if !self.m_loadedEntityList.contains(entity)
        {
            self.m_loadedEntityList.add(Rc::clone(entity));
        }
    }

    pub fn updateEntityList(self: &Rc<Self>)
    {
        self.m_loadedEntityList.removeAll(&self.m_unloadedEntityList);
        let mut i: i32 = 0;
        while i < self.m_unloadedEntityList.size()
        {
            let entity: Rc<Entity> = self.m_unloadedEntityList.get(i);
            let j: i32 = entity.m_chunkCoordX.get();
            let k: i32 = entity.m_chunkCoordZ.get();
            if entity.m_addedToChunk.get() && self.chunkExists(j, k)
            {
                self.getChunkFromChunkCoords(j, k).removeEntity(&entity);
            }
            i += 1;
        }
        let mut l: i32 = 0;
        while l < self.m_unloadedEntityList.size()
        {
            self.releaseEntitySkin(&self.m_unloadedEntityList.get(l));
            l += 1;
        }
        self.m_unloadedEntityList.clear();

        let mut i1: i32 = 0;
        while i1 < self.m_loadedEntityList.size()
        {
            let entity1: Rc<Entity> = self.m_loadedEntityList.get(i1);
            if let Some(riding) = entity1.m_ridingEntity.get()
            {
                let ridden: Option<Rc<Entity>> = riding.m_riddenByEntity.get();
                if !riding.m_isDead.get() && ridden.as_ref().is_some_and(|ridden| Rc::ptr_eq(ridden, &entity1))
                {
                    i1 += 1;
                    continue;
                }
                riding.m_riddenByEntity.set(None);
                entity1.m_ridingEntity.set(None);
            }
            if entity1.m_isDead.get()
            {
                let j1: i32 = entity1.m_chunkCoordX.get();
                let k1: i32 = entity1.m_chunkCoordZ.get();
                if entity1.m_addedToChunk.get() && self.chunkExists(j1, k1)
                {
                    self.getChunkFromChunkCoords(j1, k1).removeEntity(&entity1);
                }
                self.m_loadedEntityList.removeAt(i1);
                i1 -= 1;
                self.releaseEntitySkin(&entity1);
            }
            i1 += 1;
        }
    }

    pub fn updateAllPlayersSleepingFlag(&self)
    {
        self.m_allPlayersSleeping.set(!self.m_playerEntities.isEmpty());
        for entityplayer in self.m_playerEntities.toVec()
        {
            if !entityplayer.isPlayerSleeping()
            {
                self.m_allPlayersSleeping.set(false);
                break;
            }
        }
    }

    pub fn wakeUpAllPlayers(&self)
    {
        self.m_allPlayersSleeping.set(false);
        for entityplayer in self.m_playerEntities.toVec()
        {
            if entityplayer.isPlayerSleeping()
            {
                entityplayer.wakeUpPlayer(false, false, true);
            }
        }
        self.stopPrecipitation();
    }

    pub fn isAllPlayersFullyAsleep(&self) -> bool
    {
        if self.m_allPlayersSleeping.get() && !self.m_multiplayerWorld.get()
        {
            for entityplayer in self.m_playerEntities.toVec()
            {
                if !crate::entity::entity_player::isPlayerFullyAsleep(&entityplayer)
                {
                    return false;
                }
            }
            return true;
        }
        false
    }
}

pub fn addWeatherEffect(this: &Rc<World>, entity: Rc<Entity>) -> bool
{
    this.m_weatherEffects.add(entity);
    true
}

pub fn updateEntityWithOptionalForce(this: &Rc<World>, entity: &Rc<Entity>, requireLoaded: bool)
{
    let i: i32 = floor_double(entity.posX());
    let j: i32 = floor_double(entity.posZ());
    let b0: i32 = 32;
    if requireLoaded && !this.checkChunksExist(i - b0, 0, j - b0, i + b0, 128, j + b0)
    {
        return;
    }

    entity.m_lastTickPosX.set(entity.posX());
    entity.m_lastTickPosY.set(entity.posY());
    entity.m_lastTickPosZ.set(entity.posZ());
    entity.m_prevRotationYaw.set(entity.m_rotationYaw.get());
    entity.m_prevRotationPitch.set(entity.m_rotationPitch.get());
    if requireLoaded && entity.m_addedToChunk.get()
    {
        if entity.m_ridingEntity.with(Option::is_some)
        {
            entity.updateRidden();
        }
        else
        {
            entity.onUpdate();
        }
    }

    if entity.posX().is_nan() || entity.posX().is_infinite()
    {
        entity.m_posX.set(entity.m_lastTickPosX.get());
    }
    if entity.posY().is_nan() || entity.posY().is_infinite()
    {
        entity.m_posY.set(entity.m_lastTickPosY.get());
    }
    if entity.posZ().is_nan() || entity.posZ().is_infinite()
    {
        entity.m_posZ.set(entity.m_lastTickPosZ.get());
    }
    if entity.m_rotationPitch.get().is_nan() || entity.m_rotationPitch.get().is_infinite()
    {
        entity.m_rotationPitch.set(entity.m_prevRotationPitch.get());
    }
    if entity.m_rotationYaw.get().is_nan() || entity.m_rotationYaw.get().is_infinite()
    {
        entity.m_rotationYaw.set(entity.m_prevRotationYaw.get());
    }

    let k: i32 = floor_double(entity.posX() / 16.0);
    let l: i32 = floor_double(entity.posY() / 16.0);
    let i1: i32 = floor_double(entity.posZ() / 16.0);
    if !entity.m_addedToChunk.get()
        || entity.m_chunkCoordX.get() != k
        || entity.m_chunkCoordY.get() != l
        || entity.m_chunkCoordZ.get() != i1
    {
        if entity.m_addedToChunk.get() && this.chunkExists(entity.m_chunkCoordX.get(), entity.m_chunkCoordZ.get())
        {
            this.getChunkFromChunkCoords(entity.m_chunkCoordX.get(), entity.m_chunkCoordZ.get())
                .removeEntityAtIndex(entity, entity.m_chunkCoordY.get());
        }
        if this.chunkExists(k, i1)
        {
            entity.m_addedToChunk.set(true);
            this.getChunkFromChunkCoords(k, i1).addEntity(entity);
        }
        else
        {
            entity.m_addedToChunk.set(false);
        }
    }

    if requireLoaded
        && entity.m_addedToChunk.get()
        && let Some(rider) = entity.m_riddenByEntity.get()
    {
        let riding: Option<Rc<Entity>> = rider.m_ridingEntity.get();
        if !rider.m_isDead.get() && riding.as_ref().is_some_and(|riding| Rc::ptr_eq(riding, entity))
        {
            this.updateEntity(&rider);
        }
        else
        {
            rider.m_ridingEntity.set(None);
            entity.m_riddenByEntity.set(None);
        }
    }
}

pub fn newExplosion(
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
    explosion.doExplosionB(true);
    explosion
}
