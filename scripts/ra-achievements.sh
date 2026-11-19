#!/usr/bin/env bash
# List or reset a character's custom achievements (IDs 60000-60999) for retesting.
#   bash scripts/ra-achievements.sh list  CharacterName
#   bash scripts/ra-achievements.sh reset CharacterName
# Reset only works while the character is LOGGED OUT (the server saves achievements at logout and
# would overwrite the change). If the character is online, nothing is deleted.
# Built-in achievements (for example Level 10) are NOT touched.
# MySQL asks for the root password: type it in the terminal, never in chat.
# Test without touching the database: DRY=1 bash scripts/ra-achievements.sh reset Name
set -euo pipefail
ACTION="${1:-}"
NAME="${2:-}"
if [[ ! "$ACTION" =~ ^(list|reset)$ || ! "$NAME" =~ ^[A-Za-z]{2,12}$ ]]; then
  echo "Usage: bash scripts/ra-achievements.sh list|reset CharacterName"
  exit 1
fi
# Character names are stored as First-letter-capital, rest lowercase (case-sensitive column).
NAME="${NAME,,}"
NAME="${NAME^}"

G="(SELECT guid FROM characters WHERE name='${NAME}')"
GOFF="(SELECT guid FROM characters WHERE name='${NAME}' AND online=0)"

SHOW_STATE="SELECT guid, name, online FROM characters WHERE name='${NAME}';"
LIST="SELECT 'native' AS src, achievement AS id, CAST(FROM_UNIXTIME(date) AS CHAR) AS info FROM character_achievement WHERE guid=${G} AND achievement BETWEEN 60000 AND 60999 UNION ALL SELECT 'local', achievement_id, CAST(unlock_time AS CHAR) FROM mod_ra_character_unlock WHERE guid=${G} AND achievement_id BETWEEN 60000 AND 60999 UNION ALL SELECT 'progress', achievement_id, CAST(counter AS CHAR) FROM mod_ra_character_progress WHERE guid=${G} AND achievement_id BETWEEN 60000 AND 60999 ORDER BY id, src;"

if [[ "$ACTION" == "list" ]]; then
  SQL="${SHOW_STATE} ${LIST}"
else
  SQL="DELETE FROM character_achievement WHERE guid=${GOFF} AND achievement BETWEEN 60000 AND 60999; DELETE FROM character_achievement_progress WHERE guid=${GOFF} AND criteria BETWEEN 60000 AND 60999; DELETE FROM mod_ra_character_unlock WHERE guid=${GOFF} AND achievement_id BETWEEN 60000 AND 60999; DELETE FROM mod_ra_character_progress WHERE guid=${GOFF} AND achievement_id BETWEEN 60000 AND 60999; ${SHOW_STATE} ${LIST}"
  echo "Reset ${NAME}: works only if online = 0 below and the list is empty afterwards."
fi

if [[ "${DRY:-0}" == "1" ]]; then
  echo "$SQL"
  exit 0
fi
docker exec -it ac-database mysql -u root -p acore_characters -e "$SQL"
