use super::{WORLD_PROVIDER, WorldProviderClass};

pub const WORLD_PROVIDER_SURFACE: WorldProviderClass =
    WorldProviderClass { m_name: "WorldProviderSurface", m_parent: Some(&WORLD_PROVIDER), ..WORLD_PROVIDER };
