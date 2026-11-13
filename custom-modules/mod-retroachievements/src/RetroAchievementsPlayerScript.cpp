#include "Chat.h"
#include "Creature.h"
#include "DatabaseEnv.h"
#include "Item.h"
#include "Log.h"
#include "Player.h"
#include "QuestDef.h"
#include "ScriptMgr.h"
#include <algorithm>
#include <string>
#include <unordered_map>
#include <unordered_set>
#include <vector>

namespace
{
    enum RaCondition : uint8
    {
        RA_COND_REACH_LEVEL    = 1,
        RA_COND_KILL_CREATURE  = 2,
        RA_COND_COMPLETE_QUEST = 3,
        RA_COND_ENTER_AREA     = 4,
        RA_COND_LOOT_ITEM      = 5
    };

    struct RaAchievement
    {
        uint32 id;
        std::string name;
        std::string description;
        uint8 type;
        uint32 value1;
        uint32 value2;
        uint32 points;
    };

    struct RaPlayerData
    {
        std::unordered_set<uint32> unlocked;
        std::unordered_map<uint32, uint32> progress;
    };

    std::vector<RaAchievement> sAchievements;
    std::unordered_map<uint32, RaPlayerData> sPlayers; // key: character guid

    void LoadAchievements()
    {
        sAchievements.clear();
        QueryResult result = WorldDatabase.Query("SELECT id, name, description, condition_type, value1, value2, points FROM mod_ra_achievement ORDER BY id");
        if (!result)
        {
            LOG_ERROR("module.retroachievements", "mod-retroachievements: no achievements loaded (is table mod_ra_achievement imported and filled?)");
            return;
        }

        do
        {
            Field* f = result->Fetch();
            RaAchievement a;
            a.id          = f[0].Get<uint32>();
            a.name        = f[1].Get<std::string>();
            a.description = f[2].Get<std::string>();
            a.type        = f[3].Get<uint8>();
            a.value1      = f[4].Get<uint32>();
            a.value2      = f[5].Get<uint32>();
            a.points      = f[6].Get<uint32>();
            sAchievements.push_back(a);
        } while (result->NextRow());

        LOG_INFO("module.retroachievements", "mod-retroachievements: loaded {} achievements", sAchievements.size());
    }

    void LoadPlayer(uint32 guid)
    {
        RaPlayerData& data = sPlayers[guid];
        data.unlocked.clear();
        data.progress.clear();

        if (QueryResult r = CharacterDatabase.Query("SELECT achievement_id FROM mod_ra_character_unlock WHERE guid = {}", guid))
        {
            do
            {
                Field* f = r->Fetch();
                data.unlocked.insert(f[0].Get<uint32>());
            } while (r->NextRow());
        }

        if (QueryResult r = CharacterDatabase.Query("SELECT achievement_id, counter FROM mod_ra_character_progress WHERE guid = {}", guid))
        {
            do
            {
                Field* f = r->Fetch();
                data.progress[f[0].Get<uint32>()] = f[1].Get<uint32>();
            } while (r->NextRow());
        }
    }

    void Award(Player* player, RaPlayerData& data, RaAchievement const& a)
    {
        uint32 guid = player->GetGUID().GetCounter();
        data.unlocked.insert(a.id);
        CharacterDatabase.Execute("REPLACE INTO mod_ra_character_unlock (guid, achievement_id) VALUES ({}, {})", guid, a.id);
        ChatHandler(player->GetSession()).SendSysMessage("Achievement unlocked: " + a.name + " - " + a.description);
        LOG_INFO("module.retroachievements", "mod-retroachievements: player {} unlocked achievement {} ({})", player->GetName(), a.id, a.name);
    }

    void Evaluate(Player* player, uint8 type, uint32 key, uint32 amount)
    {
        if (!player || !player->GetSession())
            return;

        uint32 guid = player->GetGUID().GetCounter();
        auto it = sPlayers.find(guid);
        if (it == sPlayers.end())
            return;

        RaPlayerData& data = it->second;
        for (RaAchievement const& a : sAchievements)
        {
            if (a.type != type || data.unlocked.count(a.id))
                continue;

            if (type == RA_COND_REACH_LEVEL)
            {
                if (key >= a.value1)
                    Award(player, data, a);
                continue;
            }

            if (a.value1 != key)
                continue;

            uint32 needed = std::max<uint32>(a.value2, 1);
            uint32& counter = data.progress[a.id];
            counter += amount;
            CharacterDatabase.Execute("REPLACE INTO mod_ra_character_progress (guid, achievement_id, counter) VALUES ({}, {}, {})", guid, a.id, counter);
            if (counter >= needed)
                Award(player, data, a);
        }
    }
}

class RetroAchievementsPlayerScript : public PlayerScript
{
public:
    RetroAchievementsPlayerScript() : PlayerScript("RetroAchievementsPlayerScript",
        {
            PLAYERHOOK_ON_LOGIN,
            PLAYERHOOK_ON_LEVEL_CHANGED,
            PLAYERHOOK_ON_CREATURE_KILL,
            PLAYERHOOK_ON_PLAYER_COMPLETE_QUEST,
            PLAYERHOOK_ON_UPDATE_AREA,
            PLAYERHOOK_ON_LOOT_ITEM
        }) { }

    void OnPlayerLogin(Player* player) override
    {
        LoadAchievements(); // reload every login so new table rows need no rebuild
        LoadPlayer(player->GetGUID().GetCounter());
        LOG_INFO("module.retroachievements", "mod-retroachievements: player {} logged in", player->GetName());
    }

    void OnPlayerLevelChanged(Player* player, uint8 /*oldlevel*/) override
    {
        Evaluate(player, RA_COND_REACH_LEVEL, player->GetLevel(), 1);
    }

    void OnPlayerCreatureKill(Player* killer, Creature* killed) override
    {
        if (killed)
            Evaluate(killer, RA_COND_KILL_CREATURE, killed->GetEntry(), 1);
    }

    void OnPlayerCompleteQuest(Player* player, Quest const* quest) override
    {
        if (quest)
            Evaluate(player, RA_COND_COMPLETE_QUEST, quest->GetQuestId(), 1);
    }

    void OnPlayerUpdateArea(Player* player, uint32 /*oldArea*/, uint32 newArea) override
    {
        Evaluate(player, RA_COND_ENTER_AREA, newArea, 1);
    }

    void OnPlayerLootItem(Player* player, Item* item, uint32 count, ObjectGuid /*lootguid*/) override
    {
        if (item)
            Evaluate(player, RA_COND_LOOT_ITEM, item->GetEntry(), count);
    }
};

void AddRetroAchievementsPlayerScripts()
{
    new RetroAchievementsPlayerScript();
}
