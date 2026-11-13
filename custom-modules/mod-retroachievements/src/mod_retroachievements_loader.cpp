#include "Log.h"

// Defined in RetroAchievementsPlayerScript.cpp
void AddRetroAchievementsPlayerScripts();

// The server calls this at startup.
// Name rule: "Add" + folder name with "-" changed to "_" + "Scripts"
void Addmod_retroachievementsScripts()
{
    LOG_INFO("module.retroachievements", "mod-retroachievements: module loaded");
    AddRetroAchievementsPlayerScripts();
}
