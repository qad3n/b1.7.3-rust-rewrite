use crate::block::block_fluid::{self, getFlowDecay};
use crate::block::{Block, BlockClass, Material};
use crate::world::World;
use jvm::random::Random;
use jvm::throwable::OrNpe;
use std::cell::Cell;
use std::rc::Rc;

#[derive(Default)]
pub struct FlowingExt
{
    pub m_numAdjacentSources: Cell<i32>,
    pub m_isOptimalFlowDirection: Cell<[bool; 4]>,
    pub m_flowCost: Cell<[i32; 4]>,
}

fn ext(this: &Block) -> &FlowingExt { this.ext::<FlowingExt>() }

fn is(material: &Rc<Material>, other: &Rc<Material>) -> bool { Rc::ptr_eq(material, other) }

fn convertToSource(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32)
{
    let n: i32 = world.getBlockMetadata(x, y, z);
    world.setBlockAndMetadata(x, y, z, this.m_blockID + 1, n);
    world.markBlocksDirty(x, y, z, x, y, z);
    world.markBlockNeedsUpdate(x, y, z);
}

fn updateTick(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, random: &Random)
{
    let mut i: i32 = getFlowDecay(this, world, x, y, z);
    let mut b0: i32 = 1;
    if is(&this.m_blockMaterial, &Material::lava()) && !world.worldProvider().m_isHellWorld.get()
    {
        b0 = 2;
    }

    let mut flag: bool = true;
    if i > 0
    {
        let mut j: i32 = -100;
        ext(this).m_numAdjacentSources.set(0);
        j = getSmallestFlowDecay(this, world, x - 1, y, z, j);
        j = getSmallestFlowDecay(this, world, x + 1, y, z, j);
        j = getSmallestFlowDecay(this, world, x, y, z - 1, j);
        j = getSmallestFlowDecay(this, world, x, y, z + 1, j);
        let mut k: i32 = j + b0;
        if k >= 8 || j < 0
        {
            k = -1;
        }
        if getFlowDecay(this, world, x, y + 1, z) >= 0
        {
            let l: i32 = getFlowDecay(this, world, x, y + 1, z);
            k = if l >= 8 { l } else { l + 8 };
        }
        if ext(this).m_numAdjacentSources.get() >= 2 && is(&this.m_blockMaterial, &Material::water())
        {
            if world.getBlockMaterial(x, y - 1, z).isSolid()
            {
                k = 0;
            }
            else if is(&world.getBlockMaterial(x, y - 1, z), &this.m_blockMaterial)
                && world.getBlockMetadata(x, y, z) == 0
            {
                k = 0;
            }
        }
        if is(&this.m_blockMaterial, &Material::lava()) && i < 8 && k < 8 && k > i && random.nextIntBounded(4) != 0
        {
            k = i;
            flag = false;
        }
        if k != i
        {
            i = k;
            if i < 0
            {
                world.setBlockWithNotify(x, y, z, 0);
            }
            else
            {
                world.setBlockMetadataWithNotify(x, y, z, i);
                world.scheduleBlockUpdate(x, y, z, this.m_blockID, this.tickRate());
                world.notifyBlocksOfNeighborChange(x, y, z, this.m_blockID);
            }
        }
        else if flag
        {
            convertToSource(this, world, x, y, z);
        }
    }
    else
    {
        convertToSource(this, world, x, y, z);
    }

    if liquidCanDisplaceBlock(this, world, x, y - 1, z)
    {
        if i >= 8
        {
            world.setBlockAndMetadataWithNotify(x, y - 1, z, this.m_blockID, i);
        }
        else
        {
            world.setBlockAndMetadataWithNotify(x, y - 1, z, this.m_blockID, i + 8);
        }
    }
    else if i >= 0 && (i == 0 || blockBlocksFlow(world, x, y - 1, z))
    {
        let aboolean: [bool; 4] = getOptimalFlowDirections(this, world, x, y, z);
        let mut i1: i32 = i + b0;
        if i >= 8
        {
            i1 = 1;
        }
        if i1 >= 8
        {
            return;
        }
        if aboolean[0]
        {
            flowIntoBlock(this, world, x - 1, y, z, i1);
        }
        if aboolean[1]
        {
            flowIntoBlock(this, world, x + 1, y, z, i1);
        }
        if aboolean[2]
        {
            flowIntoBlock(this, world, x, y, z - 1, i1);
        }
        if aboolean[3]
        {
            flowIntoBlock(this, world, x, y, z + 1, i1);
        }
    }
}

fn flowIntoBlock(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, depth: i32)
{
    if !liquidCanDisplaceBlock(this, world, x, y, z)
    {
        return;
    }
    let n: i32 = world.getBlockId(x, y, z);
    if n > 0
    {
        if is(&this.m_blockMaterial, &Material::lava())
        {
            block_fluid::triggerLavaMixEffects(world, x, y, z);
        }
        else
        {
            Block::byId(n).orNpe().dropBlockAsItem(world, x, y, z, world.getBlockMetadata(x, y, z));
        }
    }
    world.setBlockAndMetadataWithNotify(x, y, z, this.m_blockID, depth);
}

