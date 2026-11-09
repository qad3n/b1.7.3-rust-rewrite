#!/usr/bin/env bash
# builds the game and launcher, extra arguments go to beta173_launcher
set -euo pipefail

cd "$(dirname "$0")"

# dev checkouts reuse the local jar and sounds instead of downloading them
files="$PWD/../b1.7.3_files"
if [[ -z "${B173_CLIENT_JAR:-}" && -f "$files/minecraft-b1.7.3-client.jar" ]]; then
    export B173_CLIENT_JAR="$files/minecraft-b1.7.3-client.jar"
fi
if [[ -z "${B173_SOUNDS_DIR:-}" && -d "$files/InventivetalentDev-minecraft-assets-737829f/assets" ]]; then
    export B173_SOUNDS_DIR="$files/InventivetalentDev-minecraft-assets-737829f/assets"
fi

cargo build --release --bin beta173 --bin beta173_launcher
# swallowing compositors hide the terminal that parents a game window so the launcher is orphaned but keeps this console
exec 3>&1 4>&2
pid="$( (target/release/beta173_launcher --dev "$@" >&3 2>&4 & echo $!) )"
trap 'kill "$pid" 2>/dev/null' INT TERM
while kill -0 "$pid" 2>/dev/null; do
    sleep 0.5
done
