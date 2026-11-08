use super::IS_LIT;
use crate::block::Block;
use crate::world::{Chunk, EnumSkyBlock, World};
use common::world::NibbleArray;
use jvm::throwable::OrNpe;
use std::rc::Rc;

pub fn generateHeightMap(this: &Rc<Chunk>)
{
    let mut i: i32 = 127;
    for j in 0..16
    {
        for k in 0..16
        {
            let mut l: i32 = 127;
            let i1: i32 = j << 11 | k << 7;
            while l > 0 && Block::lightOpacity(this.rawBlock(i1 + l - 1) as i32 & 0xFF) == 0
            {
                l -= 1;
            }
            this.m_heightMap.update(|map| map[(k << 4 | j) as usize] = l as i8);
            if l < i
            {
                i = l;
            }
        }
    }
    this.m_lowestBlockHeight.set(i);
    this.m_isModified.set(true);
}

pub fn populateHeightMap(this: &Rc<Chunk>)
{
    let mut lowestBlockHeight: i32 = 127;
    let hasNoSky: bool = this.world().worldProvider().m_hasNoSky.get();
    let skylight: Rc<NibbleArray> = this.skylight();
    for i in 0..16
    {
        for j in 0..16
        {
            let mut n: i32 = 127;
            let n2: i32 = i << 11 | j << 7;
            while n > 0 && Block::lightOpacity(this.rawBlock(n2 + n - 1) as i32 & 0xFF) == 0
            {
                n -= 1;
            }
            this.m_heightMap.update(|map| map[(j << 4 | i) as usize] = n as i8);
            if n < lowestBlockHeight
            {
                lowestBlockHeight = n;
            }
            if !hasNoSky
            {
                let mut value: i32 = 15;
                let mut y: i32 = 127;
                loop
                {
                    value -= Block::lightOpacity(this.rawBlock(n2 + y) as i32 & 0xFF);
                    if value > 0
                    {
                        skylight.setNibble(i, y, j, value);
                    }
                    y -= 1;
                    if !(y > 0 && value > 0)
                    {
                        break;
                    }
                }
            }
        }
    }
    this.m_lowestBlockHeight.set(lowestBlockHeight);
    for k in 0..16
    {
        for l in 0..16
        {
            lightGaps(this, k, l);
        }
    }
    this.m_isModified.set(true);
}

fn lightGaps(this: &Rc<Chunk>, localX: i32, localZ: i32)
{
    let i: i32 = this.getHeightValue(localX, localZ);
    let j: i32 = this.m_xPosition * 16 + localX;
    let k: i32 = this.m_zPosition * 16 + localZ;
    lightGap(this, j - 1, k, i);
    lightGap(this, j + 1, k, i);
    lightGap(this, j, k - 1, i);
    lightGap(this, j, k + 1, i);
}

fn lightGap(this: &Rc<Chunk>, x: i32, z: i32, y: i32)
{
    let world: Rc<World> = this.world();
    let i: i32 = world.getHeightValue(x, z);
    if i > y
    {
        world.scheduleLightingUpdate(EnumSkyBlock::Sky, x, y, z, x, i, z);
        this.m_isModified.set(true);
    }
    else if i < y
    {
        world.scheduleLightingUpdate(EnumSkyBlock::Sky, x, i, z, x, y, z);
        this.m_isModified.set(true);
    }
}

