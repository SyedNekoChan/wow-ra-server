-- Renumber the custom achievements 90001-90004 to 60001-60004.
-- Reason: character_achievement.achievement is smallint unsigned (max 65535), so IDs above that are not saved.
DELETE FROM `achievement_dbc` WHERE `ID` BETWEEN 90001 AND 90004;
DELETE FROM `mod_ra_achievement` WHERE `id` BETWEEN 90001 AND 90004;

REPLACE INTO `mod_ra_achievement` (`id`,`name`,`description`,`category_id`,`condition_type`,`value1`,`value2`,`points`)
VALUES (60001,'Rookie','Reach level 10',90000,1,10,1,5),
       (60002,'Wolf Hunter','Kill 3 Diseased Young Wolves',90000,2,299,3,5),
       (60003,'Island Reclaimer','Complete Reclaiming Sunstrider Isle',90000,3,8325,1,5),
       (60004,'Welcome to Tranquillien','Enter Tranquillien',90000,4,3488,1,5);

REPLACE INTO achievement_dbc (ID, Faction, Instance_Id, Supercedes, Title_Lang_enUS, Title_Lang_Mask, Description_Lang_enUS, Description_Lang_Mask, Category, Points, Ui_Order, Flags, IconID, Reward_Lang_Mask, Minimum_Criteria, Shares_Criteria) VALUES (60001, -1, -1, 0, 'Rookie', 16712190, 'Reach level 10', 16712190, 90000, 5, 1, 0, 3268, 16712174, 0, 0);
REPLACE INTO achievement_dbc (ID, Faction, Instance_Id, Supercedes, Title_Lang_enUS, Title_Lang_Mask, Description_Lang_enUS, Description_Lang_Mask, Category, Points, Ui_Order, Flags, IconID, Reward_Lang_Mask, Minimum_Criteria, Shares_Criteria) VALUES (60002, -1, -1, 0, 'Wolf Hunter', 16712190, 'Kill 3 Diseased Young Wolves', 16712190, 90000, 5, 2, 0, 3268, 16712174, 0, 0);
REPLACE INTO achievement_dbc (ID, Faction, Instance_Id, Supercedes, Title_Lang_enUS, Title_Lang_Mask, Description_Lang_enUS, Description_Lang_Mask, Category, Points, Ui_Order, Flags, IconID, Reward_Lang_Mask, Minimum_Criteria, Shares_Criteria) VALUES (60003, -1, -1, 0, 'Island Reclaimer', 16712190, 'Complete Reclaiming Sunstrider Isle', 16712190, 90000, 5, 3, 0, 3268, 16712174, 0, 0);
REPLACE INTO achievement_dbc (ID, Faction, Instance_Id, Supercedes, Title_Lang_enUS, Title_Lang_Mask, Description_Lang_enUS, Description_Lang_Mask, Category, Points, Ui_Order, Flags, IconID, Reward_Lang_Mask, Minimum_Criteria, Shares_Criteria) VALUES (60004, -1, -1, 0, 'Welcome to Tranquillien', 16712190, 'Enter Tranquillien', 16712190, 90000, 5, 4, 0, 3268, 16712174, 0, 0);
