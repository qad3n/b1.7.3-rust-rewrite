# beta173_launcher

Offline command line launcher for the `beta173` client. It asks for a player name, installs the game assets from Mojang, and starts the game.

```
beta173_launcher [--client-jar <path>] [--game <path>]
```

On first launch it asks for an offline name (3 to 16 letters, digits or underscores) and saves it. Later launches offer to play as the saved name or change it.

The game gets the name and `-` as its session, the same two positional arguments Mojang's b1.7.3 version json passes (`${auth_player_name} ${auth_session}`). `-` means no session, so the client joins servers in offline mode only.

The game starts from `beta173` next to the launcher, or `--game <path>`.

## Data folder

`$XDG_DATA_HOME/beta173` (default `~/.local/share/beta173`) on Linux, `~/Library/Application Support/beta173` on macOS, `%APPDATA%\beta173` on Windows. It holds `name.txt` and `assets/`, unless `B173_ASSETS` names another assets folder. Sounds go to the game directory's `resources/` folder instead, as the vanilla game expects. See `assets/README.md` at the workspace root for the asset sources.
