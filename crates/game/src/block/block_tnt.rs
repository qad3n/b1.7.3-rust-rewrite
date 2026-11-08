use crate::block::{BLOCK, Block, BlockClass, Material};
use crate::entity::Entity;
use crate::item::{Item, ItemStack};
use jvm::throwable::OrNpe;
use std::cell::Cell;
use std::rc::Rc;

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockTNT",
    m_parent: Some(&BLOCK),
    getBlockTextureFromSide: |this, face| match face
    {
        0 => this.m_blockIndexInTexture.get() + 2,
        1 => this.m_blockIndexInTexture.get() + 1,
        _ => this.m_blockIndexInTexture.get(),
    },
    onBlockAdded: |this, world, x, y, z| {
        (BLOCK.onBlockAdded)(this, world, x, y, z);
        if world.isBlockIndirectlyGettingPowered(x, y, z)
        {
            this.onBlockDestroyedByPlayer(world, x, y, z, 1);
            world.setBlockWithNotify(x, y, z, 0);
        }
    },
    onNeighborBlockChange: |this, world, x, y, z, neighborBlock| {
        if neighborBlock > 0
            && Block::byId(neighborBlock).orNpe().canProvidePower()
            && world.isBlockIndirectlyGettingPowered(x, y, z)
        {
            this.onBlockDestroyedByPlayer(world, x, y, z, 1);
            world.setBlockWithNotify(x, y, z, 0);
        }
    },
    quantityDropped: |_, _| 0,
    onBlockDestroyedByExplosion: |_, world, x, y, z| {
        let entity: Rc<Entity> = crate::entity::entity_tnt_primed::create(
            world,
            (x as f32 + 0.5) as f64,
            (y as f32 + 0.5) as f64,
            (z as f32 + 0.5) as f64,
        );
        let fuse: &Cell<i32> = &crate::entity::entity_tnt_primed::fields(&entity).m_fuse;
        fuse.set(world.m_rand.nextIntBounded(fuse.get() / 4) + fuse.get() / 8);
        world.entityJoinedWorld(&entity);
    },
    onBlockDestroyedByPlayer: |this, world, x, y, z, metadata| {
        if world.m_multiplayerWorld.get()
        {
            return;
        }
        if (metadata & 1) == 0
        {
            this.dropBlockAsItem_do(world, x, y, z, ItemStack::withDamage(Block::tnt().m_blockID, 1, 0));
        }
        else
        {
            let entity: Rc<Entity> = crate::entity::entity_tnt_primed::create(
                world,
                (x as f32 + 0.5) as f64,
                (y as f32 + 0.5) as f64,
                (z as f32 + 0.5) as f64,
            );
            world.entityJoinedWorld(&entity);
            world.playSoundAtEntity(&entity, "random.fuse", 1.0, 1.0);
        }
    },
    onBlockClicked: |this, world, x, y, z, player| {
        if crate::entity::entity_player::getCurrentEquippedItem(player)
            .is_some_and(|stack| stack.m_itemID.get() == Item::flintAndSteel().m_shiftedIndex)
        {
            world.setBlockMetadata(x, y, z, 1);
        }
        (BLOCK.onBlockClicked)(this, world, x, y, z, player);
    },
    blockActivated: |this, world, x, y, z, player| (BLOCK.blockActivated)(this, world, x, y, z, player),
    ..BLOCK
};

pub fn create(id: i32, sprite: i32) -> Rc<Block>
{
    Block::constructWithSprite(&CLASS, Box::new(()), id, sprite, Material::tnt())
}
