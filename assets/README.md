# assets

Game assets load at runtime from `assets/` in the game data folder (see the main README), or from `B173_ASSETS`. Nothing here is committed except this file.

`beta173_launcher` fills that folder. Sources, in order:

1. A b1.7.3 client jar: `--client-jar <path>`, the `B173_CLIENT_JAR` env var, or the official launcher's `.minecraft/versions/b1.7.3/b1.7.3.jar`. Its non-class files are extracted.
2. The official Mojang download (version manifest, b1.7.3 version json, client jar). Every sha1 is checked.
3. By hand: copy the files of the mcasset.cloud b1.7.3 `assets` folder into the data folder's `assets/`.

Sounds were never inside the b1.7.3 jar. Like Mojang's launcher (`map_to_resources` in the `pre-1.6` asset index), `beta173_launcher` puts them in the data folder's `resources/`, where the vanilla resource thread installs them at startup. They come from the `pre-1.6` asset index, or from `B173_SOUNDS_DIR`, a local folder holding `sound/`, `newsound/`, `music/`, `newmusic/`, `streaming/`, which is copied instead of downloading.
