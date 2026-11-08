use crate::block::Block;
use crate::world::World;
use jvm::throwable::OrNpe;
use std::rc::Rc;

impl World
{
    pub fn isBlockProvidingPowerTo(self: &Rc<Self>, x: i32, y: i32, z: i32, dir: i32) -> bool
    {
        let i: i32 = self.getBlockId(x, y, z);
        if i == 0
        {
            false
        }
        else
        {
            Block::byId(i).orNpe().isIndirectlyPoweringTo(self, x, y, z, dir)
        }
    }

    pub fn isBlockGettingPowered(self: &Rc<Self>, x: i32, y: i32, z: i32) -> bool
    {
        self.isBlockProvidingPowerTo(x, y - 1, z, 0)
            || self.isBlockProvidingPowerTo(x, y + 1, z, 1)
            || self.isBlockProvidingPowerTo(x, y, z - 1, 2)
            || self.isBlockProvidingPowerTo(x, y, z + 1, 3)
            || self.isBlockProvidingPowerTo(x - 1, y, z, 4)
            || self.isBlockProvidingPowerTo(x + 1, y, z, 5)
    }

    pub fn isBlockIndirectlyProvidingPowerTo(self: &Rc<Self>, x: i32, y: i32, z: i32, dir: i32) -> bool
    {
        if self.isBlockNormalCube(x, y, z)
        {
            return self.isBlockGettingPowered(x, y, z);
        }
        let i: i32 = self.getBlockId(x, y, z);
        if i == 0
        {
            false
        }
        else
        {
            Block::byId(i).orNpe().isPoweringTo(&**self, x, y, z, dir)
        }
    }

    pub fn isBlockIndirectlyGettingPowered(self: &Rc<Self>, x: i32, y: i32, z: i32) -> bool
    {
        self.isBlockIndirectlyProvidingPowerTo(x, y - 1, z, 0)
            || self.isBlockIndirectlyProvidingPowerTo(x, y + 1, z, 1)
            || self.isBlockIndirectlyProvidingPowerTo(x, y, z - 1, 2)
            || self.isBlockIndirectlyProvidingPowerTo(x, y, z + 1, 3)
            || self.isBlockIndirectlyProvidingPowerTo(x - 1, y, z, 4)
            || self.isBlockIndirectlyProvidingPowerTo(x + 1, y, z, 5)
    }
}
