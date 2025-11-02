use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

use crate::live::opcodes_models::{CombatStats, Encounter};
use blueprotobuf_lib::blueprotobuf::EEntityType;

pub type DbConnection = Mutex<Connection>;

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct EncounterRecord {
    pub id: i64,
    pub start_time: String,
    pub end_time: String,
    pub duration_ms: i64,
    pub total_damage: i64,
    pub total_healing: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct PlayerRecord {
    pub id: i64,                    // Database auto-increment ID (internal use only)
    #[serde(skip)]
    pub db_id: i64,                 // Database primary key (for internal queries)
    pub encounter_id: i64,
    pub name: String,
    pub class: Option<String>,
    pub class_spec: Option<String>,
    pub ability_score: Option<i32>,
    pub total_damage: i64,
    pub damage_hits: i64,
    pub crit_value: i64,
    pub crit_hits: i64,
    pub lucky_value: i64,
    pub lucky_hits: i64,
    pub total_healing: i64,
    pub healing_hits: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct AbilityRecord {
    pub id: i64,
    pub player_id: i64,
    pub skill_id: i32,
    pub skill_name: String,
    pub total_damage: i64,
    pub damage_hits: i64,
    pub crit_value: i64,
    pub crit_hits: i64,
    pub lucky_value: i64,
    pub lucky_hits: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct EncounterDetail {
    pub encounter: EncounterRecord,
    pub players: Vec<PlayerDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct PlayerDetail {
    pub player: PlayerRecord,
    pub abilities: Vec<AbilityRecord>,
}

pub fn init_db(db_path: PathBuf) -> SqliteResult<DbConnection> {
    let conn = Connection::open(db_path)?;
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS encounters (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            start_time TEXT NOT NULL,
            end_time TEXT NOT NULL,
            duration_ms INTEGER NOT NULL,
            total_damage INTEGER NOT NULL,
            total_healing INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS players (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            encounter_id INTEGER NOT NULL,
            player_uid INTEGER NOT NULL,
            name TEXT NOT NULL,
            class TEXT,
            class_spec TEXT,
            ability_score INTEGER,
            total_damage INTEGER NOT NULL,
            damage_hits INTEGER NOT NULL,
            crit_value INTEGER NOT NULL,
            crit_hits INTEGER NOT NULL,
            lucky_value INTEGER NOT NULL,
            lucky_hits INTEGER NOT NULL,
            total_healing INTEGER NOT NULL,
            healing_hits INTEGER NOT NULL,
            FOREIGN KEY (encounter_id) REFERENCES encounters (id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS abilities (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            player_id INTEGER NOT NULL,
            skill_id INTEGER NOT NULL,
            skill_name TEXT NOT NULL,
            total_damage INTEGER NOT NULL,
            damage_hits INTEGER NOT NULL,
            crit_value INTEGER NOT NULL,
            crit_hits INTEGER NOT NULL,
            lucky_value INTEGER NOT NULL,
            lucky_hits INTEGER NOT NULL,
            FOREIGN KEY (player_id) REFERENCES players (id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_players_encounter_id ON players(encounter_id);
        CREATE INDEX IF NOT EXISTS idx_abilities_player_id ON abilities(player_id);
        ",
    )?;

    // Migration: Add player_uid column if it doesn't exist
    // This handles databases created before player_uid was added
    let column_exists: bool = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('players') WHERE name='player_uid'",
        [],
        |row| {
            let count: i32 = row.get(0)?;
            Ok(count > 0)
        },
    ).unwrap_or(false);
    
    if !column_exists {
        log::info!("Adding player_uid column to existing players table");
        conn.execute(
            "ALTER TABLE players ADD COLUMN player_uid INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
    }

    // Migration: Add ability_score column if it doesn't exist
    let ability_score_exists: bool = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('players') WHERE name='ability_score'",
        [],
        |row| {
            let count: i32 = row.get(0)?;
            Ok(count > 0)
        },
    ).unwrap_or(false);
    
    if !ability_score_exists {
        log::info!("Adding ability_score column to existing players table");
        conn.execute(
            "ALTER TABLE players ADD COLUMN ability_score INTEGER",
            [],
        )?;
    }

    Ok(Mutex::new(conn))
}

pub fn save_encounter(
    db: &DbConnection,
    encounter: &Encounter,
) -> SqliteResult<i64> {
    let time_elapsed_ms = encounter.time_last_combat_packet_ms as i64 - encounter.time_fight_start_ms as i64;
    let end_time = Utc::now().to_rfc3339();
    
    // Calculate start time by subtracting elapsed time from end time
    let start_time_utc = Utc::now() - chrono::Duration::milliseconds(time_elapsed_ms);
    let start_time_str = start_time_utc.to_rfc3339();
    
    log::info!("Saving encounter: damage={}, healing={}, duration_ms={}, elapsed_ms={}", 
        encounter.dmg_stats.value, encounter.heal_stats.value, 
        encounter.time_fight_start_ms, time_elapsed_ms);

    // Pre-fetch historical metadata for all players BEFORE acquiring the write lock
    let mut player_metadata_cache: std::collections::HashMap<i64, Option<PlayerMetadata>> = std::collections::HashMap::new();
    for (&entity_uid, entity) in &encounter.entity_uid_to_entity {
        if entity.entity_type != EEntityType::EntChar {
            continue;
        }
        if entity.dmg_stats.value == 0 && entity.heal_stats.value == 0 {
            continue;
        }
        
        // Check if we should fetch metadata from history
        let has_name = entity.name.is_some() && entity.name.as_ref().is_some_and(|n| !n.is_empty() && n != "Unknown");
        let has_class = entity.class.is_some();
        let has_spec = entity.class_spec.is_some();
        
        // Fetch metadata if we might need it (missing name, class, or spec)
        if !has_name || !has_class || !has_spec {
            log::debug!("Fetching metadata for player {}: has_name={}, has_class={}, has_spec={}", entity_uid, has_name, has_class, has_spec);
            match lookup_player_metadata(db, entity_uid) {
                Ok(metadata) => {
                    player_metadata_cache.insert(entity_uid, metadata);
                }
                Err(e) => {
                    log::warn!("Failed to lookup metadata for player {}: {}", entity_uid, e);
                    player_metadata_cache.insert(entity_uid, None);
                }
            }
        }
    }

    // Now acquire the lock once and do all writes
    let conn = db.lock().unwrap();

    conn.execute(
        "INSERT INTO encounters (start_time, end_time, duration_ms, total_damage, total_healing)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![&start_time_str, &end_time, time_elapsed_ms, encounter.dmg_stats.value, encounter.heal_stats.value],
    )?;

    let encounter_id = conn.last_insert_rowid();

    // Save all players and their abilities
    for (&entity_uid, entity) in &encounter.entity_uid_to_entity {
        if entity.entity_type != EEntityType::EntChar {
            continue;
        }

        if entity.dmg_stats.value == 0 && entity.heal_stats.value == 0 {
            continue;
        }

        // Use current session data, but fall back to historical metadata if missing
        let mut player_name = entity.name.clone().unwrap_or_else(|| "Unknown".to_string());
        let mut class = entity.class.map(|c| crate::live::opcodes_models::class::get_class_name(c));
        let mut class_spec = entity.class_spec.map(|cs| crate::live::opcodes_models::class::get_class_spec(cs));

        // Special handling for the local player - use SyncContainerData if available
        if let Some(local_player_data) = &encounter.local_player {
            if let Some(v_data) = &local_player_data.v_data {
                if let Some(player_uid_from_sync) = v_data.char_id {
                    if player_uid_from_sync == entity_uid {
                        // This is the local player, use the SyncContainerData
                        if let Some(char_base) = &v_data.char_base {
                            if let Some(name) = &char_base.name {
                                player_name = name.clone();
                                log::info!("Using local_player SyncContainerData for name: {}", player_name);
                            }
                        }
                        if let Some(profession_id) = v_data.profession_list.as_ref().and_then(|p| p.cur_profession_id) {
                            let local_class = crate::live::opcodes_models::class::Class::from(profession_id);
                            class = Some(crate::live::opcodes_models::class::get_class_name(local_class));
                            log::info!("Using local_player profession for class: {:?}", class);
                        }
                    }
                }
            }
        }

        // If we still don't have complete name/spec, try to use cached historical metadata
        let name_is_incomplete = player_name.is_empty() || player_name == "Unknown";
        if name_is_incomplete || class.is_none() || class_spec.is_none() {
            if let Some(Some(historical_metadata)) = player_metadata_cache.get(&entity_uid) {
                log::info!("Using historical metadata for player {}: {} (had: name={}, class={:?}, spec={:?})", 
                    entity_uid, historical_metadata.name, player_name, class, class_spec);
                if name_is_incomplete {
                    player_name = historical_metadata.name.clone();
                }
                if class.is_none() {
                    class = historical_metadata.class.clone();
                }
                if class_spec.is_none() {
                    class_spec = historical_metadata.class_spec.clone();
                }
            }
        }

        log::info!("Saving player: uid={}, name={}, class={:?}, spec={:?}, ability_score={:?}", entity_uid, player_name, class, class_spec, entity.ability_score);

        // Try to insert, but if the player already exists in this encounter, update them instead
        let result = conn.execute(
            "INSERT INTO players (
                encounter_id, player_uid, name, class, class_spec, ability_score,
                total_damage, damage_hits, crit_value, crit_hits, lucky_value, lucky_hits,
                total_healing, healing_hits
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                encounter_id,
                entity_uid,
                player_name,
                class,
                class_spec,
                entity.ability_score,
                entity.dmg_stats.value,
                entity.dmg_stats.hits,
                entity.dmg_stats.crit_value,
                entity.dmg_stats.crit_hits,
                entity.dmg_stats.lucky_value,
                entity.dmg_stats.lucky_hits,
                entity.heal_stats.value,
                entity.heal_stats.hits,
            ],
        );

        let player_id = match result {
            Ok(_) => conn.last_insert_rowid(),
            Err(e) => {
                // If insertion failed (likely due to duplicate), try to update instead
                log::debug!("Player insert failed (possibly duplicate), attempting update: {}", e);
                conn.execute(
                    "UPDATE players SET
                        name = ?1,
                        class = CASE WHEN ?2 IS NOT NULL THEN ?2 ELSE class END,
                        class_spec = CASE WHEN ?3 IS NOT NULL THEN ?3 ELSE class_spec END,
                        ability_score = CASE WHEN ?4 IS NOT NULL THEN ?4 ELSE ability_score END,
                        total_damage = ?5,
                        damage_hits = ?6,
                        crit_value = ?7,
                        crit_hits = ?8,
                        lucky_value = ?9,
                        lucky_hits = ?10,
                        total_healing = ?11,
                        healing_hits = ?12
                     WHERE encounter_id = ?13 AND player_uid = ?14",
                    params![
                        player_name,
                        class,
                        class_spec,
                        entity.ability_score,
                        entity.dmg_stats.value,
                        entity.dmg_stats.hits,
                        entity.dmg_stats.crit_value,
                        entity.dmg_stats.crit_hits,
                        entity.dmg_stats.lucky_value,
                        entity.dmg_stats.lucky_hits,
                        entity.heal_stats.value,
                        entity.heal_stats.hits,
                        encounter_id,
                        entity_uid,
                    ],
                )?;
                // Get the player_id after update
                conn.query_row(
                    "SELECT id FROM players WHERE encounter_id = ?1 AND player_uid = ?2",
                    params![encounter_id, entity_uid],
                    |row| row.get(0),
                )?
            }
        };

        // Save abilities for this player
        for (&skill_id, stats) in &entity.skill_uid_to_dps_stats {
            if stats.value == 0 {
                continue;
            }

            let skill_name = CombatStats::get_skill_name(skill_id);

            conn.execute(
                "INSERT INTO abilities (
                    player_id, skill_id, skill_name,
                    total_damage, damage_hits, crit_value, crit_hits, lucky_value, lucky_hits
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    player_id,
                    skill_id,
                    skill_name,
                    stats.value,
                    stats.hits,
                    stats.crit_value,
                    stats.crit_hits,
                    stats.lucky_value,
                    stats.lucky_hits,
                ],
            )?;
        }

        // Also save healing abilities
        for (&skill_id, stats) in &entity.skill_uid_to_heal_stats {
            if stats.value == 0 {
                continue;
            }

            let skill_name = CombatStats::get_skill_name(skill_id);

            conn.execute(
                "INSERT INTO abilities (
                    player_id, skill_id, skill_name,
                    total_damage, damage_hits, crit_value, crit_hits, lucky_value, lucky_hits
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    player_id,
                    skill_id,
                    skill_name,
                    stats.value,
                    stats.hits,
                    stats.crit_value,
                    stats.crit_hits,
                    stats.lucky_value,
                    stats.lucky_hits,
                ],
            )?;
        }
    }

    Ok(encounter_id)
}

pub fn get_all_encounters(db: &DbConnection) -> SqliteResult<Vec<EncounterRecord>> {
    let conn = db.lock().unwrap();

    let mut stmt = conn.prepare(
        "SELECT id, start_time, end_time, duration_ms, total_damage, total_healing
         FROM encounters
         ORDER BY start_time DESC",
    )?;

    let encounters = stmt
        .query_map([], |row| {
            Ok(EncounterRecord {
                id: row.get(0)?,
                start_time: row.get(1)?,
                end_time: row.get(2)?,
                duration_ms: row.get(3)?,
                total_damage: row.get(4)?,
                total_healing: row.get(5)?,
            })
        })?
        .collect::<SqliteResult<Vec<_>>>()?;

    log::info!("Retrieved {} encounters from database", encounters.len());
    Ok(encounters)
}

pub fn get_encounter_detail(db: &DbConnection, encounter_id: i64) -> SqliteResult<Option<EncounterDetail>> {
    let conn = db.lock().unwrap();

    // Get encounter
    let encounter = conn
        .query_row(
            "SELECT id, start_time, end_time, duration_ms, total_damage, total_healing
             FROM encounters
             WHERE id = ?1",
            params![encounter_id],
            |row| {
                Ok(EncounterRecord {
                    id: row.get(0)?,
                    start_time: row.get(1)?,
                    end_time: row.get(2)?,
                    duration_ms: row.get(3)?,
                    total_damage: row.get(4)?,
                    total_healing: row.get(5)?,
                })
            },
        )
        .optional()?;

    match encounter {
        None => Ok(None),
        Some(encounter) => {
            // Get players
            let mut stmt = conn.prepare(
                "SELECT id, encounter_id, COALESCE(player_uid, id) as player_uid_or_id, name, class, class_spec, ability_score,
                        total_damage, damage_hits, crit_value, crit_hits, lucky_value, lucky_hits,
                        total_healing, healing_hits
                 FROM players
                 WHERE encounter_id = ?1
                 ORDER BY total_damage DESC",
            )?;

            let players = stmt
                .query_map(params![encounter_id], |row| {
                    Ok(PlayerRecord {
                        id: row.get(2)?,                // player_uid or id as fallback
                        db_id: row.get(0)?,             // auto-increment id
                        encounter_id: row.get(1)?,
                        name: row.get(3)?,
                        class: row.get(4)?,
                        class_spec: row.get(5)?,
                        ability_score: row.get(6)?,
                        total_damage: row.get(7)?,
                        damage_hits: row.get(8)?,
                        crit_value: row.get(9)?,
                        crit_hits: row.get(10)?,
                        lucky_value: row.get(11)?,
                        lucky_hits: row.get(12)?,
                        total_healing: row.get(13)?,
                        healing_hits: row.get(14)?,
                    })
                })?
                .collect::<SqliteResult<Vec<_>>>()?;

            // Get abilities for each player
            let mut player_details = Vec::new();
            for player in players {
                let mut stmt = conn.prepare(
                    "SELECT id, player_id, skill_id, skill_name,
                            total_damage, damage_hits, crit_value, crit_hits, lucky_value, lucky_hits
                     FROM abilities
                     WHERE player_id = ?1
                     ORDER BY total_damage DESC",
                )?;

                let abilities = stmt
                    .query_map(params![player.db_id], |row| {
                        Ok(AbilityRecord {
                            id: row.get(0)?,
                            player_id: row.get(1)?,
                            skill_id: row.get(2)?,
                            skill_name: row.get(3)?,
                            total_damage: row.get(4)?,
                            damage_hits: row.get(5)?,
                            crit_value: row.get(6)?,
                            crit_hits: row.get(7)?,
                            lucky_value: row.get(8)?,
                            lucky_hits: row.get(9)?,
                        })
                    })?
                    .collect::<SqliteResult<Vec<_>>>()?;

                player_details.push(PlayerDetail { player, abilities });
            }

            Ok(Some(EncounterDetail {
                encounter,
                players: player_details,
            }))
        }
    }
}

pub fn delete_encounter(db: &DbConnection, encounter_id: i64) -> SqliteResult<()> {
    let conn = db.lock().unwrap();
    conn.execute("DELETE FROM encounters WHERE id = ?1", params![encounter_id])?;
    Ok(())
}

pub fn clear_all_encounters(db: &DbConnection) -> SqliteResult<()> {
    let conn = db.lock().unwrap();
    // Delete all encounter rows (players and abilities will cascade if foreign keys are enabled)
    conn.execute("DELETE FROM encounters", [])?;

    // Reset SQLite AUTOINCREMENT counters so IDs start from 1 again.
    // sqlite_sequence is only present when AUTOINCREMENT is used on INTEGER PRIMARY KEY columns.
    // Remove sequence entries for encounters, players and abilities to fully reset state.
    let _ = conn.execute("DELETE FROM sqlite_sequence WHERE name = 'encounters'", []);
    let _ = conn.execute("DELETE FROM sqlite_sequence WHERE name = 'players'", []);
    let _ = conn.execute("DELETE FROM sqlite_sequence WHERE name = 'abilities'", []);

    // Optional: run VACUUM to ensure the database file is compacted and sqlite_sequence changes apply cleanly.
    // VACUUM can be somewhat expensive but this is a user-initiated clear operation and should be fine.
    let _ = conn.execute_batch("VACUUM;") ;

    log::info!("Cleared all encounters and reset autoincrement sequences");

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerMetadata {
    pub name: String,
    pub class: Option<String>,
    pub class_spec: Option<String>,
}

/// Look up player metadata from history by player UID
/// Returns the most recent player metadata for the given UID
pub fn lookup_player_metadata(db: &DbConnection, player_uid: i64) -> SqliteResult<Option<PlayerMetadata>> {
    let conn = db.lock().unwrap();
    
    let mut stmt = conn.prepare(
        "SELECT name, class, class_spec FROM players 
         WHERE player_uid = ?1 
         ORDER BY encounter_id DESC 
         LIMIT 1"
    )?;
    
    let result = stmt.query_row([player_uid], |row| {
        Ok(PlayerMetadata {
            name: row.get(0)?,
            class: row.get(1)?,
            class_spec: row.get(2)?,
        })
    }).optional()?;
    
    Ok(result)
}

/// Look up ability_score from history by player UID
/// Returns the most recent non-NULL ability_score for the given UID
pub fn lookup_player_ability_score_from_history(db: &DbConnection, player_uid: i64) -> SqliteResult<Option<i32>> {
    let conn = db.lock().unwrap();
    
    conn.query_row(
        "SELECT ability_score FROM players 
         WHERE player_uid = ?1 AND ability_score IS NOT NULL
         ORDER BY encounter_id DESC 
         LIMIT 1",
        [player_uid],
        |row| row.get(0),
    ).optional()
}

/// Look up class_spec from history by player UID
/// Returns the most recent non-NULL class_spec for the given UID
pub fn lookup_player_class_spec_from_history(db: &DbConnection, player_uid: i64) -> SqliteResult<Option<String>> {
    let conn = db.lock().unwrap();
    
    conn.query_row(
        "SELECT class_spec FROM players 
         WHERE player_uid = ?1 AND class_spec IS NOT NULL
         ORDER BY encounter_id DESC 
         LIMIT 1",
        [player_uid],
        |row| row.get(0),
    ).optional()
}
