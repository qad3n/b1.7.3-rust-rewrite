# beta173_launcher

Offline command line launcher for the `beta173` client. It asks for a player name, installs the game assets from Mojang, and starts the game.

```
beta173_launcher [--client-jar <path>] [--game <path>] [--dev]
```

On first launch it asks for an offline name (3 to 16 letters, digits or underscores) and saves it. Later launches offer to play as the saved name or change it.

The game gets the name and `-` as its session, the same two positional arguments Mojang's b1.7.3 version json passes (`${auth_player_name} ${auth_session}`). `-` means no session, so the client joins servers in offline mode only.

The game starts from `beta173` next to the launcher, or `--game <path>`.

The game runs detached with no console, and the launcher exits once it has started. Its log still goes to `logs/client.log`. `--dev` keeps the game in the launcher's console and waits for it to exit. `run.sh` and `run.bat` pass it.

## Data folder

The launcher, game and server share one per-user data folder, see the main README. The launcher keeps `name.txt` there, installs `assets/` (or into `B173_ASSETS`) and puts the sounds in `resources/`, as the vanilla game expects. See `assets/README.md` at the workspace root for the asset sources.
