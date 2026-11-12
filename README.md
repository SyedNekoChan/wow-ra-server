# WoW WotLK + RetroAchievements server

AzerothCore (WotLK 3.3.5a) with community modules and our own
RetroAchievements module. Free to play, no donations.

## Folders
- modules.list      community modules to include (one per line)
- custom-modules/   our own modules (needs a src folder to be built)
- config/           Docker settings copied into the build
- scripts/          helper commands
- docs/             project notes

## Commands
    bash scripts/build.sh          download everything, build, start
    bash scripts/dc.sh ps          what is running
    bash scripts/dc.sh logs --tail=50
    bash scripts/console.sh        world server console (exit: Ctrl+X then X)

## Rules
- Never commit passwords, tokens or API keys.
- Never run "docker compose down -v" (deletes the database).
