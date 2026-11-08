use crate::block::map_color::{MapColor, MapColors};
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MaterialKind
{
    Material,
    Liquid,
    Logic,
    Transparent,
    Portal,
}

pub struct Material
{
    pub m_kind: MaterialKind,
    m_canBurn: bool,
    m_groundCover: bool,
    m_isOpaque: bool,
    pub m_materialMapColor: Rc<MapColor>,
    m_canHarvest: bool,
    m_mobilityFlag: i32,
}

macro_rules! materials {
    ($($name:ident),* $(,)?) => {
        pub struct Materials
        {
            $(pub $name: Rc<Material>,)*
        }

        impl Material
        {
            $(pub fn $name() -> Rc<Material> { MATERIALS.with(|materials| Rc::clone(&materials.$name)) })*
        }
    };
}

materials!(
    air,
    grassMaterial,
    ground,
    wood,
    rock,
    iron,
    water,
    lava,
    leaves,
    plants,
    sponge,
    cloth,
    fire,
    sand,
    circuits,
    glass,
    tnt,
    CORAL,
    ice,
    snow,
    builtSnow,
    cactus,
    clay,
    pumpkin,
    portal,
    cakeMaterial,
    COBWEB,
    PISTON,
);

thread_local! {
    static MATERIALS: Materials = Materials::create();
}

impl Material
{
    fn new(kind: MaterialKind, color: &Rc<MapColor>) -> Self
    {
        let mut material: Self = Self {
            m_kind: kind,
            m_canBurn: false,
            m_groundCover: false,
            m_isOpaque: false,
            m_materialMapColor: Rc::clone(color),
            m_canHarvest: true,
            m_mobilityFlag: 0,
        };
        match kind
        {
            MaterialKind::Liquid =>
            {
                material = material.setIsGroundCover();
                material = material.setNoPushMobility();
            }
            MaterialKind::Transparent => material = material.setIsGroundCover(),
            _ => (),
        }
        material
    }

    pub fn getIsLiquid(&self) -> bool { self.m_kind == MaterialKind::Liquid }

    pub fn isSolid(&self) -> bool { self.m_kind == MaterialKind::Material }

    pub fn getCanBlockGrass(&self) -> bool { matches!(self.m_kind, MaterialKind::Material | MaterialKind::Liquid) }

    pub fn getIsSolid(&self) -> bool { matches!(self.m_kind, MaterialKind::Material) }

    fn setIsTranslucent(mut self) -> Self
    {
        self.m_isOpaque = true;
        self
    }

    fn setNoHarvest(mut self) -> Self
    {
        self.m_canHarvest = false;
        self
    }

    fn setBurning(mut self) -> Self
    {
        self.m_canBurn = true;
        self
    }

    pub fn getBurning(&self) -> bool { self.m_canBurn }

    fn setIsGroundCover(mut self) -> Self
    {
        self.m_groundCover = true;
        self
    }

    pub fn getIsGroundCover(&self) -> bool { self.m_groundCover }

    pub fn getIsTranslucent(&self) -> bool
    {
        if self.m_isOpaque
        {
            false
        }
        else
        {
            self.getIsSolid()
        }
    }

    pub fn getIsHarvestable(&self) -> bool { self.m_canHarvest }

    pub fn getMaterialMobility(&self) -> i32 { self.m_mobilityFlag }

    fn setNoPushMobility(mut self) -> Self
    {
        self.m_mobilityFlag = 1;
        self
    }

    fn setImmovableMobility(mut self) -> Self
    {
        self.m_mobilityFlag = 2;
        self
    }
}

impl Materials
{
    fn create() -> Self
    {
        use MaterialKind::*;
        let c: Rc<MapColors> = MapColor::colors();
        let m = |kind: MaterialKind, color: &Rc<MapColor>| crate::block::material::Material::new(kind, color);
        let rc = Rc::new;
        Self {
            air: rc(m(Transparent, &c.m_airColor)),
            grassMaterial: rc(m(Material, &c.m_grassColor)),
            ground: rc(m(Material, &c.m_dirtColor)),
            wood: rc(m(Material, &c.m_woodColor).setBurning()),
            rock: rc(m(Material, &c.m_stoneColor).setNoHarvest()),
            iron: rc(m(Material, &c.m_ironColor).setNoHarvest()),
            water: rc(m(Liquid, &c.m_waterColor).setNoPushMobility()),
            lava: rc(m(Liquid, &c.m_tntColor).setNoPushMobility()),
            leaves: rc(m(Material, &c.m_foliageColor).setBurning().setIsTranslucent().setNoPushMobility()),
            plants: rc(m(Logic, &c.m_foliageColor).setNoPushMobility()),
            sponge: rc(m(Material, &c.m_clothColor)),
            cloth: rc(m(Material, &c.m_clothColor).setBurning()),
            fire: rc(m(Transparent, &c.m_airColor).setNoPushMobility()),
            sand: rc(m(Material, &c.m_sandColor)),
            circuits: rc(m(Logic, &c.m_airColor).setNoPushMobility()),
            glass: rc(m(Material, &c.m_airColor).setIsTranslucent()),
            tnt: rc(m(Material, &c.m_tntColor).setBurning().setIsTranslucent()),
            CORAL: rc(m(Material, &c.m_foliageColor).setNoPushMobility()),
            ice: rc(m(Material, &c.m_iceColor).setIsTranslucent()),
            snow: rc(m(Logic, &c.m_snowColor).setIsGroundCover().setIsTranslucent().setNoHarvest().setNoPushMobility()),
            builtSnow: rc(m(Material, &c.m_snowColor).setNoHarvest()),
            cactus: rc(m(Material, &c.m_foliageColor).setIsTranslucent().setNoPushMobility()),
            clay: rc(m(Material, &c.m_clayColor)),
            pumpkin: rc(m(Material, &c.m_foliageColor).setNoPushMobility()),
            portal: rc(m(Portal, &c.m_airColor).setImmovableMobility()),
            cakeMaterial: rc(m(Material, &c.m_airColor).setNoPushMobility()),
            COBWEB: rc(m(Material, &c.m_clothColor).setNoHarvest().setNoPushMobility()),
            PISTON: rc(m(Material, &c.m_stoneColor).setImmovableMobility()),
        }
    }
}
