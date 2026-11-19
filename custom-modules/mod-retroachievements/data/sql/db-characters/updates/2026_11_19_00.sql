-- Remove the old local unlock/progress rows of the retired IDs 90001-90004 (test data only).
DELETE FROM `mod_ra_character_unlock` WHERE `achievement_id` BETWEEN 90001 AND 90004;
DELETE FROM `mod_ra_character_progress` WHERE `achievement_id` BETWEEN 90001 AND 90004;
