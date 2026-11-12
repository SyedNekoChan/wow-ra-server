#!/usr/bin/env bash
# Builds and starts the server. To rebuild just one part, name it:
#   bash scripts/build.sh ac-worldserver
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
bash "$ROOT/scripts/setup.sh"
cd "$ROOT/azerothcore-wotlk"
docker compose up -d --build "$@"
