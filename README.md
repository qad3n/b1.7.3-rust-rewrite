# b1.7.3-rust-rewrite

The Minecraft Beta 1.7.3 client and server, reconstructed in Rust and then rewritten. It starts from a 1:1 port of the original game and changes it on purpose:

- fixes general bugs from the original game
- improves performance
- adds a built-in mod loader: one mod file works on the client and the server, see `crates/modloader/README.md`
- more improvements over time

Game assets are not included, see `assets/README.md`.

This is a hobby project and a proof of concept. All credit for Minecraft goes to Mojang and its original authors. Not affiliated with Mojang or Microsoft.

```
git clone --recurse-submodules https://github.com/qad3n/b1.7.3-rust-rewrite.git
cargo build --release
```

`run.sh` starts the client through the launcher, and `run_server.sh` starts the server.

## Source layout

```
crates/
  jvm/          Java runtime pieces the game depends on
  common/       NBT, noise, chunk coordinates
  game/         classes shared by the client and server
  client/       client classes, beta173 binary
  server/       server classes, beta173_server binary
  lwjgl/        LWJGL stand-in: display, OpenGL, input
  awt/          AWT stand-in
  launcher/     beta173_launcher binary
  modloader/    built-in mod loader: discovery, registries, wasm host, events, networking
vendor/
  pudscode/             sound engine (submodule)
  b1.7.3-modding-sdk/   mod SDK (submodule): wire protocol, cargo b173, b173_api, example mods
assets/         game assets, loaded at runtime
```
