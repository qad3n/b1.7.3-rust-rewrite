# b1.7.3-rust-reconstruct

A 1:1 reconstruction of the Minecraft Beta 1.7.3 client and server, written in Rust. Game assets are not included, see `assets/README.md`.

This is a hobby project and a proof of concept. All credit for Minecraft goes to Mojang and its original authors. Not affiliated with Mojang or Microsoft.

```
cargo build --release
```

## Source layout

```
crates/
  jvm/        Java runtime pieces the game depends on
  common/     NBT, noise, chunk coordinates
  game/       classes shared by the client and server
  client/     client classes, beta173 binary
  server/     server classes, beta173_server binary
  lwjgl/      LWJGL stand-in: display, OpenGL, input
  awt/        AWT stand-in
  launcher/   beta173_launcher binary
vendor/
  pudscode/   sound engine (submodule)
assets/       game assets, loaded at runtime
```

`run.sh` starts the client through the launcher, and `run_server.sh` starts the server.
