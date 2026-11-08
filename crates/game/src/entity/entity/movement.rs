use crate::block::{Block, StepSound};
use crate::entity::Entity;
use crate::math::AxisAlignedBB;
use crate::math::math_helper::{self, floor_double};
use crate::world::World;
use jvm::array_list::JArrayList;
use jvm::throwable::OrNpe;
use std::rc::Rc;

fn collide(list: &JArrayList<Rc<AxisAlignedBB>>, bb: &AxisAlignedBB, axis: u8, value: f64) -> f64
{
    let mut value: f64 = value;
    for i in 0..list.size()
    {
        let other: Rc<AxisAlignedBB> = list.get(i);
        value = match axis
        {
            0 => other.calculateXOffset(bb, value),
            1 => other.calculateYOffset(bb, value),
            _ => other.calculateZOffset(bb, value),
        };
    }
    value
}

pub fn moveEntity(this: &Rc<Entity>, dx: f64, dy: f64, dz: f64)
{
    let mut dx: f64 = dx;
    let mut dy: f64 = dy;
    let mut dz: f64 = dz;
    let bb: &Rc<AxisAlignedBB> = &this.m_boundingBox;
    if this.m_noClip.get()
    {
        bb.offset(dx, dy, dz);
        this.m_posX.set((bb.minX() + bb.maxX()) / 2.0);
        this.m_posY.set(bb.minY() + this.m_yOffset.get() as f64 - this.m_ySize.get() as f64);
        this.m_posZ.set((bb.minZ() + bb.maxZ()) / 2.0);
        return;
    }

    let world: Rc<World> = this.world();
    this.m_ySize.set(this.m_ySize.get() * 0.4);
    let d0: f64 = this.posX();
    let d1: f64 = this.posZ();
    if this.m_isInWeb.get()
    {
        this.m_isInWeb.set(false);
        dx *= 0.25;
        dy *= 0.05f32 as f64;
        dz *= 0.25;
        this.m_motionX.set(0.0);
        this.m_motionY.set(0.0);
        this.m_motionZ.set(0.0);
    }

    let mut d2: f64 = dx;
    let d3: f64 = dy;
    let mut d4: f64 = dz;
    let axisalignedbb: Rc<AxisAlignedBB> = bb.copy();
    let flag: bool = this.m_onGround.get() && this.isSneaking();
    if flag
    {
        let d5: f64 = 0.05;
        while dx != 0.0 && world.getCollidingBoundingBoxes(this, &bb.getOffsetBoundingBox(dx, -1.0, 0.0)).isEmpty()
        {
            if dx < d5 && dx >= -d5
            {
                dx = 0.0;
            }
            else if dx > 0.0
            {
                dx -= d5;
            }
            else
            {
                dx += d5;
            }
            d2 = dx;
        }

        while dz != 0.0 && world.getCollidingBoundingBoxes(this, &bb.getOffsetBoundingBox(0.0, -1.0, dz)).isEmpty()
        {
            if dz < d5 && dz >= -d5
            {
                dz = 0.0;
            }
            else if dz > 0.0
            {
                dz -= d5;
            }
            else
            {
                dz += d5;
            }
            d4 = dz;
        }
    }

    let mut list: Rc<JArrayList<Rc<AxisAlignedBB>>> = world.getCollidingBoundingBoxes(this, &bb.addCoord(dx, dy, dz));
    dy = collide(&list, bb, 1, dy);
    bb.offset(0.0, dy, 0.0);
    if !this.m_immovable.get() && d3 != dy
    {
        dz = 0.0;
        dy = 0.0;
        dx = 0.0;
    }

    let flag1: bool = this.m_onGround.get() || d3 != dy && d3 < 0.0;
    dx = collide(&list, bb, 0, dx);
    bb.offset(dx, 0.0, 0.0);
    if !this.m_immovable.get() && d2 != dx
    {
        dz = 0.0;
        dy = 0.0;
        dx = 0.0;
    }

    dz = collide(&list, bb, 2, dz);
    bb.offset(0.0, 0.0, dz);
    if !this.m_immovable.get() && d4 != dz
    {
        dz = 0.0;
        dy = 0.0;
        dx = 0.0;
    }

    if this.m_stepHeight.get() > 0.0 && flag1 && (flag || this.m_ySize.get() < 0.05) && (d2 != dx || d4 != dz)
    {
        let d8: f64 = dx;
        let d6: f64 = dy;
        let d7: f64 = dz;
        dx = d2;
        dy = this.m_stepHeight.get() as f64;
        dz = d4;
        let axisalignedbb1: Rc<AxisAlignedBB> = bb.copy();
        bb.setBB(&axisalignedbb);
        list = world.getCollidingBoundingBoxes(this, &bb.addCoord(dx, dy, dz));
        dy = collide(&list, bb, 1, dy);
        bb.offset(0.0, dy, 0.0);
        if !this.m_immovable.get() && d3 != dy
        {
            dz = 0.0;
            dy = 0.0;
            dx = 0.0;
        }

        dx = collide(&list, bb, 0, dx);
        bb.offset(dx, 0.0, 0.0);
        if !this.m_immovable.get() && d2 != dx
        {
            dz = 0.0;
            dy = 0.0;
            dx = 0.0;
        }

        dz = collide(&list, bb, 2, dz);
        bb.offset(0.0, 0.0, dz);
        if !this.m_immovable.get() && d4 != dz
        {
            dz = 0.0;
            dy = 0.0;
            dx = 0.0;
        }

        if !this.m_immovable.get() && d3 != dy
        {
            dz = 0.0;
            dy = 0.0;
            dx = 0.0;
        }
        else
        {
            dy = -this.m_stepHeight.get() as f64;
            dy = collide(&list, bb, 1, dy);
            bb.offset(0.0, dy, 0.0);
        }

        if d8 * d8 + d7 * d7 >= dx * dx + dz * dz
        {
            dx = d8;
            dy = d6;
            dz = d7;
            bb.setBB(&axisalignedbb1);
        }
        else
        {
            let d11: f64 = bb.minY() - (bb.minY() as i32) as f64;
            if d11 > 0.0
            {
                this.m_ySize.set((this.m_ySize.get() as f64 + (d11 + 0.01)) as f32);
            }
        }
    }

    this.m_posX.set((bb.minX() + bb.maxX()) / 2.0);
    this.m_posY.set(bb.minY() + this.m_yOffset.get() as f64 - this.m_ySize.get() as f64);
    this.m_posZ.set((bb.minZ() + bb.maxZ()) / 2.0);
    this.m_isCollidedHorizontally.set(d2 != dx || d4 != dz);
    this.m_isCollidedVertically.set(d3 != dy);
    this.m_onGround.set(d3 != dy && d3 < 0.0);
    this.m_isCollided.set(this.m_isCollidedHorizontally.get() || this.m_isCollidedVertically.get());
    this.updateFallState(dy, this.m_onGround.get());
    if d2 != dx
    {
        this.m_motionX.set(0.0);
    }
    if d3 != dy
    {
        this.m_motionY.set(0.0);
    }
    if d4 != dz
    {
        this.m_motionZ.set(0.0);
    }

    let d9: f64 = this.posX() - d0;
    let d10: f64 = this.posZ() - d1;
    if this.canTriggerWalking() && !flag && this.m_ridingEntity.with(Option::is_none)
    {
        this.m_distanceWalkedModified.set(
            (this.m_distanceWalkedModified.get() as f64 + math_helper::sqrt_double(d9 * d9 + d10 * d10) as f64 * 0.6)
                as f32,
        );
        let k2: i32 = floor_double(this.posX());
        let k: i32 = floor_double(this.posY() - 0.2f32 as f64 - this.m_yOffset.get() as f64);
        let j3: i32 = floor_double(this.posZ());
        let mut k4: i32 = world.getBlockId(k2, k, j3);
        if world.getBlockId(k2, k - 1, j3) == Block::fence().m_blockID
        {
            k4 = world.getBlockId(k2, k - 1, j3);
        }

        if this.m_distanceWalkedModified.get() > this.m_nextStepDistance.get() as f32 && k4 > 0
        {
            this.m_nextStepDistance.set(this.m_nextStepDistance.get() + 1);
            let block: Rc<Block> = Block::byId(k4).orNpe();
            let mut stepsound: Rc<StepSound> = block.m_stepSound.get();
            if world.getBlockId(k2, k + 1, j3) == Block::snow().m_blockID
            {
                stepsound = Block::snow().m_stepSound.get();
                world.playSoundAtEntity(
                    this,
                    &stepsound.getStepping(),
                    stepsound.getVolume() * 0.15,
                    stepsound.getPitch(),
                );
            }
            else if !block.m_blockMaterial.getIsLiquid()
            {
                world.playSoundAtEntity(
                    this,
                    &stepsound.getStepping(),
                    stepsound.getVolume() * 0.15,
                    stepsound.getPitch(),
                );
            }
            block.onEntityWalking(&world, k2, k, j3, this);
        }
    }

    let l2: i32 = floor_double(bb.minX() + 0.001);
    let i3: i32 = floor_double(bb.minY() + 0.001);
    let k3: i32 = floor_double(bb.minZ() + 0.001);
    let l4: i32 = floor_double(bb.maxX() - 0.001);
    let i5: i32 = floor_double(bb.maxY() - 0.001);
    let i1: i32 = floor_double(bb.maxZ() - 0.001);
    if world.checkChunksExist(l2, i3, k3, l4, i5, i1)
    {
        for j1 in l2..=l4
        {
            for k1 in i3..=i5
            {
                for l1 in k3..=i1
                {
                    let i2: i32 = world.getBlockId(j1, k1, l1);
                    if i2 > 0
                    {
                        Block::byId(i2).orNpe().onEntityCollidedWithBlock(&world, j1, k1, l1, this);
                    }
                }
            }
        }
    }

    let flag2: bool = this.isWet();
    if world.isBoundingBoxBurning(&bb.contract(0.001, 0.001, 0.001))
    {
        this.dealFireDamage(1);
        if !flag2
        {
            this.m_fire.set(this.m_fire.get() + 1);
            if this.m_fire.get() == 0
            {
                this.m_fire.set(300);
            }
        }
    }
    else if this.m_fire.get() <= 0
    {
        this.m_fire.set(-this.m_fireResistance.get());
    }

    if flag2 && this.m_fire.get() > 0
    {
        let pitch: f32 = 1.6 + (this.m_rand.nextFloat() - this.m_rand.nextFloat()) * 0.4;
        world.playSoundAtEntity(this, "random.fizz", 0.7, pitch);
        this.m_fire.set(-this.m_fireResistance.get());
    }
}

