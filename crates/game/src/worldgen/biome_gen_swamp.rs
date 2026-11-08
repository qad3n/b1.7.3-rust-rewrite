use crate::worldgen::{BIOME_GEN_BASE, BiomeGenBaseClass};

pub const BIOME_GEN_SWAMP: BiomeGenBaseClass =
    BiomeGenBaseClass { m_name: "BiomeGenSwamp", m_parent: Some(&BIOME_GEN_BASE), ..BIOME_GEN_BASE };
