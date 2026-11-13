#include "Log.h"
#include "ScriptMgr.h"

void AddRetroAchievementsPlayerScripts();

void Addmod_retroachievementsScripts()
{
    LOG_INFO("module.retroachievements", "mod-retroachievements: module loaded");
    AddRetroAchievementsPlayerScripts();
}
