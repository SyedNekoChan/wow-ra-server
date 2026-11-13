-- condition_type: 1 reach level, 2 kill creature, 3 complete quest, 4 enter area, 5 loot item
-- value1: level / creature id / quest id / area id / item id.  value2: how many (default 1)
CREATE TABLE IF NOT EXISTS `mod_ra_achievement` (
  `id` INT UNSIGNED NOT NULL,
  `name` VARCHAR(100) NOT NULL,
  `description` VARCHAR(255) NOT NULL,
  `category_id` INT UNSIGNED NOT NULL DEFAULT 90000,
  `condition_type` TINYINT UNSIGNED NOT NULL,
  `value1` INT UNSIGNED NOT NULL DEFAULT 0,
  `value2` INT UNSIGNED NOT NULL DEFAULT 1,
  `points` INT UNSIGNED NOT NULL DEFAULT 0,
  `ra_id` INT UNSIGNED NULL DEFAULT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

REPLACE INTO `mod_ra_achievement` (`id`,`name`,`description`,`category_id`,`condition_type`,`value1`,`value2`,`points`)
VALUES (90001,'Rookie','Reach level 10',90000,1,10,1,5);
