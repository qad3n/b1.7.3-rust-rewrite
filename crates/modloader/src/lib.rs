pub mod atlas;
pub mod attributes;
pub mod behavior;
pub mod biomes;
pub mod catalog;
pub mod classes;
pub mod commands;
pub mod config;
pub mod containers;
pub mod content;
pub mod damage;
pub mod devtools;
pub mod dimensions;
pub mod discovery;
pub mod entities;
pub mod events;
pub mod extensions;
pub mod gameevents;
pub mod gamelog;
pub mod hooks;
pub mod host;
pub mod keybinds;
pub mod lifecycle;
pub mod log;
pub mod manifest;
pub mod modify;
pub mod netplay;
pub mod netsync;
pub mod network;
pub mod palette;
pub mod paths;
pub mod persist;
pub mod play;
pub mod profiler;
pub mod registry;
pub mod remap;
pub mod resolver;
pub mod screens;
pub mod services;
pub mod side;
pub mod sounds;
pub mod state;
pub mod states;
pub mod structures;
pub mod textures;
pub mod tiles;
pub mod viewer;
pub mod wasm;
pub mod worlds;

pub use mod_protocol::abi::EventKind;
pub use mod_protocol::{self, Identifier};
use paths::LoaderPaths;
use std::cell::OnceCell;
use std::path::Path;

pub const LOADER_VERSION: &str = env!("CARGO_PKG_VERSION");

thread_local!
{
    static PATHS: OnceCell<LoaderPaths> = const { OnceCell::new() };
}

pub fn paths() -> Option<LoaderPaths> { PATHS.with(|paths| paths.get().cloned()) }

// each binary calls this once at startup with its game directory
pub fn bootstrap(root: &Path)
{
    let paths: LoaderPaths = LoaderPaths::under(root);
    if let Err(error) = paths.create()
    {
        log::error(
            log::LOADER,
            &format!("could not create {} or {}: {error}", paths.m_mods.display(), paths.m_config.display()),
        );
    }

    lifecycle::load(&paths);
    PATHS.with(|slot| slot.set(paths).ok());
}
