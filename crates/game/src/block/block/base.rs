use crate::block::Block;
use crate::entity::Entity;
use crate::item::ItemStack;
use crate::math::{AxisAlignedBB, MovingObjectPosition, Vec3D};
use crate::world::{IBlockAccess, World};
use jvm::array_list::JArrayList;
use jvm::throwable::OrNpe;
use std::rc::Rc;

pub fn boundsAt(this: &Block, x: i32, y: i32, z: i32) -> Rc<AxisAlignedBB>
{
    AxisAlignedBB::getBoundingBoxFromPool(
        x as f64 + this.m_minX.get(),
        y as f64 + this.m_minY.get(),
        z as f64 + this.m_minZ.get(),
        x as f64 + this.m_maxX.get(),
        y as f64 + this.m_maxY.get(),
        z as f64 + this.m_maxZ.get(),
    )
}

pub fn shouldSideBeRendered(this: &Rc<Block>, world: &dyn IBlockAccess, x: i32, y: i32, z: i32, face: i32) -> bool
{
    if face == 0 && this.m_minY.get() > 0.0
    {
        return true;
    }
    if face == 1 && this.m_maxY.get() < 1.0
    {
        return true;
    }
    if face == 2 && this.m_minZ.get() > 0.0
    {
        return true;
    }
    if face == 3 && this.m_maxZ.get() < 1.0
    {
        return true;
    }
    if face == 4 && this.m_minX.get() > 0.0
    {
        return true;
    }
    if face == 5 && this.m_maxX.get() < 1.0
    {
        return true;
    }
    !world.isBlockOpaqueCube(x, y, z)
}

pub fn getCollidingBoundingBoxes(
    this: &Rc<Block>,
    world: &Rc<World>,
    x: i32,
    y: i32,
    z: i32,
    shape: &AxisAlignedBB,
    collisions: &JArrayList<Rc<AxisAlignedBB>>,
)
{
    if let Some(bb) = this.getCollisionBoundingBoxFromPool(world, x, y, z)
        && shape.intersectsWith(&bb)
    {
        collisions.add(bb);
    }
}

pub fn canPlaceBlockAt(_this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32) -> bool
{
    let n: i32 = world.getBlockId(x, y, z);
    n == 0 || Block::byId(n).orNpe().m_blockMaterial.getIsGroundCover()
}

pub fn dropBlockAsItemWithChance(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, metadata: i32, luck: f32)
{
    if world.m_multiplayerWorld.get()
    {
        return;
    }
    let n: i32 = this.quantityDropped(&world.m_rand);
    for _ in 0..n
    {
        if world.m_rand.nextFloat() > luck
        {
            continue;
        }
        let n2: i32 = this.idDropped(metadata, &world.m_rand);
        if n2 <= 0
        {
            continue;
        }
        dropBlockAsItem_do(world, x, y, z, ItemStack::withDamage(n2, 1, this.damageDropped(metadata)));
    }
}

pub fn dropBlockAsItem_do(world: &Rc<World>, x: i32, y: i32, z: i32, item: Rc<ItemStack>)
{
    if world.m_multiplayerWorld.get()
    {
        return;
    }
    let f: f32 = 0.7;
    let d: f64 = (world.m_rand.nextFloat() * f) as f64 + (1.0 - f) as f64 * 0.5;
    let d2: f64 = (world.m_rand.nextFloat() * f) as f64 + (1.0 - f) as f64 * 0.5;
    let d3: f64 = (world.m_rand.nextFloat() * f) as f64 + (1.0 - f) as f64 * 0.5;
    let entityItem: Rc<Entity> =
        crate::entity::entity_item::create(world, x as f64 + d, y as f64 + d2, z as f64 + d3, item);
    crate::entity::entity_item::fields(&entityItem).m_delayBeforeCanPickup.set(10);
    world.entityJoinedWorld(&entityItem);
}

impl Block
{
    pub fn dropBlockAsItem_do(&self, world: &Rc<World>, x: i32, y: i32, z: i32, item: Rc<ItemStack>)
    {
        dropBlockAsItem_do(world, x, y, z, item);
    }

    pub fn getBlockBrightnessBase(&self, world: &dyn IBlockAccess, x: i32, y: i32, z: i32) -> f32
    {
        world.getBrightness(x, y, z, Block::lightValue(self.m_blockID))
    }

    pub fn blockStrength(self: &Rc<Self>, player: &Rc<Entity>) -> f32
    {
        if self.m_blockHardness.get() < 0.0
        {
            return 0.0;
        }
        if !crate::entity::entity_player::canHarvestBlock(player, self)
        {
            return 1.0 / self.m_blockHardness.get() / 100.0;
        }
        crate::entity::entity_player::getCurrentPlayerStrVsBlock(player, self) / self.m_blockHardness.get() / 30.0
    }

