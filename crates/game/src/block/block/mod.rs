mod base;
mod class;
mod registry;
mod step_sound;

use crate::block::Material;
pub use class::{BLOCK, BlockClass};
use jvm::field::Field;
use jvm::object::JavaObject;
use std::any::Any;
use std::cell::Cell;
use std::rc::Rc;
pub use step_sound::StepSound;

pub struct Block
{
    pub m_class: &'static BlockClass,
    pub m_ext: Box<dyn Any>,
    pub m_blockIndexInTexture: Cell<i32>,
    pub m_blockID: i32,
    pub m_blockHardness: Cell<f32>,
    pub m_blockResistance: Cell<f32>,
    pub m_blockConstructorCalled: Cell<bool>,
    pub m_enableStats: Cell<bool>,
    pub m_minX: Cell<f64>,
    pub m_minY: Cell<f64>,
    pub m_minZ: Cell<f64>,
    pub m_maxX: Cell<f64>,
    pub m_maxY: Cell<f64>,
    pub m_maxZ: Cell<f64>,
    pub m_stepSound: Field<Rc<StepSound>>,
    pub m_blockParticleGravity: Cell<f32>,
    pub m_blockMaterial: Rc<Material>,
    pub m_slipperiness: Cell<f32>,
    m_blockName: Field<Option<String>>,
}

impl Block
{
    pub fn construct(class: &'static BlockClass, ext: Box<dyn Any>, id: i32, material: Rc<Material>) -> Rc<Self>
    {
        if Block::exists(id)
        {
            jvm::throwable::throw(
                &jvm::throwable::ILLEGAL_ARGUMENT,
                &format!("Slot {id} is already occupied when adding {id}"),
            );
        }

        let block: Rc<Self> = Rc::new(Self {
            m_class: class,
            m_ext: ext,
            m_blockIndexInTexture: Cell::new(0),
            m_blockID: id,
            m_blockHardness: Cell::new(0.0),
            m_blockResistance: Cell::new(0.0),
            m_blockConstructorCalled: Cell::new(true),
            m_enableStats: Cell::new(true),
            m_minX: Cell::new(0.0),
            m_minY: Cell::new(0.0),
            m_minZ: Cell::new(0.0),
            m_maxX: Cell::new(0.0),
            m_maxY: Cell::new(0.0),
            m_maxZ: Cell::new(0.0),
            m_stepSound: Field::new(StepSound::powder()),
            m_blockParticleGravity: Cell::new(1.0),
            m_blockMaterial: Rc::clone(&material),
            m_slipperiness: Cell::new(0.6),
            m_blockName: Field::new(None),
        });
        registry::register(id, &block);
        block.setBlockBounds(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        let opaque: bool = block.isOpaqueCube();
        registry::setOpaqueCubeLookup(id, opaque);
        registry::setLightOpacityValue(id, if block.isOpaqueCube() { 255 } else { 0 });
        registry::setCanBlockGrass(id, !material.getCanBlockGrass());
        registry::setIsBlockContainer(id, false);
        block
    }

    pub fn constructWithSprite(
        class: &'static BlockClass,
        ext: Box<dyn Any>,
        id: i32,
        sprite: i32,
        material: Rc<Material>,
    ) -> Rc<Self>
    {
        let block: Rc<Self> = Self::construct(class, ext, id, material);
        block.m_blockIndexInTexture.set(sprite);
        block
    }

    pub fn ext<T: 'static>(&self) -> &T { self.m_ext.downcast_ref::<T>().expect("block ext matches its class") }

    pub fn disableNeighborNotifyOnMetadataChange(self: Rc<Self>) -> Rc<Self>
    {
        registry::setUpdateClients(self.m_blockID, true);
        self
    }

    pub fn setStepSound(self: Rc<Self>, sounds: Rc<StepSound>) -> Rc<Self>
    {
        self.m_stepSound.set(sounds);
        self
    }

    pub fn setLightOpacity(self: Rc<Self>, opacity: i32) -> Rc<Self>
    {
        registry::setLightOpacityValue(self.m_blockID, opacity);
        self
    }

    pub fn setLightValue(self: Rc<Self>, light: f32) -> Rc<Self>
    {
        registry::setLightValueEntry(self.m_blockID, (15.0 * light) as i32);
        self
    }

    pub fn setResistance(self: Rc<Self>, blastResistance: f32) -> Rc<Self>
    {
        self.m_blockResistance.set(blastResistance * 3.0);
        self
    }

    pub fn setHardness(self: Rc<Self>, strength: f32) -> Rc<Self>
    {
        self.m_blockHardness.set(strength);
        if self.m_blockResistance.get() < strength * 5.0
        {
            self.m_blockResistance.set(strength * 5.0);
        }
        self
    }

    pub fn setBlockUnbreakable(self: Rc<Self>) -> Rc<Self> { self.setHardness(-1.0) }

    pub fn getHardness(&self) -> f32 { self.m_blockHardness.get() }

    pub fn setTickOnLoad(self: Rc<Self>, ticksRandomly: bool) -> Rc<Self>
    {
        registry::setTickOnLoadEntry(self.m_blockID, ticksRandomly);
        self
    }

    pub fn setBlockBounds(&self, minX: f32, minY: f32, minZ: f32, maxX: f32, maxY: f32, maxZ: f32)
    {
        self.m_minX.set(minX as f64);
        self.m_minY.set(minY as f64);
        self.m_minZ.set(minZ as f64);
        self.m_maxX.set(maxX as f64);
        self.m_maxY.set(maxY as f64);
        self.m_maxZ.set(maxZ as f64);
    }

    pub fn setBlockName(self: Rc<Self>, key: &str) -> Rc<Self>
    {
        self.m_blockName.set(Some(format!("tile.{key}")));
        self
    }

    pub fn translateBlockName(&self) -> String
    {
        let name: String = self.getBlockName().unwrap_or_else(|| "null".to_owned());
        crate::locale::StatCollector::translateToLocal(&format!("{name}.name"))
    }

    pub fn getBlockName(&self) -> Option<String> { self.m_blockName.get() }

    pub fn getEnableStats(&self) -> bool { self.m_enableStats.get() }

    pub fn disableStats(self: Rc<Self>) -> Rc<Self>
    {
        self.m_enableStats.set(false);
        self
    }

    pub fn dropBlockAsItem(self: &Rc<Self>, world: &Rc<crate::world::World>, x: i32, y: i32, z: i32, metadata: i32)
    {
        self.dropBlockAsItemWithChance(world, x, y, z, metadata, 1.0);
    }
}

impl JavaObject for Block
{
    fn javaEquals(&self, other: &Self) -> bool { std::ptr::eq(self, other) }

    fn javaHash(&self) -> i32 { self.m_blockID }
}