fn updateHeightMap(this: &Rc<Chunk>, localX: i32, y: i32, localZ: i32)
{
    let world: Rc<World> = this.world();
    let i: i32 = this.height(localZ << 4 | localX);
    let mut j: i32 = i;
    if y > i
    {
        j = y;
    }
    let k: i32 = localX << 11 | localZ << 7;
    while j > 0 && Block::lightOpacity(this.rawBlock(k + j - 1) as i32 & 0xFF) == 0
    {
        j -= 1;
    }
    if j == i
    {
        return;
    }

    world.markBlocksDirtyVertical(localX, localZ, j, i);
    this.m_heightMap.update(|map| map[(localZ << 4 | localX) as usize] = j as i8);
    if j < this.m_lowestBlockHeight.get()
    {
        this.m_lowestBlockHeight.set(j);
    }
    else
    {
        let mut l: i32 = 127;
        for i1 in 0..16
        {
            for j1 in 0..16
            {
                if this.height(j1 << 4 | i1) < l
                {
                    l = this.height(j1 << 4 | i1);
                }
            }
        }
        this.m_lowestBlockHeight.set(l);
    }

    let skylight: Rc<NibbleArray> = this.skylight();
    let i2: i32 = this.m_xPosition * 16 + localX;
    let j2: i32 = this.m_zPosition * 16 + localZ;
    if j < i
    {
        for k2 in j..i
        {
            skylight.setNibble(localX, k2, localZ, 15);
        }
    }
    else
    {
        world.scheduleLightingUpdate(EnumSkyBlock::Sky, i2, i, j2, i2, j, j2);
        for l2 in i..j
        {
            skylight.setNibble(localX, l2, localZ, 0);
        }
    }

    let mut i3: i32 = 15;
    let k1: i32 = j;
    while j > 0 && i3 > 0
    {
        j -= 1;
        let mut l1: i32 = Block::lightOpacity(this.getBlockID(localX, j, localZ));
        if l1 == 0
        {
            l1 = 1;
        }
        i3 -= l1;
        if i3 < 0
        {
            i3 = 0;
        }
        skylight.setNibble(localX, j, localZ, i3);
    }
    while j > 0 && Block::lightOpacity(this.getBlockID(localX, j - 1, localZ)) == 0
    {
        j -= 1;
    }
    if j != k1
    {
        world.scheduleLightingUpdate(EnumSkyBlock::Sky, i2 - 1, j, j2 - 1, i2 + 1, k1, j2 + 1);
    }
    this.m_isModified.set(true);
}

pub fn setBlockIDWithMetadata(this: &Rc<Chunk>, localX: i32, y: i32, localZ: i32, block: i32, metadata: i32) -> bool
{
    let b0: i8 = block as i8;
    let i: i32 = this.height(localZ << 4 | localX);
    let index: i32 = localX << 11 | localZ << 7 | y;
    let j: i32 = this.rawBlock(index) as i32 & 255;
    if j == block && this.data().getNibble(localX, y, localZ) == metadata
    {
        return false;
    }

    let world: Rc<World> = this.world();
    let k: i32 = this.m_xPosition * 16 + localX;
    let l: i32 = this.m_zPosition * 16 + localZ;
    this.m_blocks.update(|blocks| blocks[index as usize] = (b0 as i32 & 0xFF) as i8);
    if j != 0 && !world.m_multiplayerWorld.get()
    {
        Block::byId(j).orNpe().onBlockRemoval(&world, k, y, l);
    }
    this.data().setNibble(localX, y, localZ, metadata);
    if !world.worldProvider().m_hasNoSky.get()
    {
        if Block::lightOpacity(b0 as i32 & 0xFF) != 0
        {
            if y >= i
            {
                updateHeightMap(this, localX, y + 1, localZ);
            }
        }
        else if y == i - 1
        {
            updateHeightMap(this, localX, y, localZ);
        }
        world.scheduleLightingUpdate(EnumSkyBlock::Sky, k, y, l, k, y, l);
    }
    world.scheduleLightingUpdate(EnumSkyBlock::Block, k, y, l, k, y, l);
    lightGaps(this, localX, localZ);
    this.data().setNibble(localX, y, localZ, metadata);
    if block != 0
    {
        Block::byId(block).orNpe().onBlockAdded(&world, k, y, l);
    }
    this.m_isModified.set(true);
    true
}

pub fn setBlockID(this: &Rc<Chunk>, localX: i32, y: i32, localZ: i32, block: i32) -> bool
{
    let b0: i8 = block as i8;
    let i: i32 = this.height(localZ << 4 | localX);
    let index: i32 = localX << 11 | localZ << 7 | y;
    let j: i32 = this.rawBlock(index) as i32 & 255;
    if j == block
    {
        return false;
    }

    let world: Rc<World> = this.world();
    let k: i32 = this.m_xPosition * 16 + localX;
    let l: i32 = this.m_zPosition * 16 + localZ;
    this.m_blocks.update(|blocks| blocks[index as usize] = (b0 as i32 & 0xFF) as i8);
    if j != 0
    {
        Block::byId(j).orNpe().onBlockRemoval(&world, k, y, l);
    }
    this.data().setNibble(localX, y, localZ, 0);
    if Block::lightOpacity(b0 as i32 & 0xFF) != 0
    {
        if y >= i
        {
            updateHeightMap(this, localX, y + 1, localZ);
        }
    }
    else if y == i - 1
    {
        updateHeightMap(this, localX, y, localZ);
    }
    world.scheduleLightingUpdate(EnumSkyBlock::Sky, k, y, l, k, y, l);
    world.scheduleLightingUpdate(EnumSkyBlock::Block, k, y, l, k, y, l);
    lightGaps(this, localX, localZ);
    if block != 0 && !world.m_multiplayerWorld.get()
    {
        Block::byId(block).orNpe().onBlockAdded(&world, k, y, l);
    }
    this.m_isModified.set(true);
    true
}

