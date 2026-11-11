use crate::block::{Block, Material};
use std::cell::{Cell, RefCell, RefMut};
use std::rc::Rc;

// which beta liquid a fluid behaves like for sounds, particles, render pass, hardening, fire and slow spreading
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FluidStyle
{
    Water,
    Lava,
}

pub struct Fluid
{
    pub m_name: String,
    pub m_material: Rc<Material>,
    pub m_style: FluidStyle,
    // ids change when a world remaps them, so the loader sets them again
    pub m_flowing: Cell<i32>,
    pub m_still: Cell<i32>,
    pub m_bucket: Cell<i32>,
    pub m_tickRate: i32,
    // levels lost per block of flow in the overworld and in the nether
    pub m_decay: i32,
    pub m_netherDecay: i32,
    // two sources beside a flowing block make a new source
    pub m_infinite: bool,
    pub m_sprite: i32,
}

mod beta
{
    pub const WATER_TICKS: i32 = 5;
    pub const LAVA_TICKS: i32 = 30;
    pub const LAVA_DECAY: i32 = 2;
    pub const DECAY: i32 = 1;
    pub const WATER_SPRITE: i32 = 12 * 16 + 13;
    pub const LAVA_SPRITE: i32 = 14 * 16 + 13;
    pub const BUCKET_WATER: i32 = 256 + 70;
    pub const BUCKET_LAVA: i32 = 256 + 71;
}

thread_local!
{
    static FLUIDS: RefCell<Vec<Rc<Fluid>>> = RefCell::new(vanilla());
}

fn vanilla() -> Vec<Rc<Fluid>>
{
    vec![
        Rc::new(Fluid
        {
            m_name: "minecraft:water".to_owned(),
            m_material: Material::water(),
            m_style: FluidStyle::Water,
            m_flowing: Cell::new(Block::WATER_MOVING),
            m_still: Cell::new(Block::WATER_STILL),
            m_bucket: Cell::new(beta::BUCKET_WATER),
            m_tickRate: beta::WATER_TICKS,
            m_decay: beta::DECAY,
            m_netherDecay: beta::DECAY,
            m_infinite: true,
            m_sprite: beta::WATER_SPRITE,
        }),
        Rc::new(Fluid
        {
            m_name: "minecraft:lava".to_owned(),
            m_material: Material::lava(),
            m_style: FluidStyle::Lava,
            m_flowing: Cell::new(Block::LAVA_MOVING),
            m_still: Cell::new(Block::LAVA_STILL),
            m_bucket: Cell::new(beta::BUCKET_LAVA),
            m_tickRate: beta::LAVA_TICKS,
            m_decay: beta::LAVA_DECAY,
            m_netherDecay: beta::DECAY,
            m_infinite: false,
            m_sprite: beta::LAVA_SPRITE,
        }),
    ]
}

impl Fluid
{
    pub fn vanilla(&self) -> bool { self.m_name.starts_with("minecraft:") }
}

pub fn of(material: &Rc<Material>) -> Option<Rc<Fluid>>
{
    FLUIDS.with(|fluids| fluids.borrow().iter().find(|fluid| Rc::ptr_eq(&fluid.m_material, material)).cloned())
}

pub fn byName(name: &str) -> Option<Rc<Fluid>>
{
    FLUIDS.with(|fluids| fluids.borrow().iter().find(|fluid| fluid.m_name == name).cloned())
}

// a mod fluid of the same style as a vanilla liquid counts as that liquid for entities
pub fn alike(material: &Rc<Material>, liquid: &Rc<Material>) -> bool
{
    let (Some(fluid), Some(wanted)) = (of(material), of(liquid))
    else
    {
        return false;
    };

    !fluid.vanilla() && fluid.m_style == wanted.m_style
}

pub fn add(fluid: Fluid)
{
    FLUIDS.with(|fluids|
    {
        let mut fluids: RefMut<Vec<Rc<Fluid>>> = fluids.borrow_mut();
        fluids.retain(|old| old.m_name != fluid.m_name);
        fluids.push(Rc::new(fluid));
    });
}

// the terrain sprite of a fluid block, any unknown liquid draws as water as beta did
pub fn sprite(material: &Rc<Material>) -> i32 { of(material).map_or(beta::WATER_SPRITE, |fluid| fluid.m_sprite) }

// runs for each mod fluid of a style without collecting them, entity movement asks this every tick
pub fn eachModded(style: FluidStyle, mut body: impl FnMut(&Fluid))
{
    FLUIDS.with(|fluids|
    {
        for fluid in fluids.borrow().iter().filter(|fluid| fluid.m_style == style && !fluid.vanilla())
        {
            body(fluid);
        }
    });
}

pub fn removeModded() { FLUIDS.with(|fluids| fluids.borrow_mut().retain(|fluid| fluid.vanilla())); }
