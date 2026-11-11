# beta173_launcher

Offline command line launcher for the `beta173` client. It installs the game assets from Mojang and starts the game.

```
beta173_launcher [--client-jar <path>] [--game <path>] [--dev]
```

The launcher passes no name. The game asks for one in Multiplayer before it connects, saves it as `lastName` in `options.txt` and plays single player under it on later starts. Running `beta173 <name> <session>` directly still sets the name, like Mojang's b1.7.3 version json (`${auth_player_name} ${auth_session}`).

The game starts from `beta173` next to the launcher, or `--game <path>`.

The game runs detached with no console, and the launcher exits once it has started. Its log still goes to `logs/client.log`. `--dev` keeps the game in the launcher's console and waits for it to exit. `run.sh` and `run.bat` pass it.

## Data folder

The launcher, game and server share one per-user data folder, see the main README. The launcher installs `assets/` (or into `B173_ASSETS`) and puts the sounds in `resources/`, as the vanilla game expects. See `assets/README.md` in the repo root for the asset sources.
