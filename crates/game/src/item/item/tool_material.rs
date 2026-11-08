#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnumToolMaterial
{
    Wood,
    Stone,
    Iron,
    Emerald,
    Gold,
}

struct ToolMaterialRow
{
    m_material: EnumToolMaterial,
    m_harvestLevel: i32,
    m_maxUses: i32,
    m_efficiencyOnProperMaterial: f32,
    m_damageVsEntity: i32,
}

const MATERIALS: [ToolMaterialRow; 5] = [
    ToolMaterialRow {
        m_material: EnumToolMaterial::Wood,
        m_harvestLevel: 0,
        m_maxUses: 59,
        m_efficiencyOnProperMaterial: 2.0,
        m_damageVsEntity: 0,
    },
    ToolMaterialRow {
        m_material: EnumToolMaterial::Stone,
        m_harvestLevel: 1,
        m_maxUses: 131,
        m_efficiencyOnProperMaterial: 4.0,
        m_damageVsEntity: 1,
    },
    ToolMaterialRow {
        m_material: EnumToolMaterial::Iron,
        m_harvestLevel: 2,
        m_maxUses: 250,
        m_efficiencyOnProperMaterial: 6.0,
        m_damageVsEntity: 2,
    },
    ToolMaterialRow {
        m_material: EnumToolMaterial::Emerald,
        m_harvestLevel: 3,
        m_maxUses: 1561,
        m_efficiencyOnProperMaterial: 8.0,
        m_damageVsEntity: 3,
    },
    ToolMaterialRow {
        m_material: EnumToolMaterial::Gold,
        m_harvestLevel: 0,
        m_maxUses: 32,
        m_efficiencyOnProperMaterial: 12.0,
        m_damageVsEntity: 0,
    },
];

impl EnumToolMaterial
{
    fn row(self) -> &'static ToolMaterialRow
    {
        match MATERIALS.iter().find(|row| row.m_material == self)
        {
            Some(row) => row,
            None => unreachable!("every tool material has a row"),
        }
    }

    pub fn getMaxUses(self) -> i32 { self.row().m_maxUses }

    pub fn getEfficiencyOnProperMaterial(self) -> f32 { self.row().m_efficiencyOnProperMaterial }

    pub fn getDamageVsEntity(self) -> i32 { self.row().m_damageVsEntity }

    pub fn getHarvestLevel(self) -> i32 { self.row().m_harvestLevel }
}
