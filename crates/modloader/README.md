# Mod loader

The game has a mod loader built in. You don't need a separate installer.

## Playing with mods

1. Put a `.bmod` file in the `mods/` folder.
   - Client: `mods/` next to `saves/` in the game data folder (see the main README).
   - Server: `server/mods/` in the same data folder.
2. Start the game or the server.

The same `.bmod` file works on both sides. When a mod adds content, the server and the client need the same mods installed. If they don't match, the join screen tells you which mods are missing or have the wrong version.

With no mod files the game runs as vanilla. Nothing extra loads.

In game:

- The Mods and Texture Packs screen gets a "Mods" button. It lists your mods, any load problems, keybind conflicts and each mod's settings.
- If a mod fails to load, a screen before the title screen says why.
- F3 shows how much tick time each mod uses.

Settings files:

- `config/<modid>.toml` holds one mod's settings.
- `config/loader.toml` sets what happens when a mod is too slow:

```toml
[watchdog]
action = "warn"        # warn | disable | stop
tick_budget_ms = 20.0  # time a mod may use per tick
strikes = 40           # slow ticks in a row before the action
```

Server settings in `server.properties`:

- `modsloaded=false` keeps the server vanilla and ignores `mods/`.
- `strictmods=true` refuses any client whose mod list differs from the server's. By default a client is refused only when it lacks a shared mod the server needs, or has a different network version of it.
- A mod is needed on the client when it is `side = "both"` and adds blocks, items, entities, tile entities, dimensions or biomes, which nothing can override. A mod with network channels is also needed unless `[network] required = false`, and `required = true` forces it. Client and server only mods are never needed, and adding synced content to one is a load error.

Logs go to `logs/client.log` in the game folder and `logs/server.log` next to `server.properties`. Mod lines start with the mod id, such as `[ruby]`. Loader lines start with `[loader]`.

## How it works

- A `.bmod` file is a zip. It holds a `mod.toml` manifest, optional `data/` and `assets/` folders, and an optional `mod.wasm` with code.
- On start the loader reads every mod, checks versions and dependencies, and sorts them into load order. Then it adds their blocks, items, recipes and other content to the game's registries.
- Blocks and items have names like `ruby:ruby_ore`, not fixed numbers. The loader picks the numbers and saves them in each world, so adding or removing mods doesn't scramble a save. Content from a removed mod becomes a placeholder, and the loader makes one backup of the world first.
- With mods installed, worlds save blocks and items by name. Chunks store a palette of block names per 16-high section, and item stacks store their item's name. If a world is opened without mods, it still loads from the saved numbers.
- Blocks can have named states such as `facing=north,lit=true`. A block can have up to 256 states. Past 16, the loader uses extra hidden ids.
- Mods never overwrite each other's data:
  - Data a mod keeps on entities, players, tile entities, chunks, the world and item stacks has one slot per mod.
  - Item stacks with different mod data don't stack together.
- Changes to blocks a mod doesn't own (hardness, light, flammability, tool) and to spawn lists run in a separate step, in load order. When two mods change the same thing, the Mod Loader screen shows it under "Block changes".
- A mod's ores and plants are seeded from their names. Adding or removing another mod doesn't move them in new chunks.
- Mod commands also answer to `/<modid>:<name>`. If two mods want the same short name, the first one in load order gets it.
- Mod code runs as WebAssembly in a sandbox. If a mod crashes or hangs, the loader turns that mod off and the game keeps running.
- Mods hook the game through events and behaviors. Many mods can listen to the same event, so they don't overwrite each other.
- On join, the server sends its mod list and registry ids to the client before play starts.
- Single player uses the same code paths as multiplayer.

## Making a mod

A mod can be data only (no code), code, or both.

