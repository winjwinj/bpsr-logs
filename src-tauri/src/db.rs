use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{Row, sqlite::SqlitePool};

use crate::live::opcodes_models::{CombatStats, Encounter};
use blueprotobuf_lib::blueprotobuf::EEntityType;

pub type DbConnection = SqlitePool;

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

pub fn init_db() -> Vec<tauri_plugin_sql::Migration> {
    // Return migrations for tauri-plugin-sql to run
    // The plugin will handle connection pooling and schema creation
    vec![
        tauri_plugin_sql::Migration {
            version: 1,
            description: "create_encounters_players_abilities_tables",
            sql: r#"
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
            "#,
            kind: tauri_plugin_sql::MigrationKind::Up,
        },
        tauri_plugin_sql::Migration {
            version: 2,
            description: "add_player_uid_column",
            sql: "ALTER TABLE players ADD COLUMN player_uid INTEGER NOT NULL DEFAULT 0;",
            kind: tauri_plugin_sql::MigrationKind::Up,
        },
        tauri_plugin_sql::Migration {
            version: 3,
            description: "add_ability_score_column",
            sql: "ALTER TABLE players ADD COLUMN ability_score INTEGER;",
            kind: tauri_plugin_sql::MigrationKind::Up,
        },
        tauri_plugin_sql::Migration {
            version: 4,
            description: "create_players_metadata_table",
            sql: r#"
                CREATE TABLE IF NOT EXISTS players_metadata (
                    player_uid INTEGER PRIMARY KEY,
                    name TEXT,
                    class TEXT,
                    class_spec TEXT,
                    ability_score INTEGER,
                    last_seen TEXT
                );
            "#,
            kind: tauri_plugin_sql::MigrationKind::Up,
        },
        tauri_plugin_sql::Migration {
            version: 5,
            description: "add_index_to_players_metadata",
            sql: r#"
                CREATE INDEX IF NOT EXISTS idx_players_metadata_player_uid ON players_metadata(player_uid);
            "#,
            kind: tauri_plugin_sql::MigrationKind::Up,
        },
    ]
}

