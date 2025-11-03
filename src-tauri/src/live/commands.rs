use crate::live::commands_models::{HeaderInfo, PlayerRow, PlayersWindow, SkillRow, SkillsWindow};
use crate::live::opcodes_models::class::{Class, ClassSpec};
use crate::live::opcodes_models::{class, CombatStats, Encounter, EncounterMutex};
use crate::packets::packet_capture::request_restart;
use crate::db::DbConnection;
use crate::WINDOW_LIVE_LABEL;
use blueprotobuf_lib::blueprotobuf::EEntityType;
use log::info;
use tauri::Manager;
use tauri_plugin_clipboard_manager::ClipboardExt;
use window_vibrancy::{apply_blur, clear_blur};

fn nan_is_zero(value: f64) -> f64 {
    if value.is_nan() || value.is_infinite() {
        0.0
    } else {
        value
    }
}

#[tauri::command]
#[specta::specta]
pub fn enable_blur(app: tauri::AppHandle) {
    if let Some(meter_window) = app.get_webview_window(WINDOW_LIVE_LABEL) {
        apply_blur(&meter_window, Some((10, 10, 10, 50))).ok();
    }
}

#[tauri::command]
#[specta::specta]
pub fn disable_blur(app: tauri::AppHandle) {
    if let Some(meter_window) = app.get_webview_window(WINDOW_LIVE_LABEL) {
        clear_blur(&meter_window).ok();
    }
}

#[tauri::command]
#[specta::specta]
pub fn copy_sync_container_data(app: tauri::AppHandle) {
    let state = app.state::<EncounterMutex>();
    let encounter = state.lock().unwrap();
    if let Some(local_player) = &encounter.local_player
        && let Ok(json) = serde_json::to_string_pretty(local_player)
        && app.clipboard().write_text(json).is_err()
    {
        info!("No SyncContainerData found. Nothing copied to the clipboard.");
    }
}

#[tauri::command]
#[specta::specta]
pub fn get_header_info(state: tauri::State<'_, EncounterMutex>) -> Result<HeaderInfo, String> {
    let encounter = state.lock().unwrap();
    if encounter.dmg_stats.value == 0 {
        return Err("No damage found".to_string());
    }

    let time_elapsed_ms = encounter.time_last_combat_packet_ms - encounter.time_fight_start_ms;
    #[allow(clippy::cast_precision_loss)]
    let time_elapsed_secs = time_elapsed_ms as f64 / 1000.0;

    let encounter_stats = &encounter.dmg_stats;

    #[allow(clippy::cast_precision_loss)]
    Ok(HeaderInfo {
        total_dps: nan_is_zero(encounter_stats.value as f64 / time_elapsed_secs),
        total_dmg: encounter_stats.value as f64,
        elapsed_ms: time_elapsed_ms as f64,
        time_last_combat_packet_ms: encounter.time_last_combat_packet_ms as f64,
    })
}

#[tauri::command]
#[specta::specta]
pub fn hard_reset(state: tauri::State<'_, EncounterMutex>) {
    let mut encounter = state.lock().unwrap();
    encounter.clone_from(&Encounter::default());
    request_restart();
    info!("Hard Reset");
}

