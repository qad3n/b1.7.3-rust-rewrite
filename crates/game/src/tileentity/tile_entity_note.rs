use crate::block::Material;
use crate::tileentity::{TILE_ENTITY, TileEntity, TileEntityClass};
use crate::world::World;
use std::cell::Cell;
use std::rc::Rc;

#[derive(Default)]
pub struct NoteExt
{
    pub m_note: Cell<i8>,
    pub m_previousRedstoneState: Cell<bool>,
}

fn note(this: &TileEntity) -> &NoteExt { this.ext::<NoteExt>() }

pub fn changePitch(this: &Rc<TileEntity>)
{
    note(this).m_note.set(((note(this).m_note.get() as i32 + 1) % 25) as i8);
    this.onInventoryChanged();
}

pub fn triggerNote(this: &Rc<TileEntity>, world: &Rc<World>, x: i32, y: i32, z: i32)
{
    if !Rc::ptr_eq(&world.getBlockMaterial(x, y + 1, z), &Material::air())
    {
        return;
    }
    let material: Rc<Material> = world.getBlockMaterial(x, y - 1, z);
    let mut n: i32 = 0;
    if Rc::ptr_eq(&material, &Material::rock())
    {
        n = 1;
    }
    if Rc::ptr_eq(&material, &Material::sand())
    {
        n = 2;
    }
    if Rc::ptr_eq(&material, &Material::glass())
    {
        n = 3;
    }
    if Rc::ptr_eq(&material, &Material::wood())
    {
        n = 4;
    }
    world.playNoteAt(x, y, z, n, note(this).m_note.get() as i32);
}

pub const CLASS: TileEntityClass = TileEntityClass {
    m_name: "TileEntityNote",
    m_parent: Some(&TILE_ENTITY),
    writeToNBT: |this, nbt| {
        (TILE_ENTITY.writeToNBT)(this, nbt);
        nbt.setByte("note", note(this).m_note.get());
    },
    readFromNBT: |this, nbt| {
        (TILE_ENTITY.readFromNBT)(this, nbt);
        let state: &NoteExt = note(this);
        state.m_note.set(nbt.getByte("note"));
        if state.m_note.get() < 0
        {
            state.m_note.set(0);
        }
        if state.m_note.get() > 24
        {
            state.m_note.set(24);
        }
    },
    ..TILE_ENTITY
};

pub fn create() -> Rc<TileEntity> { TileEntity::new(&CLASS, Box::new(NoteExt::default())) }
