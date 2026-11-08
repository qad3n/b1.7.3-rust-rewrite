use crate::worldgen::{BIOME_GEN_BASE, BiomeGenBaseClass};

pub const BIOME_GEN_DESERT: BiomeGenBaseClass =
    BiomeGenBaseClass { m_name: "BiomeGenDesert", m_parent: Some(&BIOME_GEN_BASE), ..BIOME_GEN_BASE };
