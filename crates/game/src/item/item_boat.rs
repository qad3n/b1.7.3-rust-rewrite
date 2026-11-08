use crate::block::Block;
use crate::entity::{Entity, entity_boat};
use crate::item::{ITEM, Item, ItemClass, ItemStack};
use crate::math::{EnumMovingObjectType, MovingObjectPosition, Vec3D, math_helper};
use crate::world::World;
use std::rc::Rc;

pub fn lookRay(
    world: &Rc<World>,
    player: &Rc<Entity>,
    allowLiquids: bool,
) -> (Option<MovingObjectPosition>, f64, f64, f64)
{
    let f: f32 = 1.0;
    let f1: f32 =
        player.m_prevRotationPitch.get() + (player.m_rotationPitch.get() - player.m_prevRotationPitch.get()) * f;
    let f2: f32 = player.m_prevRotationYaw.get() + (player.m_rotationYaw.get() - player.m_prevRotationYaw.get()) * f;
    let d0: f64 = player.m_prevPosX.get() + (player.m_posX.get() - player.m_prevPosX.get()) * f as f64;
    let d1: f64 = player.m_prevPosY.get() + (player.m_posY.get() - player.m_prevPosY.get()) * f as f64 + 1.62
        - player.m_yOffset.get() as f64;
    let d2: f64 = player.m_prevPosZ.get() + (player.m_posZ.get() - player.m_prevPosZ.get()) * f as f64;
    let vec3d: Rc<Vec3D> = Vec3D::createVector(d0, d1, d2);
    let f3: f32 = math_helper::cos(-f2 * (std::f64::consts::PI / 180.0) as f32 - std::f64::consts::PI as f32);
    let f4: f32 = math_helper::sin(-f2 * (std::f64::consts::PI / 180.0) as f32 - std::f64::consts::PI as f32);
    let f5: f32 = -math_helper::cos(-f1 * (std::f64::consts::PI / 180.0) as f32);
    let f6: f32 = math_helper::sin(-f1 * (std::f64::consts::PI / 180.0) as f32);
    let f7: f32 = f4 * f5;
    let f8: f32 = f6;
    let f9: f32 = f3 * f5;
    let d3: f64 = 5.0;
    let vec3d1: Rc<Vec3D> = vec3d.addVector(f7 as f64 * d3, f8 as f64 * d3, f9 as f64 * d3);
    (world.rayTraceBlocks_do(&vec3d, &vec3d1, allowLiquids), d0, d1, d2)
}

fn onItemRightClick(
    _this: &Rc<Item>,
    stack: &Rc<ItemStack>,
    world: &Rc<World>,
    player: &Rc<Entity>,
) -> Option<Rc<ItemStack>>
{
    let (movingobjectposition, _, _, _): (Option<MovingObjectPosition>, f64, f64, f64) = lookRay(world, player, true);
    let Some(movingobjectposition) = movingobjectposition
    else
    {
        return Some(Rc::clone(stack));
    };
    if movingobjectposition.m_typeOfHit == EnumMovingObjectType::TILE
    {
        let i: i32 = movingobjectposition.m_blockX;
        let mut j: i32 = movingobjectposition.m_blockY;
        let k: i32 = movingobjectposition.m_blockZ;
        if !world.m_multiplayerWorld.get()
        {
            if world.getBlockId(i, j, k) == Block::snow().m_blockID
            {
                j -= 1;
            }
            world.entityJoinedWorld(&entity_boat::createAt(
                world,
                (i as f32 + 0.5) as f64,
                (j as f32 + 1.0) as f64,
                (k as f32 + 0.5) as f64,
            ));
        }
        stack.m_stackSize.set(stack.m_stackSize.get() - 1);
    }
    Some(Rc::clone(stack))
}

pub const CLASS: ItemClass = ItemClass { m_name: "ItemBoat", m_parent: Some(&ITEM), onItemRightClick, ..ITEM };

pub fn create(id: i32) -> Rc<Item>
{
    let item: Rc<Item> = Item::construct(&CLASS, Box::new(()), id);
    item.m_maxStackSize.set(1);
    item
}