    fn isVecInsideYZBounds(&self, pos: Option<&Vec3D>) -> bool
    {
        let Some(pos) = pos
        else
        {
            return false;
        };
        pos.y() >= self.m_minY.get()
            && pos.y() <= self.m_maxY.get()
            && pos.z() >= self.m_minZ.get()
            && pos.z() <= self.m_maxZ.get()
    }

    fn isVecInsideXZBounds(&self, pos: Option<&Vec3D>) -> bool
    {
        let Some(pos) = pos
        else
        {
            return false;
        };
        pos.x() >= self.m_minX.get()
            && pos.x() <= self.m_maxX.get()
            && pos.z() >= self.m_minZ.get()
            && pos.z() <= self.m_maxZ.get()
    }

    fn isVecInsideXYBounds(&self, pos: Option<&Vec3D>) -> bool
    {
        let Some(pos) = pos
        else
        {
            return false;
        };
        pos.x() >= self.m_minX.get()
            && pos.x() <= self.m_maxX.get()
            && pos.y() >= self.m_minY.get()
            && pos.y() <= self.m_maxY.get()
    }
}

fn sameVec(a: &Option<Rc<Vec3D>>, b: &Option<Rc<Vec3D>>) -> bool
{
    match (a, b)
    {
        (Some(a), Some(b)) => Rc::ptr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}

pub fn collisionRayTrace(
    this: &Rc<Block>,
    world: &Rc<World>,
    x: i32,
    y: i32,
    z: i32,
    from: &Vec3D,
    to: &Vec3D,
) -> Option<MovingObjectPosition>
{
    this.setBlockBoundsBasedOnState(&**world, x, y, z);
    let from: Rc<Vec3D> = from.addVector(-x as f64, -y as f64, -z as f64);
    let to: Rc<Vec3D> = to.addVector(-x as f64, -y as f64, -z as f64);
    let mut vec3D: Option<Rc<Vec3D>> = from.getIntermediateWithXValue(&to, this.m_minX.get());
    let mut vec3D2: Option<Rc<Vec3D>> = from.getIntermediateWithXValue(&to, this.m_maxX.get());
    let mut vec3D3: Option<Rc<Vec3D>> = from.getIntermediateWithYValue(&to, this.m_minY.get());
    let mut vec3D4: Option<Rc<Vec3D>> = from.getIntermediateWithYValue(&to, this.m_maxY.get());
    let mut vec3D5: Option<Rc<Vec3D>> = from.getIntermediateWithZValue(&to, this.m_minZ.get());
    let mut vec3D6: Option<Rc<Vec3D>> = from.getIntermediateWithZValue(&to, this.m_maxZ.get());
    if !this.isVecInsideYZBounds(vec3D.as_deref())
    {
        vec3D = None;
    }
    if !this.isVecInsideYZBounds(vec3D2.as_deref())
    {
        vec3D2 = None;
    }
    if !this.isVecInsideXZBounds(vec3D3.as_deref())
    {
        vec3D3 = None;
    }
    if !this.isVecInsideXZBounds(vec3D4.as_deref())
    {
        vec3D4 = None;
    }
    if !this.isVecInsideXYBounds(vec3D5.as_deref())
    {
        vec3D5 = None;
    }
    if !this.isVecInsideXYBounds(vec3D6.as_deref())
    {
        vec3D6 = None;
    }

    let mut vec3D7: Option<Rc<Vec3D>> = None;
    for candidate in [&vec3D, &vec3D2, &vec3D3, &vec3D4, &vec3D5, &vec3D6]
    {
        if let Some(candidate) = candidate
        {
            let closer: bool = match &vec3D7
            {
                None => true,
                Some(best) => from.distanceTo(candidate) < from.distanceTo(best),
            };
            if closer
            {
                vec3D7 = Some(Rc::clone(candidate));
            }
        }
    }

    let hit: Rc<Vec3D> = vec3D7.clone()?;
    let mut n: i32 = -1;
    if sameVec(&vec3D7, &vec3D)
    {
        n = 4;
    }
    if sameVec(&vec3D7, &vec3D2)
    {
        n = 5;
    }
    if sameVec(&vec3D7, &vec3D3)
    {
        n = 0;
    }
    if sameVec(&vec3D7, &vec3D4)
    {
        n = 1;
    }
    if sameVec(&vec3D7, &vec3D5)
    {
        n = 2;
    }
    if sameVec(&vec3D7, &vec3D6)
    {
        n = 3;
    }
    Some(MovingObjectPosition::newTile(x, y, z, n, &hit.addVector(x as f64, y as f64, z as f64)))
}

pub fn harvestBlock(this: &Rc<Block>, world: &Rc<World>, player: &Rc<Entity>, x: i32, y: i32, z: i32, metadata: i32)
{
    player.addStat(crate::stats::StatList::mineBlockStat(this.m_blockID).as_ref(), 1);
    this.dropBlockAsItem(world, x, y, z, metadata);
}
