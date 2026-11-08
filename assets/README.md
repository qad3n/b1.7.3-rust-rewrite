# assets

Game assets load from an assets folder at runtime. Nothing in it is committed except this file.

`beta173_launcher` fills its own assets folder in the launcher data folder (see `crates/launcher/README.md`) and points the game at it with `B173_ASSETS`. Run `beta173` directly with `B173_ASSETS` set to that folder, or with this folder filled, for development. Sources, in order:

1. A b1.7.3 client jar: `--client-jar <path>`, the `B173_CLIENT_JAR` env var, or `~/.minecraft/versions/b1.7.3/b1.7.3.jar`. Its non-class files are extracted.
2. The official Mojang download (version manifest, b1.7.3 version json, client jar). Every sha1 is checked.
3. By hand: copy the files of the mcasset.cloud b1.7.3 `assets` folder in here.

Sounds were never inside the b1.7.3 jar. Like Mojang's launcher (`map_to_resources` in the `pre-1.6` asset index), `beta173_launcher` puts them in the game directory's `resources/` folder (`~/.minecraft/resources`, or `B173_MINECRAFT_DIR`), where the vanilla resource thread installs them at startup. They come from the `pre-1.6` asset index, or from `B173_SOUNDS_DIR`, a local folder holding `sound/`, `newsound/`, `music/`, `newmusic/`, `streaming/`, which is copied instead of downloading.