The SDK lives in its own repo, [b1.7.3-modding-sdk](https://github.com/qad3n/b1.7.3-modding-sdk), vendored here at `vendor/b1.7.3-modding-sdk`. Mod authors can clone just that repo. The paths below are from the game repo root.

Setup, once:

```bash
rustup target add wasm32-unknown-unknown
```

```bash
cargo install --path vendor/b1.7.3-modding-sdk/mod_tool
```

New mod:

```bash
B173_SDK=/path/to/b1.7.3-rust-rewrite/vendor/b1.7.3-modding-sdk/b173_api cargo b173 new mymod
```

Commands, run in the mod folder or with `--path <mod dir>`:

- `cargo b173 check` checks `mod.toml` and every data file.
- `cargo b173 build` makes `target/<id>-<version>.bmod`.
- `cargo b173 run --client` or `--server` builds the mod, copies it into the game data folder's `mods/` or `server/mods/`, and starts the game. The game binaries come from `PATH` or `B173_BIN_DIR`.

### mod.toml

```toml
id = "ruby"
version = "1.2.0"
name = "Ruby"
api = "1.0"
side = "both"

[depends]
loader = ">=0.1"
```

`side` is `both`, `client` or `server`. Other mods go under `[depends]` (required) or `[optional]`.

### Content without code

Put TOML files under `data/<modid>/`:

- `blocks/`, `items/`: copy a vanilla template and change what you need.
- `recipes/`: shaped, shapeless, smelting and fuel.
- `worldgen/`: ore and plant generation.
- `tags/`: groups such as `c:ores/ruby`.
- `states/`: named states for one of your blocks. See below.
- `modify/blocks/`, `modify/spawns/`: changes to blocks and spawn lists you don't own.

Each file under `modify/blocks/` changes one block, vanilla or another mod's:

```toml
target = "minecraft:bookshelf"
flammability = { encouragement = 30, flammability = 20 }
```

`data/<modid>/states/<block>.toml` gives your block named states:

```toml
properties = [{ name = "lit", values = ["false", "true"] }]
default = { lit = "false" }
variants = [{ when = { lit = "true" }, texture = "ruby:block/ruby_lamp_on" }]
```

To make pickaxes, axes or shovels mine your block at full speed, add it to the tag `minecraft:mineable/pickaxe`, `minecraft:mineable/axe` or `minecraft:mineable/shovel`.

A block:

```toml
template = "minecraft:stone"
hardness = 3.0
texture = "ruby:block/ruby_ore"
name = "Ruby Ore"
tool = { class = "pickaxe", level = 2 }
drops = [{ item = "ruby:ruby", count = [1, 2] }]
```

Textures go in `assets/<modid>/textures/<path>.png` and are named `<modid>:<path>`. Names for the game go in `assets/<modid>/lang/en_US.lang`.

### Code

Code mods are Rust crates built for wasm with the `b173_api` SDK in `vendor/b1.7.3-modding-sdk/`. The entry point runs once at load:

```rust
use b173_api::prelude::*;

#[mod_main]
fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    // register blocks, items, events, commands, keybinds, screens here
    Ok(())
}
```

Block, item, entity and tile entity logic are trait impls marked `#[block_behavior]`, `#[item_behavior]`, `#[entity_behavior]` or `#[tile_behavior]`. Only the methods you write are called.

Rules:

- Check `ctx.isAuthority()` before you change the world.
- Check everything a client sends to the server.

### Mods that use other mods

A mod can offer a service that other mods call, such as an energy API. Put the shared names and types in a `<modid>_api` crate. Other mods depend on that crate and list the mod under `[depends]` or `[optional]`.

## Examples

All in `vendor/b1.7.3-modding-sdk/examples/`:

- `ruby`: data only, with blocks, items, recipes, ore generation and a tag.
- `testmod`: uses every part of the API.
- `energy`, `energy_api`, `machines`: two mods that work together through a service.

## For game developers

The game calls the loader through hook sites listed in `hooks.toml`. When the game or server changes, update the hooks, the ABI in `mod_protocol`, the SDK and the examples to match. A released hook or API contract never changes. New behavior gets a new version.

`mod_protocol`, the SDK and the examples are in the `vendor/b1.7.3-modding-sdk` submodule. Commit and push a change there first, then commit the submodule bump in this repo.