pub async fn save_encounter(
    db: &DbConnection,
    encounter: &Encounter,
) -> Result<i64, Box<dyn std::error::Error>> {
    // Start a transaction for the entire save operation
    // This batches all writes and improves performance dramatically
    let mut tx = db.begin().await?;
    
    let time_elapsed_ms = encounter.time_last_combat_packet_ms as i64 - encounter.time_fight_start_ms as i64;
    let end_time = Utc::now().to_rfc3339();
    
    // Calculate start time by subtracting elapsed time from end time
    let start_time_utc = Utc::now() - chrono::Duration::milliseconds(time_elapsed_ms);
    let start_time_str = start_time_utc.to_rfc3339();
    
    log::info!("Saving encounter: damage={}, healing={}, duration_ms={}, elapsed_ms={}", 
        encounter.dmg_stats.value, encounter.heal_stats.value, 
        encounter.time_fight_start_ms, time_elapsed_ms);

    // Pre-fetch historical metadata for all players
    let mut player_metadata_cache: std::collections::HashMap<i64, Option<PlayerMetadata>> = std::collections::HashMap::new();
    for (&entity_uid, entity) in &encounter.entity_uid_to_entity {
        if entity.entity_type != EEntityType::EntChar {
            continue;
        }
        if entity.dmg_stats.value == 0 && entity.heal_stats.value == 0 {
            continue;
        }
        
        // Check if we should fetch metadata from history
        let has_name = entity.name.is_some() && entity.name.as_ref().is_some_and(|n| crate::db::is_valid_player_name(n));
        let has_class = entity.class.is_some();
        let has_spec = entity.class_spec.is_some();
        
        // Fetch metadata if we might need it (missing name, class, or spec)
        if !has_name || !has_class || !has_spec {
            log::debug!("Fetching metadata for player {}: has_name={}, has_class={}, has_spec={}", entity_uid, has_name, has_class, has_spec);
            match lookup_player_metadata(db, entity_uid).await {
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
    
    // Collect player metadata to upsert after transaction commits
    let mut player_metadata_to_upsert: Vec<(i64, String, Option<String>, Option<String>, Option<i32>)> = Vec::new();

    // Insert the encounter
    sqlx::query(
        "INSERT INTO encounters (start_time, end_time, duration_ms, total_damage, total_healing)
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&start_time_str)
    .bind(&end_time)
    .bind(time_elapsed_ms)
    .bind(encounter.dmg_stats.value)
    .bind(encounter.heal_stats.value)
    .execute(&mut *tx)
    .await?;

    // Get the last inserted encounter ID
    let row = sqlx::query(
        "SELECT id FROM encounters WHERE start_time = ? ORDER BY id DESC LIMIT 1"
    )
    .bind(&start_time_str)
    .fetch_one(&mut *tx)
    .await?;

    let encounter_id: i64 = row.get("id");

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
        let name_is_incomplete = !crate::db::is_valid_player_name(&player_name);
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

        // Try to insert player, but if it already exists in this encounter, update them instead
        let insert_result = sqlx::query(
            "INSERT INTO players (
                encounter_id, player_uid, name, class, class_spec, ability_score,
                total_damage, damage_hits, crit_value, crit_hits, lucky_value, lucky_hits,
                total_healing, healing_hits
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(encounter_id)
        .bind(entity_uid)
        .bind(&player_name)
        .bind(&class)
        .bind(&class_spec)
        .bind(entity.ability_score)
        .bind(entity.dmg_stats.value)
        .bind(entity.dmg_stats.hits)
        .bind(entity.dmg_stats.crit_value)
        .bind(entity.dmg_stats.crit_hits)
        .bind(entity.dmg_stats.lucky_value)
        .bind(entity.dmg_stats.lucky_hits)
        .bind(entity.heal_stats.value)
        .bind(entity.heal_stats.hits)
        .execute(&mut *tx)
        .await;

        let player_id = match insert_result {
            Ok(_) => {
                // Get the player_id after insert
                let row = sqlx::query(
                    "SELECT id FROM players WHERE encounter_id = ? AND player_uid = ? ORDER BY id DESC LIMIT 1"
                )
                .bind(encounter_id)
                .bind(entity_uid)
                .fetch_one(&mut *tx)
                .await?;
                let id: i64 = row.get("id");
                id
            }
            Err(e) => {
                // If insertion failed (likely due to duplicate), try to update instead
                log::debug!("Player insert failed (possibly duplicate), attempting update: {}", e);
                sqlx::query(
                    "UPDATE players SET
                        name = ?,
                        class = CASE WHEN ? IS NOT NULL THEN ? ELSE class END,
                        class_spec = CASE WHEN ? IS NOT NULL THEN ? ELSE class_spec END,
                        ability_score = CASE WHEN ? IS NOT NULL THEN ? ELSE ability_score END,
                        total_damage = ?,
                        damage_hits = ?,
                        crit_value = ?,
                        crit_hits = ?,
                        lucky_value = ?,
                        lucky_hits = ?,
                        total_healing = ?,
                        healing_hits = ?
                     WHERE encounter_id = ? AND player_uid = ?"
                )
                .bind(&player_name)
                .bind(class.as_ref().map(|_| 1).unwrap_or(0))
                .bind(&class)
                .bind(class_spec.as_ref().map(|_| 1).unwrap_or(0))
                .bind(&class_spec)
                .bind(entity.ability_score.map(|_| 1).unwrap_or(0))
                .bind(entity.ability_score)
                .bind(entity.dmg_stats.value)
                .bind(entity.dmg_stats.hits)
                .bind(entity.dmg_stats.crit_value)
                .bind(entity.dmg_stats.crit_hits)
                .bind(entity.dmg_stats.lucky_value)
                .bind(entity.dmg_stats.lucky_hits)
                .bind(entity.heal_stats.value)
                .bind(entity.heal_stats.hits)
                .bind(encounter_id)
                .bind(entity_uid)
                .execute(&mut *tx)
                .await?;
                
                // Get the player_id after update
                let row = sqlx::query(
                    "SELECT id FROM players WHERE encounter_id = ? AND player_uid = ? ORDER BY id DESC LIMIT 1"
                )
                .bind(encounter_id)
                .bind(entity_uid)
                .fetch_one(&mut *tx)
                .await?;
                let id: i64 = row.get("id");
                id
            }
        };

        // Save abilities for this player
        for (&skill_id, stats) in &entity.skill_uid_to_dps_stats {
            if stats.value == 0 {
                continue;
            }

            let skill_name = CombatStats::get_skill_name(skill_id);

            sqlx::query(
                "INSERT INTO abilities (
                    player_id, skill_id, skill_name,
                    total_damage, damage_hits, crit_value, crit_hits, lucky_value, lucky_hits
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(player_id)
            .bind(skill_id)
            .bind(&skill_name)
            .bind(stats.value)
            .bind(stats.hits)
            .bind(stats.crit_value)
            .bind(stats.crit_hits)
            .bind(stats.lucky_value)
            .bind(stats.lucky_hits)
            .execute(&mut *tx)
            .await?;
        }

        // Also save healing abilities
        for (&skill_id, stats) in &entity.skill_uid_to_heal_stats {
            if stats.value == 0 {
                continue;
            }

            let skill_name = CombatStats::get_skill_name(skill_id);

            sqlx::query(
                "INSERT INTO abilities (
                    player_id, skill_id, skill_name,
                    total_damage, damage_hits, crit_value, crit_hits, lucky_value, lucky_hits
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(player_id)
            .bind(skill_id)
            .bind(&skill_name)
            .bind(stats.value)
            .bind(stats.hits)
            .bind(stats.crit_value)
            .bind(stats.crit_hits)
            .bind(stats.lucky_value)
            .bind(stats.lucky_hits)
            .execute(&mut *tx)
            .await?;
        }

        // Upsert to players_metadata table to persist discovered metadata
        // Collect for batch processing after transaction commits
        if crate::db::is_valid_player_name(&player_name) {
            player_metadata_to_upsert.push((
                entity_uid,
                player_name,
                class,
                class_spec,
                entity.ability_score,
            ));
        }
    }

    // Commit the transaction - all writes batched together
    tx.commit().await?;

    // Now upsert all player metadata AFTER transaction is committed
    // This ensures the encounter/player data is already saved before we update metadata
    for (uid, name, class_opt, spec_opt, ability_score) in player_metadata_to_upsert {
        let _ = upsert_player_metadata(
            db,
            uid,
            Some(&name),
            class_opt.as_deref(),
            spec_opt.as_deref(),
            ability_score,
        )
        .await;
    }

    Ok(encounter_id)
}

pub async fn get_all_encounters(db: &DbConnection) -> Result<Vec<EncounterRecord>, Box<dyn std::error::Error>> {
    let encounters_data = sqlx::query(
        "SELECT id, start_time, end_time, duration_ms, total_damage, total_healing
         FROM encounters
         ORDER BY start_time DESC"
    )
    .fetch_all(db)
    .await?;

    let encounters: Vec<EncounterRecord> = encounters_data
        .iter()
        .map(|row| EncounterRecord {
            id: row.get("id"),
            start_time: row.get("start_time"),
            end_time: row.get("end_time"),
            duration_ms: row.get("duration_ms"),
            total_damage: row.get("total_damage"),
            total_healing: row.get("total_healing"),
        })
        .collect();

    log::info!("Retrieved {} encounters from database", encounters.len());
    Ok(encounters)
}

pub async fn get_encounter_detail(db: &DbConnection, encounter_id: i64) -> Result<Option<EncounterDetail>, Box<dyn std::error::Error>> {
    // Get encounter
    let encounter_result = sqlx::query(
        "SELECT id, start_time, end_time, duration_ms, total_damage, total_healing
         FROM encounters
         WHERE id = ?"
    )
    .bind(encounter_id)
    .fetch_optional(db)
    .await?;

    match encounter_result {
        None => Ok(None),
        Some(row) => {
            let encounter = EncounterRecord {
                id: row.get("id"),
                start_time: row.get("start_time"),
                end_time: row.get("end_time"),
                duration_ms: row.get("duration_ms"),
                total_damage: row.get("total_damage"),
                total_healing: row.get("total_healing"),
            };

            // Get players
            let players_data = sqlx::query(
                "SELECT id, encounter_id, COALESCE(player_uid, id) as player_uid_or_id, name, class, class_spec, ability_score,
                        total_damage, damage_hits, crit_value, crit_hits, lucky_value, lucky_hits,
                        total_healing, healing_hits
                 FROM players
                 WHERE encounter_id = ?
                 ORDER BY total_damage DESC"
            )
            .bind(encounter_id)
            .fetch_all(db)
            .await?;

            let mut players: Vec<PlayerRecord> = Vec::new();
            for row in &players_data {
                let player = PlayerRecord {
                    id: row.get("player_uid_or_id"),
                    db_id: row.get("id"),
                    encounter_id: row.get("encounter_id"),
                    name: row.get("name"),
                    class: row.get("class"),
                    class_spec: row.get("class_spec"),
                    ability_score: row.get("ability_score"),
                    total_damage: row.get("total_damage"),
                    damage_hits: row.get("damage_hits"),
                    crit_value: row.get("crit_value"),
                    crit_hits: row.get("crit_hits"),
                    lucky_value: row.get("lucky_value"),
                    lucky_hits: row.get("lucky_hits"),
                    total_healing: row.get("total_healing"),
                    healing_hits: row.get("healing_hits"),
                };
                players.push(player);
            }

            // Get abilities for each player
            let mut player_details = Vec::new();
            for player in players {
                let abilities_data = sqlx::query(
                    "SELECT id, player_id, skill_id, skill_name,
                            total_damage, damage_hits, crit_value, crit_hits, lucky_value, lucky_hits
                     FROM abilities
                     WHERE player_id = ?
                     ORDER BY total_damage DESC"
                )
                .bind(player.db_id)
                .fetch_all(db)
                .await?;

                let mut abilities: Vec<AbilityRecord> = Vec::new();
                for row in &abilities_data {
                    let ability = AbilityRecord {
                        id: row.get("id"),
                        player_id: row.get("player_id"),
                        skill_id: row.get("skill_id"),
                        skill_name: row.get("skill_name"),
                        total_damage: row.get("total_damage"),
                        damage_hits: row.get("damage_hits"),
                        crit_value: row.get("crit_value"),
                        crit_hits: row.get("crit_hits"),
                        lucky_value: row.get("lucky_value"),
                        lucky_hits: row.get("lucky_hits"),
                    };
                    abilities.push(ability);
                }

                player_details.push(PlayerDetail { player, abilities });
            }

            Ok(Some(EncounterDetail {
                encounter,
                players: player_details,
            }))
        }
    }
}

pub async fn delete_encounter(db: &DbConnection, encounter_id: i64) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::query("DELETE FROM encounters WHERE id = ?")
        .bind(encounter_id)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn clear_all_encounters(db: &DbConnection) -> Result<(), Box<dyn std::error::Error>> {
    // Delete all encounter rows (players and abilities will cascade if foreign keys are enabled)
    sqlx::query("DELETE FROM encounters").execute(db).await?;

    // Reset SQLite AUTOINCREMENT counters so IDs start from 1 again.
    let _ = sqlx::query("DELETE FROM sqlite_sequence WHERE name = 'encounters'").execute(db).await;
    let _ = sqlx::query("DELETE FROM sqlite_sequence WHERE name = 'players'").execute(db).await;
    let _ = sqlx::query("DELETE FROM sqlite_sequence WHERE name = 'abilities'").execute(db).await;

    // Also clear the players_metadata table so historical user metadata does not grow unbounded
    let _ = sqlx::query("DELETE FROM players_metadata").execute(db).await;
    let _ = sqlx::query("DELETE FROM sqlite_sequence WHERE name = 'players_metadata'").execute(db).await;

    // Optional: run VACUUM to ensure the database file is compacted
    let _ = sqlx::query("VACUUM;").execute(db).await;

    log::info!("Cleared all encounters, players_metadata and reset autoincrement sequences");

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct PlayerMetadata {
    pub name: String,
    pub class: Option<String>,
    pub class_spec: Option<String>,
    pub ability_score: Option<i32>,
}

/// Check if a name is valid (not empty, not placeholder, not unknown)
pub fn is_valid_player_name(name: &str) -> bool {
    !name.is_empty() 
        && name != "Unknown" 
        && name != "Unknown Name" 
        && !name.contains("unknown")  // Case-insensitive check
}

/// Look up player metadata from history by player UID
/// Returns the most recent player metadata for the given UID
pub async fn lookup_player_metadata(db: &DbConnection, player_uid: i64) -> Result<Option<PlayerMetadata>, Box<dyn std::error::Error>> {
    // First, try the dedicated players_metadata table (single row per player_uid).
    let metadata_result = sqlx::query(
        "SELECT name, class, class_spec, ability_score FROM players_metadata WHERE player_uid = ? LIMIT 1"
    )
    .bind(player_uid)
    .fetch_optional(db)
    .await?;

    if let Some(row) = metadata_result {
        let metadata = PlayerMetadata {
            name: row.get("name"),
            class: row.get("class"),
            class_spec: row.get("class_spec"),
            ability_score: row.get("ability_score"),
        };
        return Ok(Some(metadata));
    }

    // Fallback: look in the players table for the most recent encounter row
    let players_result = sqlx::query(
        "SELECT name, class, class_spec, ability_score FROM players 
         WHERE player_uid = ? 
         ORDER BY id DESC 
         LIMIT 1"
    )
    .bind(player_uid)
    .fetch_optional(db)
    .await?;

    if let Some(row) = players_result {
        let metadata = PlayerMetadata {
            name: row.get("name"),
            class: row.get("class"),
            class_spec: row.get("class_spec"),
            ability_score: row.get("ability_score"),
        };
        Ok(Some(metadata))
    } else {
        Ok(None)
    }
}

/// Upsert latest metadata for a player into players_metadata
pub async fn upsert_player_metadata(
    db: &DbConnection,
    player_uid: i64,
    name: Option<&str>,
    class: Option<&str>,
    class_spec: Option<&str>,
    ability_score: Option<i32>,
) -> Result<(), Box<dyn std::error::Error>> {
    let now = Utc::now().to_rfc3339();
    
    // Only accept valid names - never store placeholder names like "Unknown" or "Unknown Name"
    let valid_name = name.filter(|n| is_valid_player_name(n));
    
    // Optimized upsert that never overwrites good data with bad data
    // Only update fields where we have valid new data
    sqlx::query(
        "INSERT INTO players_metadata (player_uid, name, class, class_spec, ability_score, last_seen)
         VALUES (?, ?, ?, ?, ?, ?)
         ON CONFLICT(player_uid) DO UPDATE SET
            name = CASE WHEN excluded.name IS NOT NULL THEN excluded.name ELSE players_metadata.name END,
            class = CASE WHEN excluded.class IS NOT NULL THEN excluded.class ELSE players_metadata.class END,
            class_spec = CASE WHEN excluded.class_spec IS NOT NULL THEN excluded.class_spec ELSE players_metadata.class_spec END,
            ability_score = CASE WHEN excluded.ability_score IS NOT NULL THEN excluded.ability_score ELSE players_metadata.ability_score END,
            last_seen = excluded.last_seen"
    )
    .bind(player_uid)
    .bind(valid_name)
    .bind(class)
    .bind(class_spec)
    .bind(ability_score)
    .bind(&now)
    .execute(db)
    .await?;
    Ok(())
}

/// Look up ability_score from history by player UID
/// Returns the most recent non-NULL ability_score for the given UID
pub async fn lookup_player_ability_score_from_history(db: &DbConnection, player_uid: i64) -> Result<Option<i32>, Box<dyn std::error::Error>> {
    let result = sqlx::query(
        "SELECT ability_score FROM players 
         WHERE player_uid = ? AND ability_score IS NOT NULL
         ORDER BY id DESC 
         LIMIT 1"
    )
    .bind(player_uid)
    .fetch_optional(db)
    .await?;

    Ok(result.map(|row| row.get("ability_score")))
}

/// Look up class_spec from history by player UID
/// Returns the most recent non-NULL class_spec for the given UID
pub async fn lookup_player_class_spec_from_history(db: &DbConnection, player_uid: i64) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let result = sqlx::query(
        "SELECT class_spec FROM players 
         WHERE player_uid = ? AND class_spec IS NOT NULL
         ORDER BY id DESC 
         LIMIT 1"
    )
    .bind(player_uid)
    .fetch_optional(db)
    .await?;

    Ok(result.map(|row| row.get("class_spec")))
}
