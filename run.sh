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
exec target/release/beta173_launcher "$@"
