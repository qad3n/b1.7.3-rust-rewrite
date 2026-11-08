use crate::block::{BLOCK, Block, BlockClass, Material, model_bed};
use crate::entity::entity_player::EnumStatus;
use crate::entity::{Entity, entity_player};
use crate::item::Item;
use crate::world::{ChunkCoordinates, World};
use jvm::array_list::ListIterator;
use jvm::throwable::OrNpe;
use std::rc::Rc;

pub const HEAD_BLOCK_TO_FOOT_BLOCK_MAP: [[i32; 2]; 4] = [[0, 1], [-1, 0], [0, -1], [1, 0]];

fn blockActivated(this: &Rc<Block>, world: &Rc<World>, mut x: i32, y: i32, mut z: i32, player: &Rc<Entity>) -> bool
{
    if world.m_multiplayerWorld.get()
    {
        return true;
    }
    let mut n: i32 = world.getBlockMetadata(x, y, z);
    if !isBlockFootOfBed(n)
    {
        let n2: usize = getDirectionFromMetadata(n) as usize;
        x += HEAD_BLOCK_TO_FOOT_BLOCK_MAP[n2][0];
        z += HEAD_BLOCK_TO_FOOT_BLOCK_MAP[n2][1];
        if world.getBlockId(x, y, z) != this.m_blockID
        {
            return true;
        }
        n = world.getBlockMetadata(x, y, z);
    }
    if !world.worldProvider().canRespawnHere()
    {
        let mut d: f64 = x as f64 + 0.5;
        let mut d2: f64 = y as f64 + 0.5;
        let mut d3: f64 = z as f64 + 0.5;
        world.setBlockWithNotify(x, y, z, 0);
        let n3: usize = getDirectionFromMetadata(n) as usize;
        x += HEAD_BLOCK_TO_FOOT_BLOCK_MAP[n3][0];
        z += HEAD_BLOCK_TO_FOOT_BLOCK_MAP[n3][1];
        if world.getBlockId(x, y, z) == this.m_blockID
        {
            world.setBlockWithNotify(x, y, z, 0);
            d = (d + x as f64 + 0.5) / 2.0;
            d2 = (d2 + y as f64 + 0.5) / 2.0;
            d3 = (d3 + z as f64 + 0.5) / 2.0;
        }
        world.newExplosion(None, (x as f32 + 0.5) as f64, (y as f32 + 0.5) as f64, (z as f32 + 0.5) as f64, 5.0, true);
        return true;
    }
    if isBedOccupied(n)
    {
        let mut entityPlayer: Option<Rc<Entity>> = None;
        let mut iterator: ListIterator = world.m_playerEntities.iterator();
        while iterator.hasNext(&world.m_playerEntities)
        {
            let entityPlayer2: Rc<Entity> = iterator.next(&world.m_playerEntities);
            if !entityPlayer2.isPlayerSleeping()
            {
                continue;
            }
            let chunkCoordinates: ChunkCoordinates =
                entity_player::fields(&entityPlayer2).m_bedChunkCoordinates.get().orNpe();
            if chunkCoordinates.m_x != x || chunkCoordinates.m_y != y || chunkCoordinates.m_z != z
            {
                continue;
            }
            entityPlayer = Some(entityPlayer2);
        }
        if entityPlayer.is_none()
        {
            setBedOccupied(world, x, y, z, false);
        }
        else
        {
            player.addChatMessage("tile.bed.occupied");
            return true;
        }
    }
    let enumStatus: EnumStatus = player.sleepInBedAt(x, y, z);
    if enumStatus == EnumStatus::Ok
    {
        setBedOccupied(world, x, y, z, true);
        return true;
    }
    if enumStatus == EnumStatus::NotPossibleNow
    {
        player.addChatMessage("tile.bed.noSleep");
    }
    true
}

fn getBlockTextureFromSideAndMetadata(this: &Rc<Block>, face: i32, metadata: i32) -> i32
{
    if face == 0
    {
        return Block::planks().m_blockIndexInTexture.get();
    }
    let n: usize = getDirectionFromMetadata(metadata) as usize;
    let n2: i32 = model_bed::BED_DIRECTION[n][face as usize];
    let index: i32 = this.m_blockIndexInTexture.get();
    if isBlockFootOfBed(metadata)
    {
        if n2 == 2
        {
            return index + 2 + 16;
        }
        if n2 == 5 || n2 == 4
        {
            return index + 1 + 16;
        }
        return index + 1;
    }
    if n2 == 3
    {
        return index - 1 + 16;
    }
    if n2 == 5 || n2 == 4
    {
        return index + 16;
    }
    index
}

