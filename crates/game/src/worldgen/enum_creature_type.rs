use crate::block::Material;
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnumCreatureType
{
    Monster,
    Creature,
    WaterCreature,
}

struct CreatureTypeRow
{
    m_creatureClass: &'static str,
    m_maxNumberOfCreature: i32,
    m_creatureMaterial: fn() -> Rc<Material>,
    m_isPeacefulCreature: bool,
}

static ROWS: [CreatureTypeRow; 3] = [
    CreatureTypeRow {
        m_creatureClass: "IMob",
        m_maxNumberOfCreature: 70,
        m_creatureMaterial: Material::air,
        m_isPeacefulCreature: false,
    },
    CreatureTypeRow {
        m_creatureClass: "EntityAnimal",
        m_maxNumberOfCreature: 15,
        m_creatureMaterial: Material::air,
        m_isPeacefulCreature: true,
    },
    CreatureTypeRow {
        m_creatureClass: "EntityWaterMob",
        m_maxNumberOfCreature: 5,
        m_creatureMaterial: Material::water,
        m_isPeacefulCreature: true,
    },
];

impl EnumCreatureType
{
    pub const VALUES: [EnumCreatureType; 3] = [Self::Monster, Self::Creature, Self::WaterCreature];

    fn row(self) -> &'static CreatureTypeRow { &ROWS[self as usize] }

    pub fn getCreatureClass(self) -> &'static str { self.row().m_creatureClass }

    pub fn getMaxNumberOfCreature(self) -> i32 { self.row().m_maxNumberOfCreature }

    pub fn getCreatureMaterial(self) -> Rc<Material> { (self.row().m_creatureMaterial)() }

    pub fn getPeacefulCreature(self) -> bool { self.row().m_isPeacefulCreature }
}
