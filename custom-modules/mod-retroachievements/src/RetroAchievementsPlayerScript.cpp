#include "Log.h"
#include "Player.h"
#include "ScriptMgr.h"

class RetroAchievementsPlayerScript : public PlayerScript
{
public:
    RetroAchievementsPlayerScript() : PlayerScript("RetroAchievementsPlayerScript", { PLAYERHOOK_ON_LOGIN }) { }

    void OnPlayerLogin(Player* player) override
    {
        LOG_INFO("module.retroachievements", "mod-retroachievements: player {} logged in", player->GetName());
    }
};

void AddRetroAchievementsPlayerScripts()
{
    new RetroAchievementsPlayerScript();
}
