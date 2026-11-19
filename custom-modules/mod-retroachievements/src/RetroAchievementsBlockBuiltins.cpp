#include "AchievementMgr.h"
#include "DBCStructure.h"
#include "Player.h"
#include "ScriptMgr.h"

// Refuses every achievement that is not one of ours.
// Our ids are 20000..65535 (the database columns are smallint unsigned, max 65535).
// The built-in WotLK achievements all have lower ids, so they are never granted.
class RetroAchievementsBlockBuiltins : public PlayerScript
{
public:
    RetroAchievementsBlockBuiltins() : PlayerScript("RetroAchievementsBlockBuiltins", { PLAYERHOOK_ON_BEFORE_ACHI_COMPLETE }) { }

    bool OnPlayerBeforeAchievementComplete(Player* /*player*/, AchievementEntry const* achievement) override
    {
        if (!achievement)
            return true;

        return achievement->ID >= 20000 && achievement->ID <= 65535;
    }
};

void AddRetroAchievementsBlockBuiltins()
{
    new RetroAchievementsBlockBuiltins();
}
