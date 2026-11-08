use crate::block::{Block, block_piston_moving};
use crate::entity::Entity;
use crate::tileentity::{TILE_ENTITY, TileEntity, TileEntityClass};
use crate::world::World;
use common::world::piston_block_textures::{X_OFFSET, Y_OFFSET, Z_OFFSET};
use jvm::array_list::{JArrayList, ListIterator};
use std::cell::Cell;
use std::rc::Rc;

thread_local! {
    static MOVED_ENTITIES: JArrayList<Rc<Entity>> = JArrayList::new();
}

#[derive(Default)]
pub struct PistonExt
{
    m_storedBlockID: Cell<i32>,
    m_storedMetadata: Cell<i32>,
    m_facing: Cell<i32>,
    m_extending: Cell<bool>,
    m_source: Cell<bool>,
    m_progress: Cell<f32>,
    m_lastProgress: Cell<f32>,
}

fn piston(this: &TileEntity) -> &PistonExt { this.ext::<PistonExt>() }

pub fn getStoredBlockID(this: &TileEntity) -> i32 { piston(this).m_storedBlockID.get() }

pub fn isExtending(this: &TileEntity) -> bool { piston(this).m_extending.get() }

pub fn getFacing(this: &TileEntity) -> i32 { piston(this).m_facing.get() }

pub fn isSource(this: &TileEntity) -> bool { piston(this).m_source.get() }

pub fn getProgress(this: &TileEntity, mut tickDelta: f32) -> f32
{
    if tickDelta > 1.0
    {
        tickDelta = 1.0;
    }
    let state: &PistonExt = piston(this);
    state.m_lastProgress.get() + (state.m_progress.get() - state.m_lastProgress.get()) * tickDelta
}

fn renderOffset(this: &TileEntity, tickDelta: f32, offset: &[i32; 6]) -> f32
{
    let facing: usize = getFacing(this) as usize;
    if isExtending(this)
    {
        return (getProgress(this, tickDelta) - 1.0) * offset[facing] as f32;
    }
    (1.0 - getProgress(this, tickDelta)) * offset[facing] as f32
}

pub fn getRenderOffsetX(this: &TileEntity, tickDelta: f32) -> f32 { renderOffset(this, tickDelta, &X_OFFSET) }

pub fn getRenderOffsetY(this: &TileEntity, tickDelta: f32) -> f32 { renderOffset(this, tickDelta, &Y_OFFSET) }

pub fn getRenderOffsetZ(this: &TileEntity, tickDelta: f32) -> f32 { renderOffset(this, tickDelta, &Z_OFFSET) }

fn moveEntities(this: &TileEntity, mut progress: f32, amount: f32)
{
    let state: &PistonExt = piston(this);
    if !state.m_extending.get()
    {
        progress -= 1.0;
    }
    else
    {
        progress = 1.0 - progress;
    }
    let world: Rc<World> = this.world();
    let facing: usize = state.m_facing.get() as usize;
    let Some(axisalignedbb) = block_piston_moving::getCollisionShape(
        &Block::pistonMoving(),
        &world,
        this.m_xCoord.get(),
        this.m_yCoord.get(),
        this.m_zCoord.get(),
        state.m_storedBlockID.get(),
        progress,
        state.m_facing.get(),
    )
    else
    {
        return;
    };
    let list: Rc<JArrayList<Rc<Entity>>> = world.getEntitiesWithinAABBExcludingEntity(None, &axisalignedbb);
    if list.isEmpty()
    {
        return;
    }
    MOVED_ENTITIES.with(|movedEntities| {
        movedEntities.addAll(&list.toVec());
        let mut iterator: ListIterator = movedEntities.iterator();
        while iterator.hasNext(movedEntities)
        {
            let entity: Rc<Entity> = iterator.next(movedEntities);
            entity.moveEntity(
                (amount * X_OFFSET[facing] as f32) as f64,
                (amount * Y_OFFSET[facing] as f32) as f64,
                (amount * Z_OFFSET[facing] as f32) as f64,
            );
        }
        movedEntities.clear();
    });
}

fn settle(this: &Rc<TileEntity>)
{
    let world: Rc<World> = this.world();
    world.removeBlockTileEntity(this.m_xCoord.get(), this.m_yCoord.get(), this.m_zCoord.get());
    this.markRemoved();
    if world.getBlockId(this.m_xCoord.get(), this.m_yCoord.get(), this.m_zCoord.get())
        == Block::pistonMoving().m_blockID
    {
        world.setBlockAndMetadataWithNotify(
            this.m_xCoord.get(),
            this.m_yCoord.get(),
            this.m_zCoord.get(),
            getStoredBlockID(this),
            piston(this).m_storedMetadata.get(),
        );
    }
}

pub fn finish(this: &Rc<TileEntity>)
{
    let state: &PistonExt = piston(this);
    if state.m_lastProgress.get() < 1.0
    {
        state.m_progress.set(1.0);
        state.m_lastProgress.set(1.0);
        settle(this);
    }
}

fn updateEntity(this: &Rc<TileEntity>)
{
    let state: &PistonExt = piston(this);
    state.m_lastProgress.set(state.m_progress.get());
    if state.m_lastProgress.get() >= 1.0
    {
        moveEntities(this, 1.0, 0.25);
        settle(this);
        return;
    }
    state.m_progress.set(state.m_progress.get() + 0.5);
    if state.m_progress.get() >= 1.0
    {
        state.m_progress.set(1.0);
    }
    if state.m_extending.get()
    {
        moveEntities(this, state.m_progress.get(), state.m_progress.get() - state.m_lastProgress.get() + 0.0625);
    }
}

pub const CLASS: TileEntityClass = TileEntityClass {
    m_name: "TileEntityPiston",
    m_parent: Some(&TILE_ENTITY),
    getBlockMetadata: |this| piston(this).m_storedMetadata.get(),
    updateEntity,
    readFromNBT: |this, nbt| {
        (TILE_ENTITY.readFromNBT)(this, nbt);
        let state: &PistonExt = piston(this);
        state.m_storedBlockID.set(nbt.getInteger("blockId"));
        state.m_storedMetadata.set(nbt.getInteger("blockData"));
        state.m_facing.set(nbt.getInteger("facing"));
        let progress: f32 = nbt.getFloat("progress");
        state.m_progress.set(progress);
        state.m_lastProgress.set(progress);
        state.m_extending.set(nbt.getBoolean("extending"));
    },
    writeToNBT: |this, nbt| {
        (TILE_ENTITY.writeToNBT)(this, nbt);
        let state: &PistonExt = piston(this);
        nbt.setInteger("blockId", state.m_storedBlockID.get());
        nbt.setInteger("blockData", state.m_storedMetadata.get());
        nbt.setInteger("facing", state.m_facing.get());
        nbt.setFloat("progress", state.m_lastProgress.get());
        nbt.setBoolean("extending", state.m_extending.get());
    },
    ..TILE_ENTITY
};

pub fn createEmpty() -> Rc<TileEntity> { TileEntity::new(&CLASS, Box::new(PistonExt::default())) }

pub fn create(movedBlock: i32, movedMetadata: i32, facing: i32, extending: bool, source: bool) -> Rc<TileEntity>
{
    let ext: PistonExt = PistonExt {
        m_storedBlockID: Cell::new(movedBlock),
        m_storedMetadata: Cell::new(movedMetadata),
        m_facing: Cell::new(facing),
        m_extending: Cell::new(extending),
        m_source: Cell::new(source),
        ..PistonExt::default()
    };
    TileEntity::new(&CLASS, Box::new(ext))
}