fn offset(i: i32, x: i32, z: i32) -> (i32, i32)
{
    match i
    {
        0 => (x - 1, z),
        1 => (x + 1, z),
        2 => (x, z - 1),
        _ => (x, z + 1),
    }
}

fn calculateFlowCost(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, distance: i32, fromDir: i32) -> i32
{
    let mut n: i32 = 1000;
    for i in 0..4
    {
        if i == 0 && fromDir == 1 || i == 1 && fromDir == 0 || i == 2 && fromDir == 3 || i == 3 && fromDir == 2
        {
            continue;
        }
        let (n3, n5): (i32, i32) = offset(i, x, z);
        let n4: i32 = y;
        if blockBlocksFlow(world, n3, n4, n5)
            || is(&world.getBlockMaterial(n3, n4, n5), &this.m_blockMaterial) && world.getBlockMetadata(n3, n4, n5) == 0
        {
            continue;
        }
        if !blockBlocksFlow(world, n3, n4 - 1, n5)
        {
            return distance;
        }
        if distance >= 4
        {
            continue;
        }
        let n2: i32 = calculateFlowCost(this, world, n3, n4, n5, distance + 1, i);
        if n2 >= n
        {
            continue;
        }
        n = n2;
    }
    n
}

fn getOptimalFlowDirections(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32) -> [bool; 4]
{
    for n2 in 0..4
    {
        let mut costs: [i32; 4] = ext(this).m_flowCost.get();
        costs[n2 as usize] = 1000;
        ext(this).m_flowCost.set(costs);
        let (n, n4): (i32, i32) = offset(n2, x, z);
        let n3: i32 = y;
        if blockBlocksFlow(world, n, n3, n4)
            || is(&world.getBlockMaterial(n, n3, n4), &this.m_blockMaterial) && world.getBlockMetadata(n, n3, n4) == 0
        {
            continue;
        }
        let cost: i32 = if !blockBlocksFlow(world, n, n3 - 1, n4)
        {
            0
        }
        else
        {
            calculateFlowCost(this, world, n, n3, n4, 1, n2)
        };
        let mut costs: [i32; 4] = ext(this).m_flowCost.get();
        costs[n2 as usize] = cost;
        ext(this).m_flowCost.set(costs);
    }

    let costs: [i32; 4] = ext(this).m_flowCost.get();
    let mut n2: i32 = costs[0];
    for n in 1..4
    {
        if costs[n] >= n2
        {
            continue;
        }
        n2 = costs[n];
    }
    let mut optimal: [bool; 4] = [false; 4];
    for n in 0..4
    {
        optimal[n] = costs[n] == n2;
    }
    ext(this).m_isOptimalFlowDirection.set(optimal);
    optimal
}

fn blockBlocksFlow(world: &Rc<World>, x: i32, y: i32, z: i32) -> bool
{
    let n: i32 = world.getBlockId(x, y, z);
    if n == Block::doorWood().m_blockID
        || n == Block::doorSteel().m_blockID
        || n == Block::signPost().m_blockID
        || n == Block::ladder().m_blockID
        || n == Block::reed().m_blockID
    {
        return true;
    }
    if n == 0
    {
        return false;
    }
    Block::byId(n).orNpe().m_blockMaterial.getIsSolid()
}

fn getSmallestFlowDecay(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, depth: i32) -> i32
{
    let mut n: i32 = getFlowDecay(this, world, x, y, z);
    if n < 0
    {
        return depth;
    }
    if n == 0
    {
        ext(this).m_numAdjacentSources.set(ext(this).m_numAdjacentSources.get() + 1);
    }
    if n >= 8
    {
        n = 0;
    }
    if depth < 0 || n < depth { n } else { depth }
}

fn liquidCanDisplaceBlock(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32) -> bool
{
    let material: Rc<Material> = world.getBlockMaterial(x, y, z);
    if is(&material, &this.m_blockMaterial) || is(&material, &Material::lava())
    {
        return false;
    }
    !blockBlocksFlow(world, x, y, z)
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockFlowing",
    m_parent: Some(&block_fluid::CLASS),
    updateTick,
    onBlockAdded: |this, world, x, y, z| {
        (block_fluid::CLASS.onBlockAdded)(this, world, x, y, z);
        if world.getBlockId(x, y, z) == this.m_blockID
        {
            world.scheduleBlockUpdate(x, y, z, this.m_blockID, this.tickRate());
        }
    },
    ..block_fluid::CLASS
};

pub fn create(id: i32, material: Rc<Material>) -> Rc<Block>
{
    block_fluid::construct(&CLASS, Box::new(FlowingExt::default()), id, material)
}