pub fn getSavedLightValue(this: &Rc<Chunk>, kind: EnumSkyBlock, localX: i32, y: i32, localZ: i32) -> i32
{
    match kind
    {
        EnumSkyBlock::Sky => this.skylight().getNibble(localX, y, localZ),
        EnumSkyBlock::Block => this.blocklight().getNibble(localX, y, localZ),
    }
}

pub fn setLightValue(this: &Rc<Chunk>, kind: EnumSkyBlock, localX: i32, y: i32, localZ: i32, light: i32)
{
    this.m_isModified.set(true);
    match kind
    {
        EnumSkyBlock::Sky => this.skylight().setNibble(localX, y, localZ, light),
        EnumSkyBlock::Block => this.blocklight().setNibble(localX, y, localZ, light),
    }
}

pub fn getBlockLightValue(this: &Rc<Chunk>, localX: i32, y: i32, localZ: i32, ambientDarkness: i32) -> i32
{
    let mut i: i32 = this.skylight().getNibble(localX, y, localZ);
    if i > 0
    {
        IS_LIT.with(|lit| lit.set(true));
    }
    i -= ambientDarkness;
    let j: i32 = this.blocklight().getNibble(localX, y, localZ);
    if j > i
    {
        i = j;
    }
    i
}

pub fn setChunkData(
    this: &Rc<Chunk>,
    data: &[i8],
    minX: i32,
    minY: i32,
    minZ: i32,
    maxX: i32,
    maxY: i32,
    maxZ: i32,
    size: i32,
) -> i32
{
    let mut size: i32 = size;
    for i in minX..maxX
    {
        for j in minZ..maxZ
        {
            let k: i32 = i << 11 | j << 7 | minY;
            let l: i32 = maxY - minY;
            this.m_blocks.update(|blocks| {
                blocks[k as usize..(k + l) as usize].copy_from_slice(&data[size as usize..(size + l) as usize])
            });
            size += l;
        }
    }
    this.generateHeightMap();

    for target in [this.data(), this.blocklight(), this.skylight()]
    {
        for i1 in minX..maxX
        {
            for l1 in minZ..maxZ
            {
                let k2: i32 = (i1 << 11 | l1 << 7 | minY) >> 1;
                let j3: i32 = (maxY - minY) / 2;
                target.m_data.update(|nibbles| {
                    nibbles[k2 as usize..(k2 + j3) as usize].copy_from_slice(&data[size as usize..(size + j3) as usize])
                });
                size += j3;
            }
        }
    }
    size
}

pub fn getChunkData(
    this: &Rc<Chunk>,
    data: &mut [i8],
    minX: i32,
    minY: i32,
    minZ: i32,
    maxX: i32,
    maxY: i32,
    maxZ: i32,
    size: i32,
) -> i32
{
    let mut size: i32 = size;
    let i: i32 = maxX - minX;
    let j: i32 = maxY - minY;
    let k: i32 = maxZ - minZ;
    if i * j * k == this.m_blocks.with(|blocks| blocks.len() as i32)
    {
        this.m_blocks.with(|blocks| data[size as usize..size as usize + blocks.len()].copy_from_slice(blocks));
        size += this.m_blocks.with(|blocks| blocks.len() as i32);
        for source in [this.data(), this.blocklight(), this.skylight()]
        {
            source.m_data.with(|nibbles| data[size as usize..size as usize + nibbles.len()].copy_from_slice(nibbles));
            size += source.m_data.with(|nibbles| nibbles.len() as i32);
        }
        return size;
    }
    for l in minX..maxX
    {
        for i1 in minZ..maxZ
        {
            let j1: i32 = l << 11 | i1 << 7 | minY;
            let k1: i32 = maxY - minY;
            this.m_blocks.with(|blocks| {
                data[size as usize..(size + k1) as usize].copy_from_slice(&blocks[j1 as usize..(j1 + k1) as usize])
            });
            size += k1;
        }
    }
    for source in [this.data(), this.blocklight(), this.skylight()]
    {
        for l1 in minX..maxX
        {
            for k2 in minZ..maxZ
            {
                let j3: i32 = (l1 << 11 | k2 << 7 | minY) >> 1;
                let i4: i32 = (maxY - minY) / 2;
                source.m_data.with(|nibbles| {
                    data[size as usize..(size + i4) as usize].copy_from_slice(&nibbles[j3 as usize..(j3 + i4) as usize])
                });
                size += i4;
            }
        }
    }
    size
}
