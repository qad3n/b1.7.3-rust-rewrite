use crate::block::{Block, BlockClass, Material, block_container};
use crate::entity::{Entity, entity_item};
use crate::item::ItemStack;
use crate::port::tile_entity;
use crate::tileentity::{TILE_ENTITY, TileEntity, TileEntityClass};
use crate::world::World;
use std::cell::Cell;
use std::rc::Rc;

#[derive(Default)]
pub struct RecordPlayerExt
{
    pub m_record: Cell<i32>,
}

pub const TILE_ENTITY_RECORD_PLAYER: TileEntityClass = TileEntityClass {
    m_name: "BlockJukeBox$TileEntityRecordPlayer",
    m_parent: Some(&TILE_ENTITY),
    readFromNBT: |this, nbt| {
        (TILE_ENTITY.readFromNBT)(this, nbt);
        this.ext::<RecordPlayerExt>().m_record.set(nbt.getInteger("Record"));
    },
    writeToNBT: |this, nbt| {
        (TILE_ENTITY.writeToNBT)(this, nbt);
        let record: i32 = this.ext::<RecordPlayerExt>().m_record.get();
        if record > 0
        {
            nbt.setInteger("Record", record);
        }
    },
    ..TILE_ENTITY
};

pub fn newRecordPlayer() -> Rc<TileEntity>
{
    TileEntity::new(&TILE_ENTITY_RECORD_PLAYER, Box::new(RecordPlayerExt::default()))
}

fn recordPlayerAt(world: &Rc<World>, x: i32, y: i32, z: i32) -> Rc<TileEntity>
{
    // port cast is the java tile entity cast and throws ClassCastException on a wrong class
    tile_entity::cast(world.getBlockTileEntity(x, y, z), "BlockJukeBox$TileEntityRecordPlayer")
}

pub fn ejectRecord(world: &Rc<World>, x: i32, y: i32, z: i32, record: i32)
{
    if world.m_multiplayerWorld.get()
    {
        return;
    }
    let tileEntityRecordPlayer: Rc<TileEntity> = recordPlayerAt(world, x, y, z);
    tileEntityRecordPlayer.ext::<RecordPlayerExt>().m_record.set(record);
    tileEntityRecordPlayer.onInventoryChanged();
    world.setBlockMetadataWithNotify(x, y, z, 1);
}

pub fn removeRecord(world: &Rc<World>, x: i32, y: i32, z: i32)
{
    if world.m_multiplayerWorld.get()
    {
        return;
    }
    let tileEntityRecordPlayer: Rc<TileEntity> = recordPlayerAt(world, x, y, z);
    let n: i32 = tileEntityRecordPlayer.ext::<RecordPlayerExt>().m_record.get();
    if n == 0
    {
        return;
    }
    world.doEvent(1005, x, y, z, 0);
    world.playRecord(None, x, y, z);
    tileEntityRecordPlayer.ext::<RecordPlayerExt>().m_record.set(0);
    tileEntityRecordPlayer.onInventoryChanged();
    world.setBlockMetadataWithNotify(x, y, z, 0);
    let n2: i32 = n;
    let f: f32 = 0.7;
    let d: f64 = (world.m_rand.nextFloat() * f) as f64 + (1.0 - f) as f64 * 0.5;
    let d2: f64 = (world.m_rand.nextFloat() * f) as f64 + (1.0 - f) as f64 * 0.2 + 0.6;
    let d3: f64 = (world.m_rand.nextFloat() * f) as f64 + (1.0 - f) as f64 * 0.5;
    let entityItem: Rc<Entity> =
        entity_item::create(world, x as f64 + d, y as f64 + d2, z as f64 + d3, ItemStack::withDamage(n2, 1, 0));
    entity_item::fields(&entityItem).m_delayBeforeCanPickup.set(10);
    world.entityJoinedWorld(&entityItem);
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockJukeBox",
    m_parent: Some(&block_container::CLASS),
    getBlockTextureFromSide: |this, face| this.m_blockIndexInTexture.get() + if face == 1 { 1 } else { 0 },
    blockActivated: |_, world, x, y, z, _| {
        if world.getBlockMetadata(x, y, z) == 0
        {
            return false;
        }
        removeRecord(world, x, y, z);
        true
    },
    onBlockRemoval: |this, world, x, y, z| {
        removeRecord(world, x, y, z);
        (block_container::CLASS.onBlockRemoval)(this, world, x, y, z);
    },
    dropBlockAsItemWithChance: |this, world, x, y, z, metadata, luck| {
        if world.m_multiplayerWorld.get()
        {
            return;
        }
        (block_container::CLASS.dropBlockAsItemWithChance)(this, world, x, y, z, metadata, luck);
    },
    getBlockEntity: |_| newRecordPlayer(),
    ..block_container::CLASS
};

pub fn create(id: i32, sprite: i32) -> Rc<Block>
{
    block_container::constructWithSprite(&CLASS, Box::new(()), id, sprite, Material::wood())
}
