use std::path::PathBuf;

const MINECRAFT_DIR_ENV: &str = "B173_MINECRAFT_DIR";

pub fn minecraftDirOverride() -> Option<PathBuf> { std::env::var_os(MINECRAFT_DIR_ENV).map(PathBuf::from) }

pub fn osName() -> &'static str { std::env::consts::OS }

pub fn userHome() -> PathBuf { std::env::var_os("HOME").map_or_else(|| PathBuf::from("."), PathBuf::from) }
