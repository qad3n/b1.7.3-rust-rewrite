#!/usr/bin/env bash
# builds and starts the beta173 server, extra arguments go to MinecraftServer.main
set -euo pipefail

cd "$(dirname "$0")"
root="$PWD"

profile="${B173_PROFILE:-release}"
cargo build --profile "$profile" --bin beta173_server

# font.txt and the lang files come from the shared assets folder
export B173_ASSETS="${B173_ASSETS:-$root/assets}"

# the server keeps server.properties, worlds and logs in its working folder like the original jar
serverDir="${B173_SERVER_DIR:-$root/run/server}"
mkdir -p "$serverDir"
cd "$serverDir"
exec "$root/target/$profile/beta173_server" "$@"
