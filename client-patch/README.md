# Client patch (not stored in this repo)

The client patch `patch-enUS-4.MPQ` is NOT committed here: it contains Blizzard's achievement data files (DBC).
Keep backups on your own PC / storage. `.gitignore` blocks *.mpq, *.MPQ and *.dbc.

## Install
Put exactly ONE copy in `Data\enUS\patch-enUS-4.MPQ` with the game closed. Never also mount it as another patch name.

## Current build (2026-11-19)
- Size: 325099 bytes
- SHA256: 8cf5339ad572eb0dfff06d8d771fab480efa4e0f2f490985d344b911fad6dafe
- Contains: DBFilesClient\Achievement.dbc, Achievement_Category.dbc (unchanged), Achievement_Criteria.dbc
- Category 90000 "Server Achievements"

## Achievements in the patch
| ID | Name | Client criteria (type / asset / qty) | Server condition |
|----|------|--------------------------------------|------------------|
| 60001 | Rookie | 5 / 0 / 10 | reach level 10 |
| 60002 | Wolf Hunter | 0 / 299 / 3 | kill creature 299 x3 |
| 60003 | Island Reclaimer | 27 / 8325 / 1 | complete quest 8325 |
| 60004 | Welcome to Tranquillien | 43 / 1148 / 1 | enter area 3488 |

Client criteria rows are display data only; the server decides the grant (see the module code).
Type 43 asset 1148 is the built-in "Tranquillien" explore entry, not area 3488.

## Why IDs 60001+
`character_achievement.achievement` is `smallint unsigned` (max 65535), so IDs above that are not saved.
The client data has no built-in achievement or criteria IDs between 20000 and 65535.

## Server side
- World DB: `achievement_dbc` rows and `mod_ra_achievement` rows: `data/sql/db-world/updates/2026_11_19_00.sql`
- Characters DB cleanup: `data/sql/db-characters/updates/2026_11_19_00.sql`
- Retest helper: `bash scripts/ra-achievements.sh list|reset CharacterName` (character must be logged out)
