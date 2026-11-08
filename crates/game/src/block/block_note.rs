use crate::block::{Block, BlockClass, Material, block_container};
use crate::port::tile_entity;
use crate::tileentity::{TileEntity, tile_entity_note};
use jvm::math;
use jvm::throwable::OrNpe;
use std::rc::Rc;

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockNote",
    m_parent: Some(&block_container::CLASS),
    getBlockTextureFromSide: |this, _| this.m_blockIndexInTexture.get(),
    onNeighborBlockChange: |_, world, x, y, z, neighborBlock| {
        if neighborBlock > 0 && Block::byId(neighborBlock).orNpe().canProvidePower()
        {
            let bl: bool = world.isBlockGettingPowered(x, y, z);
            // port cast is the java tile entity cast and throws ClassCastException on a wrong class
            let tileEntityNote: Rc<TileEntity> = tile_entity::cast(world.getBlockTileEntity(x, y, z), "TileEntityNote");
            let note: &tile_entity_note::NoteExt = tileEntityNote.ext();
            if note.m_previousRedstoneState.get() != bl
            {
                if bl
                {
                    tile_entity_note::triggerNote(&tileEntityNote, world, x, y, z);
                }
                note.m_previousRedstoneState.set(bl);
            }
        }
    },
    blockActivated: |_, world, x, y, z, _| {
        if world.m_multiplayerWorld.get()
        {
            return true;
        }
        // port cast is the java tile entity cast and throws ClassCastException on a wrong class
        let tileEntityNote: Rc<TileEntity> = tile_entity::cast(world.getBlockTileEntity(x, y, z), "TileEntityNote");
        tile_entity_note::changePitch(&tileEntityNote);
        tile_entity_note::triggerNote(&tileEntityNote, world, x, y, z);
        true
    },
    onBlockClicked: |_, world, x, y, z, _| {
        if world.m_multiplayerWorld.get()
        {
            return;
        }
        // port cast is the java tile entity cast and throws ClassCastException on a wrong class
        let tileEntityNote: Rc<TileEntity> = tile_entity::cast(world.getBlockTileEntity(x, y, z), "TileEntityNote");
        tile_entity_note::triggerNote(&tileEntityNote, world, x, y, z);
    },
    getBlockEntity: |_| tile_entity_note::create(),
    playBlock: |_, world, x, y, z, kind, data| {
        let f: f32 = math::pow(2.0, (data - 12) as f64 / 12.0) as f32;
        let mut string: &str = "harp";
        if kind == 1
        {
            string = "bd";
        }
        if kind == 2
        {
            string = "snare";
        }
        if kind == 3
        {
            string = "hat";
        }
        if kind == 4
        {
            string = "bassattack";
        }
        world.playSoundEffect(x as f64 + 0.5, y as f64 + 0.5, z as f64 + 0.5, &format!("note.{string}"), 3.0, f);
        world.spawnParticle("note", x as f64 + 0.5, y as f64 + 1.2, z as f64 + 0.5, data as f64 / 24.0, 0.0, 0.0);
    },
    ..block_container::CLASS
};

pub fn create(id: i32) -> Rc<Block>
{
    block_container::constructWithSprite(&CLASS, Box::new(()), id, 74, Material::wood())
}