#[tauri::command]
#[specta::specta]
pub async fn reset_encounter(
    state: tauri::State<'_, EncounterMutex>,
    db: tauri::State<'_, DbConnection>,
) -> Result<(), String> {
    // Get a copy of the encounter and drop the lock immediately
    let encounter_copy = {
        let encounter = state.lock().unwrap();
        info!("reset_encounter called - dmg: {}, heal: {}", encounter.dmg_stats.value, encounter.heal_stats.value);
        
        // Only save if there's actual combat data
        if encounter.dmg_stats.value > 0 || encounter.heal_stats.value > 0 {
            Some(encounter.clone())
        } else {
            info!("No combat data to save");
            None
        }
    };
    
    // Reset the encounter IMMEDIATELY without waiting for save to complete
    // This allows new packets to be processed right away
    {
        let mut encounter = state.lock().unwrap();
        encounter.clone_from(&Encounter::default());
        info!("encounter reset");
    }
    
    // Spawn background task to save encounter asynchronously
    // This prevents blocking the packet processing thread
    if let Some(encounter_copy) = encounter_copy {
        // Clone the pool for the background task
        let db_pool = (*db).clone();
        tokio::spawn(async move {
            info!("Saving encounter in background...");
            match crate::db::save_encounter(&db_pool, &encounter_copy).await {
                Ok(encounter_id) => {
                    info!("Encounter saved with ID: {}", encounter_id);
                }
                Err(e) => {
                    info!("Failed to save encounter: {}", e);
                }
            }
        });
    }
    
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn toggle_pause_encounter(state: tauri::State<'_, EncounterMutex>) {
    let mut encounter = state.lock().unwrap();
    encounter.is_encounter_paused = !encounter.is_encounter_paused;
}

/// Update a player's metadata in the live encounter cache
/// This allows newly discovered player information (name, class, spec, ability_score) to be
/// reflected immediately in the live UI without waiting for the next full refresh
/// If the player entity doesn't exist yet, it will be created with the provided metadata.
#[tauri::command]
#[specta::specta]
pub fn update_player_metadata(
    state: tauri::State<'_, EncounterMutex>,
    player_uid: i64,
    name: Option<String>,
    player_class: Option<String>,
    player_class_spec: Option<String>,
    ability_score: Option<i32>,
) -> Result<(), String> {
    use crate::live::opcodes_models::class::{Class, ClassSpec};
    use crate::live::opcodes_models::Entity;
    
    let mut encounter = state.lock().unwrap();
    
    // Get or create the entity - this ensures the local player can be updated even before
    // they appear in a combat packet
    let entity = encounter
        .entity_uid_to_entity
        .entry(player_uid)
        .or_insert_with(|| Entity {
            entity_type: blueprotobuf_lib::blueprotobuf::EEntityType::EntChar,
            ..Default::default()
        });
    
    // Update name if provided and non-empty
    if let Some(new_name) = name {
        if !new_name.is_empty() && new_name != "Unknown" && new_name != "Unknown Name" {
            entity.name = Some(new_name.clone());
            info!("Updated player {} name to: {}", player_uid, new_name);
        }
    }
    
    // Update ability_score if provided and valid (>= 0)
    if let Some(score) = ability_score {
        if score >= 0 {
            entity.ability_score = Some(score);
            info!("Updated player {} ability_score to: {}", player_uid, score);
        }
    }
    
    // Update class if provided - parse the class name string to enum
    if let Some(class_name) = player_class {
        if !class_name.is_empty() && class_name != "Unknown Class" {
            let parsed_class = match class_name.as_str() {
                "Stormblade" => Class::Stormblade,
                "Frost Mage" => Class::FrostMage,
                "Wind Knight" => Class::WindKnight,
                "Verdant Oracle" => Class::VerdantOracle,
                "Heavy Guardian" => Class::HeavyGuardian,
                "Marksman" => Class::Marksman,
                "Shield Knight" => Class::ShieldKnight,
                "Beat Performer" => Class::BeatPerformer,
                _ => Class::Unknown,
            };
            entity.class = Some(parsed_class);
            info!("Updated player {} class to: {:?}", player_uid, parsed_class);
        }
    }
    
    // Update class_spec if provided - parse the spec name string to enum
    if let Some(spec_name) = player_class_spec {
        if !spec_name.is_empty() && spec_name != "Unknown Spec" {
            let parsed_spec = match spec_name.as_str() {
                "Iaido" => ClassSpec::Iaido,
                "Moonstrike" => ClassSpec::Moonstrike,
                "Icicle" => ClassSpec::Icicle,
                "Frostbeam" => ClassSpec::Frostbeam,
                "Vanguard" => ClassSpec::Vanguard,
                "Skyward" => ClassSpec::Skyward,
                "Smite" => ClassSpec::Smite,
                "Lifebind" => ClassSpec::Lifebind,
                "Earthfort" => ClassSpec::Earthfort,
                "Block" => ClassSpec::Block,
                "Wildpack" => ClassSpec::Wildpack,
                "Falconry" => ClassSpec::Falconry,
                "Recovery" => ClassSpec::Recovery,
                "Shield" => ClassSpec::Shield,
                "Dissonance" => ClassSpec::Dissonance,
                "Concerto" => ClassSpec::Concerto,
                _ => ClassSpec::Unknown,
            };
            entity.class_spec = Some(parsed_spec);
            info!("Updated player {} class_spec to: {:?}", player_uid, parsed_spec);
        }
    }
    
    Ok(())
}

/// Set the local player UID for the current encounter
/// This is needed to identify which player is "you" on the live meter
/// Normally this is set automatically when SyncToMeDeltaInfo packets arrive,
/// but this command allows explicit setting if needed
#[tauri::command]
#[specta::specta]
pub fn set_local_player_uid(
    state: tauri::State<'_, EncounterMutex>,
    player_uid: i64,
) -> Result<(), String> {
    let mut encounter = state.lock().unwrap();
    encounter.local_player_uid = Some(player_uid);
    info!("Set local player UID to: {}", player_uid);
    Ok(())
}

/// Get the current local player UID for the live encounter
#[tauri::command]
#[specta::specta]
pub fn get_local_player_uid(
    state: tauri::State<'_, EncounterMutex>,
) -> i64 {
    let encounter = state.lock().unwrap();
    encounter.local_player_uid.unwrap_or(-1)
}

/// Persist player metadata to the database immediately
/// This ensures newly discovered metadata is saved even before encounter ends
#[tauri::command]
#[specta::specta]
pub async fn persist_player_metadata(
    db: tauri::State<'_, crate::db::DbConnection>,
    player_uid: i64,
    name: Option<String>,
    player_class: Option<String>,
    player_class_spec: Option<String>,
    ability_score: Option<i32>,
) -> Result<(), String> {
    let name_ref = name.as_deref();
    let class_ref = player_class.as_deref();
    let spec_ref = player_class_spec.as_deref();
    
    crate::db::upsert_player_metadata(&db, player_uid, name_ref, class_ref, spec_ref, ability_score)
        .await
        .map_err(|e| e.to_string())?;
    
    info!("Persisted metadata for player {}: name={:?}, class={:?}, spec={:?}, ability_score={:?}", 
        player_uid, name, player_class, player_class_spec, ability_score);
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub enum StatType {
    Dmg,
    DmgBossOnly,
    Heal,
}

#[tauri::command]
#[specta::specta]
pub async fn get_dps_player_window(
    state: tauri::State<'_, EncounterMutex>,
    db: tauri::State<'_, crate::db::DbConnection>,
) -> Result<PlayersWindow, String> {
    // Clone encounter data and immediately drop the lock
    let encounter_copy = {
        let encounter = state.lock().unwrap();
        encounter.clone()
    };
    get_player_window(encounter_copy, StatType::Dmg, &db).await.map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn get_heal_player_window(
    state: tauri::State<'_, EncounterMutex>,
    db: tauri::State<'_, crate::db::DbConnection>,
) -> Result<PlayersWindow, String> {
    // Clone encounter data and immediately drop the lock
    let encounter_copy = {
        let encounter = state.lock().unwrap();
        encounter.clone()
    };
    get_player_window(encounter_copy, StatType::Heal, &db).await.map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn get_dps_boss_only_player_window(
    state: tauri::State<'_, EncounterMutex>,
    db: tauri::State<'_, crate::db::DbConnection>,
) -> Result<PlayersWindow, String> {
    // Clone encounter data and immediately drop the lock
    let encounter_copy = {
        let encounter = state.lock().unwrap();
        encounter.clone()
    };
    get_player_window(encounter_copy, StatType::DmgBossOnly, &db).await.map_err(|e| e.to_string())
}

/// Tauri command: fetch player metadata (name, class, class_spec) from DB by UID
#[tauri::command]
#[specta::specta]
pub async fn get_player_metadata(
    db: tauri::State<'_, crate::db::DbConnection>,
    player_uid: i64,
) -> Result<Option<crate::db::PlayerMetadata>, String> {
    crate::db::lookup_player_metadata(&db, player_uid)
        .await
        .map_err(|e| e.to_string())
}

pub async fn get_player_window(
    encounter: Encounter,
    stat_type: StatType,
    db: &DbConnection,
) -> Result<PlayersWindow, Box<dyn std::error::Error>> {
    let time_elapsed_ms = encounter.time_last_combat_packet_ms - encounter.time_fight_start_ms;
    #[allow(clippy::cast_precision_loss)]
    let time_elapsed_secs = time_elapsed_ms as f64 / 1000.0;

    // Extract data we need from the encounter - no lock needed since we own it
    let local_player_uid = encounter.local_player_uid;
    let dmg_stats_global = encounter.dmg_stats.clone();
    let dmg_stats_boss_only_global = encounter.dmg_stats_boss_only.clone();
    let heal_stats_global = encounter.heal_stats.clone();

    let entity_data: Vec<_> = encounter
        .entity_uid_to_entity
        .iter()
        .map(|(uid, entity)| {
            (
                *uid,
                entity.entity_type,
                entity.dmg_stats.clone(),
                entity.dmg_stats_boss_only.clone(),
                entity.heal_stats.clone(),
                entity.name.clone(),
                entity.class,
                entity.class_spec,
                entity.ability_score,
            )
        })
        .collect();

    // Now fetch metadata for players that might need it
    // Only fetch if we have players with missing metadata - and do it sparingly
    let mut metadata_cache: std::collections::HashMap<i64, Option<crate::db::PlayerMetadata>> =
        std::collections::HashMap::new();

    let mut players_needing_metadata = Vec::new();
    for (uid, entity_type, _, _, _, name, _, _, _) in &entity_data {
        if *entity_type != EEntityType::EntChar {
            continue;
        }
        let needs_fetch = match name {
            Some(n) => n.is_empty() || n == "Unknown" || n == "Unknown Name",
            None => true,
        };
        if needs_fetch {
            players_needing_metadata.push(*uid);
        }
    }

    // Only do DB lookups if there are players missing metadata
    // and limit to avoid exhausting connection pool during heavy load
    if !players_needing_metadata.is_empty() && players_needing_metadata.len() <= 10 {
        for uid in players_needing_metadata {
            if let Ok(metadata) = crate::db::lookup_player_metadata(db, uid).await {
                metadata_cache.insert(uid, metadata);
            }
        }
    }

    #[allow(clippy::cast_precision_loss)]
    let mut player_window = PlayersWindow {
        player_rows: Vec::new(),
        local_player_uid: local_player_uid.unwrap_or(-1) as f64,
        top_value: 0.0,
    };

    for (entity_uid, entity_type, dmg_stats, dmg_stats_boss_only, heal_stats, name, class, class_spec, ability_score) in entity_data {
        // Select stats per player and encounter
        let (entity_stats, encounter_stats) = match stat_type {
            StatType::Dmg => (&dmg_stats, &dmg_stats_global),
            StatType::DmgBossOnly => (&dmg_stats_boss_only, &dmg_stats_boss_only_global),
            StatType::Heal => (&heal_stats, &heal_stats_global),
        };

        let is_player = entity_type == EEntityType::EntChar;
        let did_damage = entity_stats.value > 0;
        if !is_player || !did_damage {
            continue;
        }
        player_window.top_value = player_window.top_value.max(entity_stats.value as f64);

        // Resolve player name with proper fallback chain:
        // 1. Use current entity name if valid (non-empty, not "Unknown")
        // 2. Otherwise use cached database metadata name if available
        // 3. As last resort, use cached name or "Unknown Name"
        let player_name = {
            let is_valid_name = |n: &str| !n.is_empty() && n != "Unknown" && n != "Unknown Name";
            
            // Try current entity name first
            if let Some(ref n) = name {
                if is_valid_name(n) {
                    n.clone()
                } else {
                    // Fall back to cache
                    metadata_cache
                        .get(&entity_uid)
                        .and_then(|m| m.as_ref().map(|md| md.name.clone()))
                        .filter(|n| is_valid_name(n))
                        .unwrap_or_else(|| String::from("Unknown Name"))
                }
            } else {
                // No current name, try cache
                metadata_cache
                    .get(&entity_uid)
                    .and_then(|m| m.as_ref().map(|md| md.name.clone()))
                    .filter(|n| is_valid_name(n))
                    .unwrap_or_else(|| String::from("Unknown Name"))
            }
        };

        // Resolve class: prefer current entity, fallback to cache, default to Unknown
        let class_name = class
            .map(class::get_class_name)
            .or_else(|| {
                metadata_cache
                    .get(&entity_uid)
                    .and_then(|m| m.as_ref().and_then(|md| md.class.clone()))
            })
            .unwrap_or_else(|| class::get_class_name(Class::Unknown));

        // Resolve class_spec: prefer current entity, fallback to cache, default to Unknown
        let class_spec_name = class_spec
            .map(class::get_class_spec)
            .or_else(|| {
                metadata_cache
                    .get(&entity_uid)
                    .and_then(|m| m.as_ref().and_then(|md| md.class_spec.clone()))
            })
            .unwrap_or_else(|| class::get_class_spec(ClassSpec::Unknown));

        // Resolve ability_score: prefer current entity, fallback to cache
        let final_ability_score = ability_score.or_else(|| {
            metadata_cache
                .get(&entity_uid)
                .and_then(|m| m.as_ref().and_then(|md| md.ability_score))
        });

        #[allow(clippy::cast_precision_loss)]
        let damage_row = PlayerRow {
            uid: entity_uid as f64,
            name: player_name,
            class_name,
            class_spec_name,
            ability_score: final_ability_score.unwrap_or(-1) as f64,
            total_value: entity_stats.value as f64,
            value_per_sec: nan_is_zero(entity_stats.value as f64 / time_elapsed_secs),
            value_pct: nan_is_zero(entity_stats.value as f64 / encounter_stats.value as f64 * 100.0),
            crit_rate: nan_is_zero(entity_stats.crit_hits as f64 / entity_stats.hits as f64 * 100.0),
            crit_value_rate: nan_is_zero(entity_stats.crit_value as f64 / entity_stats.value as f64 * 100.0),
            lucky_rate: nan_is_zero(entity_stats.lucky_hits as f64 / entity_stats.hits as f64 * 100.0),
            lucky_value_rate: nan_is_zero(entity_stats.lucky_value as f64 / entity_stats.value as f64 * 100.0),
            hits: entity_stats.hits as f64,
            hits_per_minute: nan_is_zero(entity_stats.hits as f64 / time_elapsed_secs * 60.0),
        };
        player_window.player_rows.push(damage_row);
    }

    // Sort skills descending by damage dealt
    player_window.player_rows.sort_by(|this_row, other_row| {
        other_row
            .total_value
            .partial_cmp(&this_row.total_value)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(player_window)
}

#[tauri::command]
#[specta::specta]
pub async fn get_dps_skill_window(
    state: tauri::State<'_, EncounterMutex>,
    db: tauri::State<'_, crate::db::DbConnection>,
    player_uid_str: &str,
) -> Result<SkillsWindow, String> {
    let player_uid: i64 = player_uid_str.parse().map_err(|_| "Invalid player_uid")?;
    // Clone encounter data and immediately drop the lock
    let encounter_copy = {
        let encounter = state.lock().unwrap();
        encounter.clone()
    };
    get_skill_window(encounter_copy, &db, player_uid, StatType::Dmg).await
}

#[tauri::command]
#[specta::specta]
pub async fn get_dps_boss_only_skill_window(
    state: tauri::State<'_, EncounterMutex>,
    db: tauri::State<'_, crate::db::DbConnection>,
    player_uid_str: &str,
) -> Result<SkillsWindow, String> {
    let player_uid: i64 = player_uid_str.parse().map_err(|_| "Invalid player_uid")?;
    // Clone encounter data and immediately drop the lock
    let encounter_copy = {
        let encounter = state.lock().unwrap();
        encounter.clone()
    };
    get_skill_window(encounter_copy, &db, player_uid, StatType::DmgBossOnly).await
}

#[tauri::command]
#[specta::specta]
pub async fn get_heal_skill_window(
    state: tauri::State<'_, EncounterMutex>,
    db: tauri::State<'_, crate::db::DbConnection>,
    player_uid_str: &str,
) -> Result<SkillsWindow, String> {
    let player_uid: i64 = player_uid_str.parse().map_err(|_| "Invalid player_uid")?;
    // Clone encounter data and immediately drop the lock
    let encounter_copy = {
        let encounter = state.lock().unwrap();
        encounter.clone()
    };
    get_skill_window(encounter_copy, &db, player_uid, StatType::Heal).await
}

pub async fn get_skill_window(
    encounter: Encounter,
    db: &DbConnection,
    player_uid: i64,
    stat_type: StatType,
) -> Result<SkillsWindow, String> {
    let Some(player) = encounter.entity_uid_to_entity.get(&player_uid) else {
        return Err(format!("Could not find player with uid {player_uid}"));
    };

    let time_elapsed_ms = encounter.time_last_combat_packet_ms - encounter.time_fight_start_ms;
    #[allow(clippy::cast_precision_loss)]
    let time_elapsed_secs = time_elapsed_ms as f64 / 1000.0;

    let (player_stats, encounter_stats, skill_uid_to_stats) = match stat_type {
        StatType::Dmg => (&player.dmg_stats, &encounter.dmg_stats, &player.skill_uid_to_dps_stats),
        StatType::DmgBossOnly => (&player.dmg_stats_boss_only, &encounter.dmg_stats_boss_only, &player.skill_uid_to_dps_stats_boss_only),
        StatType::Heal => (&player.heal_stats, &encounter.heal_stats, &player.skill_uid_to_heal_stats),
    };

    // Fetch metadata for this player if needed
    let metadata = if player.name.is_none() || player.name.as_ref().map_or(false, |n| n.is_empty() || n == "Unknown" || n == "Unknown Name") {
        crate::db::lookup_player_metadata(db, player_uid)
            .await
            .ok()
            .flatten()
    } else {
        None
    };

    // Determine inspected player's name with proper fallback:
    // 1. Use current player name if valid (non-empty, not "Unknown", not "Unknown Name")
    // 2. Otherwise use cached database metadata name if available
    // 3. As last resort, use "Unknown Name"
    let inspected_name = {
        let is_valid_name = |n: &str| !n.is_empty() && n != "Unknown" && n != "Unknown Name";
        
        if let Some(name) = &player.name {
            if is_valid_name(name) {
                name.clone()
            } else {
                metadata
                    .as_ref()
                    .map(|md| md.name.clone())
                    .filter(|n| is_valid_name(n))
                    .unwrap_or_else(|| String::from("Unknown Name"))
            }
        } else {
            metadata
                .as_ref()
                .map(|md| md.name.clone())
                .filter(|n| is_valid_name(n))
                .unwrap_or_else(|| String::from("Unknown Name"))
        }
    };

    let mut skill_window = SkillsWindow {
        inspected_player: PlayerRow {
            uid: player_uid as f64,
            name: inspected_name,
            class_name: class::get_class_name(player.class.unwrap_or(Class::Unknown)),
            class_spec_name: class::get_class_spec(player.class_spec.unwrap_or(ClassSpec::Unknown)),
            ability_score: player.ability_score.unwrap_or(-1) as f64,
            total_value: player_stats.value as f64,
            value_per_sec: nan_is_zero(player_stats.value as f64 / time_elapsed_secs),
            value_pct: nan_is_zero(player_stats.value as f64 / encounter_stats.value as f64 * 100.0),
            crit_rate: nan_is_zero(player_stats.crit_hits as f64 / player_stats.hits as f64 * 100.0),
            crit_value_rate: nan_is_zero(player_stats.crit_value as f64 / player_stats.value as f64 * 100.0),
            lucky_rate: nan_is_zero(player_stats.lucky_hits as f64 / player_stats.hits as f64 * 100.0),
            lucky_value_rate: nan_is_zero(player_stats.lucky_value as f64 / player_stats.value as f64 * 100.0),
            hits: player_stats.hits as f64,
            hits_per_minute: nan_is_zero(player_stats.hits as f64 / time_elapsed_secs * 60.0),
        },
        local_player_uid: encounter.local_player_uid.unwrap_or(-1) as f64,
        skill_rows: Vec::new(),
        top_value: 0.0,
    };

    // Skills for this player
    for (&skill_uid, skill_stat) in skill_uid_to_stats {
        skill_window.top_value = skill_window.top_value.max(skill_stat.value as f64);
        #[allow(clippy::cast_precision_loss)]
        let skill_row = SkillRow {
            uid: skill_uid as f64,
            name: CombatStats::get_skill_name(skill_uid),
            total_value: skill_stat.value as f64,
            value_per_sec: nan_is_zero(skill_stat.value as f64 / time_elapsed_secs),
            value_pct: nan_is_zero(skill_stat.value as f64 / player_stats.value as f64 * 100.0),
            crit_rate: nan_is_zero(skill_stat.crit_hits as f64 / skill_stat.hits as f64 * 100.0),
            crit_value_rate: nan_is_zero(skill_stat.crit_value as f64 / skill_stat.value as f64 * 100.0),
            lucky_rate: nan_is_zero(skill_stat.lucky_hits as f64 / skill_stat.hits as f64 * 100.0),
            lucky_value_rate: nan_is_zero(skill_stat.lucky_value as f64 / skill_stat.value as f64 * 100.0),
            hits: skill_stat.hits as f64,
            hits_per_minute: nan_is_zero(skill_stat.hits as f64 / time_elapsed_secs * 60.0),
        };
        skill_window.skill_rows.push(skill_row);
    }

    // Sort skills descending by damage dealt
    skill_window.skill_rows.sort_by(|this_row, other_row| {
        other_row
            .total_value
            .partial_cmp(&this_row.total_value) // descending
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(skill_window)
}

#[tauri::command]
#[specta::specta]
#[allow(clippy::cast_precision_loss)]
#[allow(clippy::too_many_lines)]
pub fn get_test_player_window() -> PlayersWindow {
    PlayersWindow {
        player_rows: vec![
            PlayerRow {
                uid: 10_000_001.0,
                name: "Name Stormblade (You)".to_string(),
                class_name: "Stormblade".to_string(),
                class_spec_name: "".to_string(),
                ability_score: 1500.0,
                total_value: 100_000.0,
                value_per_sec: 10_000.6,
                value_pct: 100.0,
                crit_rate: 0.25,
                crit_value_rate: 2.0,
                lucky_rate: 0.10,
                lucky_value_rate: 1.5,
                hits: 200.0,
                hits_per_minute: 3.3,
            },
            PlayerRow {
                uid: 10_000_002.0,
                name: "Name Frost Mage".to_string(),
                class_name: "Frost Mage".to_string(),
                class_spec_name: "".to_string(),
                ability_score: 1500.0,
                total_value: 90_000.0,
                value_per_sec: 6_000.6,
                value_pct: 90.0,
                crit_rate: 0.25,
                crit_value_rate: 2.0,
                lucky_rate: 0.10,
                lucky_value_rate: 1.5,
                hits: 200.0,
                hits_per_minute: 3.3,
            },
            PlayerRow {
                uid: 10_000_003.0,
                name: "Name Wind Knight".to_string(),
                class_name: "Wind Knight".to_string(),
                class_spec_name: "".to_string(),
                ability_score: 1500.0,
                total_value: 80_000.0,
                value_per_sec: 6_000.6,
                value_pct: 80.0,
                crit_rate: 0.25,
                crit_value_rate: 2.0,
                lucky_rate: 0.10,
                lucky_value_rate: 1.5,
                hits: 200.0,
                hits_per_minute: 3.3,
            },
            PlayerRow {
                uid: 10_000_004.0,
                name: "Name Verdant Oracle".to_string(),
                class_name: "Verdant Oracle".to_string(),
                class_spec_name: "".to_string(),
                ability_score: 1500.0,
                total_value: 70_000.0,
                value_per_sec: 6_000.6,
                value_pct: 70.0,
                crit_rate: 0.25,
                crit_value_rate: 2.0,
                lucky_rate: 0.10,
                lucky_value_rate: 1.5,
                hits: 200.0,
                hits_per_minute: 3.3,
            },
            PlayerRow {
                uid: 10_000_005.0,
                name: "Name Heavy Guardian".to_string(),
                class_name: "Heavy Guardian".to_string(),
                class_spec_name: "".to_string(),
                ability_score: 1500.0,
                total_value: 60_000.0,
                value_per_sec: 6_000.6,
                value_pct: 60.0,
                crit_rate: 0.25,
                crit_value_rate: 2.0,
                lucky_rate: 0.10,
                lucky_value_rate: 1.5,
                hits: 200.0,
                hits_per_minute: 3.3,
            },
            PlayerRow {
                uid: 10_000_006.0,
                name: "Name Marksman".to_string(),
                class_name: "Marksman".to_string(),
                class_spec_name: "".to_string(),
                ability_score: 1500.0,
                total_value: 60_000.0,
                value_per_sec: 6_000.6,
                value_pct: 50.0,
                crit_rate: 0.25,
                crit_value_rate: 2.0,
                lucky_rate: 0.10,
                lucky_value_rate: 1.5,
                hits: 200.0,
                hits_per_minute: 3.3,
            },
            PlayerRow {
                uid: 10_000_007.0,
                name: "Name Shield Knight".to_string(),
                class_name: "Shield Knight".to_string(),
                class_spec_name: "".to_string(),
                ability_score: 1500.0,
                total_value: 50_000.0,
                value_per_sec: 6_000.6,
                value_pct: 40.0,
                crit_rate: 0.25,
                crit_value_rate: 2.0,
                lucky_rate: 0.10,
                lucky_value_rate: 1.5,
                hits: 200.0,
                hits_per_minute: 3.3,
            },
            PlayerRow {
                uid: 10_000_008.0,
                name: "Name Beat Performer".to_string(),
                class_name: "Beat Performer".to_string(),
                class_spec_name: "".to_string(),
                ability_score: 1500.0,
                total_value: 10_000.0,
                value_per_sec: 6_000.6,
                value_pct: 30.0,
                crit_rate: 0.25,
                crit_value_rate: 2.0,
                lucky_rate: 0.10,
                lucky_value_rate: 1.5,
                hits: 200.0,
                hits_per_minute: 3.3,
            },
            PlayerRow {
                uid: 10_000_009.0,
                name: "Blank Class".to_string(),
                class_name: "blank".to_string(),
                class_spec_name: "".to_string(),
                ability_score: 1500.0,
                total_value: 10_000.0,
                value_per_sec: 6_000.6,
                value_pct: 20.0,
                crit_rate: 0.25,
                crit_value_rate: 2.0,
                lucky_rate: 0.10,
                lucky_value_rate: 1.5,
                hits: 200.0,
                hits_per_minute: 3.3,
            },
        ],
        local_player_uid: 10_000_001.0,
        top_value: 100_000.0,
    }
}

#[tauri::command]
#[specta::specta]
#[allow(clippy::too_many_lines)]
pub fn get_test_skill_window(_player_uid: String) -> Result<SkillsWindow, String> {
    Ok(SkillsWindow {
        inspected_player: PlayerRow {
            uid: 10_000_001.0,
            name: "Name Stormblade".to_string(),
            class_name: "Stormblade".to_string(),
            class_spec_name: "Iaido".to_string(),
            ability_score: 1500.0,
            total_value: 100_000.0,
            value_per_sec: 10_000.6,
            value_pct: 90.0,
            crit_rate: 0.25,
            crit_value_rate: 2.0,
            lucky_rate: 0.10,
            lucky_value_rate: 1.5,
            hits: 200.0,
            hits_per_minute: 3.3,
        },
        skill_rows: vec![
            SkillRow {
                uid: 3602.0,
                name: "Skill 1".to_string(),
                total_value: 100_000.0,
                value_per_sec: 5_000.0,
                value_pct: 80.0,
                crit_rate: 0.30,
                crit_value_rate: 2.1,
                lucky_rate: 0.12,
                lucky_value_rate: 1.4,
                hits: 80.0,
                hits_per_minute: 1.5,
            },
            SkillRow {
                uid: 3602.0,
                name: "Skill 2".to_string(),
                total_value: 50_000.0,
                value_per_sec: 7_345.6,
                value_pct: 70.0,
                crit_rate: 0.20,
                crit_value_rate: 1.9,
                lucky_rate: 0.08,
                lucky_value_rate: 1.3,
                hits: 120.0,
                hits_per_minute: 1.8,
            },
            SkillRow {
                uid: 3602.0,
                name: "Skill 3".to_string(),
                total_value: 33_000.0,
                value_per_sec: 7_345.6,
                value_pct: 60.0,
                crit_rate: 0.20,
                crit_value_rate: 1.9,
                lucky_rate: 0.08,
                lucky_value_rate: 1.3,
                hits: 120.0,
                hits_per_minute: 1.8,
            },
            SkillRow {
                uid: 3602.0,
                name: "Skill 4".to_string(),
                total_value: 23_000.0,
                value_per_sec: 7_345.6,
                value_pct: 50.0,
                crit_rate: 0.20,
                crit_value_rate: 1.9,
                lucky_rate: 0.08,
                lucky_value_rate: 1.3,
                hits: 120.0,
                hits_per_minute: 1.8,
            },
            SkillRow {
                uid: 3602.0,
                name: "Skill 5".to_string(),
                total_value: 11_000.0,
                value_per_sec: 7_345.6,
                value_pct: 40.0,
                crit_rate: 0.20,
                crit_value_rate: 1.9,
                lucky_rate: 0.08,
                lucky_value_rate: 1.3,
                hits: 120.0,
                hits_per_minute: 1.8,
            },
            SkillRow {
                uid: 3602.0,
                name: "Skill 6".to_string(),
                total_value: 1_000.0,
                value_per_sec: 7_345.6,
                value_pct: 30.0,
                crit_rate: 0.20,
                crit_value_rate: 1.9,
                lucky_rate: 0.08,
                lucky_value_rate: 1.3,
                hits: 120.0,
                hits_per_minute: 1.8,
            },
            SkillRow {
                uid: 3602.0,
                name: "Skill 7".to_string(),
                total_value: 400.0,
                value_per_sec: 7_345.6,
                value_pct: 20.0,
                crit_rate: 0.20,
                crit_value_rate: 1.9,
                lucky_rate: 0.08,
                lucky_value_rate: 1.3,
                hits: 120.0,
                hits_per_minute: 1.8,
            },
        ],
        local_player_uid: 10_000_001.0,
        top_value: 100_000.0,
    })
}
