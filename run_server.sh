#!/usr/bin/env bash
# builds and starts the beta173 server, extra arguments go to MinecraftServer.main
set -euo pipefail

cd "$(dirname "$0")"

profile="${B173_PROFILE:-release}"
cargo build --profile "$profile" --bin beta173_server
exec "target/$profile/beta173_server" "$@"
