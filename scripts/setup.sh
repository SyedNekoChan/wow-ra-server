#!/usr/bin/env bash
# Downloads AzerothCore and the modules in modules.list, then copies our own
# modules in. Safe to run again at any time.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
AC_DIR="$ROOT/azerothcore-wotlk"

echo "==> 1/5  AzerothCore source"
if [ ! -d "$AC_DIR/.git" ]; then
  git clone --depth 1 https://github.com/azerothcore/azerothcore-wotlk.git "$AC_DIR"
else
  echo "    already downloaded"
fi

echo "==> 2/5  Community modules (from modules.list)"
while read -r name url _; do
  if [ -z "${name:-}" ] || [ "${name:0:1}" = "#" ]; then continue; fi
  if [ -d "$AC_DIR/modules/$name" ]; then
    echo "    $name already present"
  else
    git clone --depth 1 "$url" "$AC_DIR/modules/$name"
  fi
done < "$ROOT/modules.list"

echo "==> 3/5  Our own modules (from custom-modules/)"
for dir in "$ROOT"/custom-modules/*/; do
  [ -d "$dir" ] || continue
  name="$(basename "$dir")"
  if [ ! -d "$dir/src" ]; then
    echo "    $name has no src folder yet - skipped"
    continue
  fi
  rm -rf "$AC_DIR/modules/$name"
  cp -r "$dir" "$AC_DIR/modules/$name"
  echo "    copied $name"
done

echo "==> 4/5  Docker settings"
cp "$ROOT/config/docker-compose.override.yml" "$AC_DIR/docker-compose.override.yml"

echo "==> 5/5  SQL files that come with modules"
echo "    (Docker does NOT import these by itself. Claude will help place them.)"
find "$AC_DIR/modules" -name '*.sql' -not -path '*/.git/*' | sort || true
echo "Done."
