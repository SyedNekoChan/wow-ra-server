#!/usr/bin/env bash
# Runs docker compose in the right folder. Examples:
#   bash scripts/dc.sh ps
#   bash scripts/dc.sh logs --tail=50
#   bash scripts/dc.sh restart
#   bash scripts/dc.sh down
# NEVER add -v to "down": that deletes your database.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT/azerothcore-wotlk"
docker compose "$@"