fn onNeighborBlockChange(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, _neighborBlock: i32)
{
    let n: i32 = world.getBlockMetadata(x, y, z);
    let n2: usize = getDirectionFromMetadata(n) as usize;
    if isBlockFootOfBed(n)
    {
        if world.getBlockId(x - HEAD_BLOCK_TO_FOOT_BLOCK_MAP[n2][0], y, z - HEAD_BLOCK_TO_FOOT_BLOCK_MAP[n2][1])
            != this.m_blockID
        {
            world.setBlockWithNotify(x, y, z, 0);
        }
    }
    else if world.getBlockId(x + HEAD_BLOCK_TO_FOOT_BLOCK_MAP[n2][0], y, z + HEAD_BLOCK_TO_FOOT_BLOCK_MAP[n2][1])
        != this.m_blockID
    {
        world.setBlockWithNotify(x, y, z, 0);
        if !world.m_multiplayerWorld.get()
        {
            this.dropBlockAsItem(world, x, y, z, n);
        }
    }
}

fn setBounds(this: &Block) { this.setBlockBounds(0.0, 0.0, 0.0, 1.0, 0.5625, 1.0); }

pub fn getDirectionFromMetadata(metadata: i32) -> i32 { metadata & 3 }

pub fn isBlockFootOfBed(metadata: i32) -> bool { (metadata & 8) != 0 }

pub fn isBedOccupied(metadata: i32) -> bool { (metadata & 4) != 0 }

pub fn setBedOccupied(world: &Rc<World>, x: i32, y: i32, z: i32, occupied: bool)
{
    let mut i: i32 = world.getBlockMetadata(x, y, z);
    if occupied
    {
        i |= 4;
    }
    else
    {
        i &= -5;
    }
    world.setBlockMetadataWithNotify(x, y, z, i);
}

pub fn getNearestEmptyChunkCoordinates(
    world: &Rc<World>,
    x: i32,
    y: i32,
    z: i32,
    mut skip: i32,
) -> Option<ChunkCoordinates>
{
    let n: i32 = world.getBlockMetadata(x, y, z);
    let n2: usize = getDirectionFromMetadata(n) as usize;
    for i in 0..=1
    {
        let n3: i32 = x - HEAD_BLOCK_TO_FOOT_BLOCK_MAP[n2][0] * i - 1;
        let n4: i32 = z - HEAD_BLOCK_TO_FOOT_BLOCK_MAP[n2][1] * i - 1;
        let n5: i32 = n3 + 2;
        let n6: i32 = n4 + 2;
        for j in n3..=n5
        {
            for k in n4..=n6
            {
                if !world.isBlockNormalCube(j, y - 1, k) || !world.isAirBlock(j, y, k) || !world.isAirBlock(j, y + 1, k)
                {
                    continue;
                }
                if skip > 0
                {
                    skip -= 1;
                    continue;
                }
                return Some(ChunkCoordinates::new(j, y, k));
            }
        }
    }
    None
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockBed",
    m_parent: Some(&BLOCK),
    blockActivated,
    getBlockTextureFromSideAndMetadata,
    getRenderType: |_| 14,
    renderAsNormalBlock: |_| false,
    isOpaqueCube: |_| false,
    setBlockBoundsBasedOnState: |this, _, _, _, _| setBounds(this),
    onNeighborBlockChange,
    idDropped: |_, metadata, _| {
        if isBlockFootOfBed(metadata)
        {
            return 0;
        }
        Item::bed().m_shiftedIndex
    },
    dropBlockAsItemWithChance: |this, world, x, y, z, metadata, luck| {
        if !isBlockFootOfBed(metadata)
        {
            (BLOCK.dropBlockAsItemWithChance)(this, world, x, y, z, metadata, luck);
        }
    },
    getMobilityFlag: |_| 1,
    ..BLOCK
};

pub fn create(id: i32) -> Rc<Block>
{
    let block: Rc<Block> = Block::constructWithSprite(&CLASS, Box::new(()), id, 134, Material::cloth());
    setBounds(&block);
    block
}