impl Entity {}

pub fn setPositionAndRotation2(this: &Rc<Entity>, x: f64, y: f64, z: f64, yaw: f32, pitch: f32, _steps: i32)
{
    let mut y: f64 = y;
    this.setPosition(x, y, z);
    this.setRotation(yaw, pitch);
    let list: Rc<JArrayList<Rc<AxisAlignedBB>>> =
        this.world().getCollidingBoundingBoxes(this, &this.m_boundingBox.contract(0.03125, 0.0, 0.03125));
    if !list.isEmpty()
    {
        let mut d0: f64 = 0.0;
        for i in 0..list.size()
        {
            let axisalignedbb: Rc<AxisAlignedBB> = list.get(i);
            if axisalignedbb.maxY() > d0
            {
                d0 = axisalignedbb.maxY();
            }
        }
        y += d0 - this.m_boundingBox.minY();
        this.setPosition(x, y, z);
    }
}

pub fn pushOutOfBlocks(this: &Rc<Entity>, x: f64, y: f64, z: f64) -> bool
{
    let i: i32 = floor_double(x);
    let j: i32 = floor_double(y);
    let k: i32 = floor_double(z);
    let d0: f64 = x - i as f64;
    let d1: f64 = y - j as f64;
    let d2: f64 = z - k as f64;
    let world: Rc<World> = this.world();
    if world.isBlockNormalCube(i, j, k)
    {
        let flag: bool = !world.isBlockNormalCube(i - 1, j, k);
        let flag1: bool = !world.isBlockNormalCube(i + 1, j, k);
        let flag2: bool = !world.isBlockNormalCube(i, j - 1, k);
        let flag3: bool = !world.isBlockNormalCube(i, j + 1, k);
        let flag4: bool = !world.isBlockNormalCube(i, j, k - 1);
        let flag5: bool = !world.isBlockNormalCube(i, j, k + 1);
        let mut b0: i8 = -1;
        let mut d3: f64 = 9999.0;
        if flag && d0 < d3
        {
            d3 = d0;
            b0 = 0;
        }
        if flag1 && 1.0 - d0 < d3
        {
            d3 = 1.0 - d0;
            b0 = 1;
        }
        if flag2 && d1 < d3
        {
            d3 = d1;
            b0 = 2;
        }
        if flag3 && 1.0 - d1 < d3
        {
            d3 = 1.0 - d1;
            b0 = 3;
        }
        if flag4 && d2 < d3
        {
            d3 = d2;
            b0 = 4;
        }
        if flag5 && 1.0 - d2 < d3
        {
            b0 = 5;
        }

        let f: f32 = this.m_rand.nextFloat() * 0.2 + 0.1;
        match b0
        {
            0 => this.m_motionX.set(-f as f64),
            1 => this.m_motionX.set(f as f64),
            2 => this.m_motionY.set(-f as f64),
            3 => this.m_motionY.set(f as f64),
            4 => this.m_motionZ.set(-f as f64),
            5 => this.m_motionZ.set(f as f64),
            _ => (),
        }
    }
    false
}

pub fn updateFallState(this: &Rc<Entity>, dy: f64, onGround: bool)
{
    if onGround
    {
        if this.m_fallDistance.get() > 0.0
        {
            this.fall(this.m_fallDistance.get());
            this.m_fallDistance.set(0.0);
        }
    }
    else if dy < 0.0
    {
        this.m_fallDistance.set((this.m_fallDistance.get() as f64 - dy) as f32);
    }
}
