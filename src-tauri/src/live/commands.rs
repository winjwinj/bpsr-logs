use crate::live::commands_models::{HeaderInfo, PlayerRow, PlayersWindow, SkillRow, SkillsWindow};
use crate::live::crowdsource_persistence::{save_snapshot, CrowdsourceMonsterSnapshot};
use crate::live::opcodes_models::class::{Class, ClassSpec};
use crate::live::opcodes_models::{
    class, get_crowdsource_monster_choices, resolve_crowdsource_remote, CombatStats, Encounter,
    EncounterMutex,
};
use crate::live::bptimer_stream::{
    MobHpData, MobHpStoreMutex, BpTimerStreamControlSender, BPTIMER_BASE_URL,
    CREATE_HP_REPORT_ENDPOINT, CROWD_SOURCE_API_KEY,
};
use crate::packets::packet_capture::request_restart;
use crate::WINDOW_LIVE_LABEL;
use blueprotobuf_lib::blueprotobuf::EEntityType;
use log::{info, warn};
use reqwest::Client;
use serde::Serialize;
use specta::Type;
use std::sync::MutexGuard;
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

#[derive(Debug, Clone, Serialize, Type)]
pub struct CrowdsourcedMonster {
    pub name: String,
    pub id: i32,
    pub remote_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct CrowdsourcedMonsterOption {
    pub name: String,
    pub id: i32,
    pub remote_id: String,
}

#[tauri::command]
#[specta::specta]
pub fn get_last_hit_boss_name(state: tauri::State<'_, EncounterMutex>) -> Option<String> {
    let encounter = state.lock().unwrap();
    let result = encounter.crowdsource_monster_name.clone();
    result
}

#[tauri::command]
#[specta::specta]
pub fn get_crowdsourced_monster(state: tauri::State<'_, EncounterMutex>) -> Option<CrowdsourcedMonster> {
    let encounter = state.lock().unwrap();
    match (
        &encounter.crowdsource_monster_name,
        encounter.crowdsource_monster_id,
        encounter.crowdsource_monster_remote_id.as_ref(),
    ) {
        (Some(name), Some(id), remote_id) => Some(CrowdsourcedMonster {
            name: name.clone(),
            id,
             remote_id: remote_id.cloned(),
        }),
        _ => None,
    }
}

#[tauri::command]
#[specta::specta]
pub fn get_crowdsourced_monster_options() -> Vec<CrowdsourcedMonsterOption> {
    get_crowdsource_monster_choices()
        .into_iter()
        .map(|(id, name, remote_id)| CrowdsourcedMonsterOption {
            name,
            id,
            remote_id,
        })
        .collect()
}

#[tauri::command]
#[specta::specta]
pub fn get_crowdsourced_mob_hp(
    encounter_state: tauri::State<'_, EncounterMutex>,
    store_state: tauri::State<'_, MobHpStoreMutex>,
) -> Result<Vec<MobHpData>, String> {
    let remote_id = encounter_state
        .lock()
        .map_err(|_| "Failed to lock encounter".to_string())?
        .crowdsource_monster_remote_id
        .clone()
        .ok_or_else(|| "No crowdsourced monster remote id".to_string())?;


    Ok(store_state.read().get_by_remote_id(&remote_id))
}

#[tauri::command]
#[specta::specta]
pub fn set_crowdsourced_monster_remote(
    app: tauri::AppHandle,
    encounter_state: tauri::State<'_, EncounterMutex>,
    remote_id: String,
) -> Result<(), String> {
    let (monster_id, monster_name) =
        resolve_crowdsource_remote(&remote_id).ok_or_else(|| format!("Unknown remote id: {remote_id}"))?;

    {
        let mut encounter = encounter_state
            .lock()
            .map_err(|_| "Failed to lock encounter".to_string())?;
        encounter.crowdsource_monster_id = Some(monster_id);
        encounter.crowdsource_monster_name = Some(monster_name.clone());
        encounter.crowdsource_monster_remote_id = Some(remote_id.clone());
    }

    let snapshot = CrowdsourceMonsterSnapshot {
        monster_id,
        monster_name,
        remote_id: remote_id.clone(),
    };

    if let Err(err) = save_snapshot(&app, &snapshot) {
        warn!(
            "commands::set_crowdsourced_monster_remote - failed to persist snapshot for remote_id={remote_id}: {err}"
        );
    }

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn get_local_player_line(state: tauri::State<'_, EncounterMutex>) -> Result<Option<i32>, String> {
    let encounter = state
        .lock()
        .map_err(|_| "Failed to lock encounter".to_string())?;

    let line = encounter
        .local_player
        .as_ref()
        .and_then(|player| player.v_data.as_ref())
        .and_then(|v| v.scene_data.as_ref())
        .and_then(|scene| scene.line_id);

    Ok(line.map(|line| line as i32))
}

#[tauri::command]
#[specta::specta]
pub fn set_bptimer_stream_active(
    control_state: tauri::State<'_, BpTimerStreamControlSender>,
    active: bool,
) -> Result<(), String> {
    control_state
        .send(active)
        .map_err(|err| format!("Failed to update BPTimer stream state: {err}"))
}

#[tauri::command]
#[specta::specta]
pub async fn mark_current_crowdsourced_line_dead(
    encounter_state: tauri::State<'_, EncounterMutex>,
    store_state: tauri::State<'_, MobHpStoreMutex>,
) -> Result<(), String> {
    let (monster_id, monster_name, remote_id, line, pos_x, pos_y) = {
        let encounter = encounter_state
            .lock()
            .map_err(|_| "Failed to lock encounter".to_string())?;

        let monster_id = encounter
            .crowdsource_monster_id
            .ok_or_else(|| "No crowdsourced monster ID available".to_string())?;

        let monster_name = encounter
            .crowdsource_monster_name
            .clone()
            .unwrap_or_else(|| "Unknown Monster".to_string());

        let remote_id = encounter
            .crowdsource_monster_remote_id
            .clone()
            .ok_or_else(|| "No crowdsourced monster remote ID available".to_string())?;

        let scene_data = encounter
            .local_player
            .as_ref()
            .and_then(|player| player.v_data.as_ref())
            .and_then(|v| v.scene_data.as_ref())
            .ok_or_else(|| "No local player scene data available".to_string())?;

        let line = scene_data
            .line_id
            .ok_or_else(|| "No line id available for local player".to_string())?;

        let pos = scene_data
            .pos
            .as_ref()
            .ok_or_else(|| "No position data available for local player".to_string())?;

        let pos_x = pos
            .x
            .ok_or_else(|| "No pos_x available for local player".to_string())? as f64;

        let pos_y = pos
            .y
            .ok_or_else(|| "No pos_y available for local player".to_string())? as f64;

        (monster_id, monster_name, remote_id, line, pos_x, pos_y)
    };

    info!(
        "mark_current_crowdsourced_line_dead - reporting monster '{}' ({}) as dead on line {}",
        monster_name, monster_id, line
    );

    let body = serde_json::json!({
        "monster_id": monster_id,
        "hp_pct": 0,
        "line": line,
        "pos_x": pos_x,
        "pos_y": pos_y,
    });

    let client = Client::new();
    let response = client
        .post(format!("{BPTIMER_BASE_URL}{CREATE_HP_REPORT_ENDPOINT}"))
        .header("X-API-Key", CROWD_SOURCE_API_KEY)
        .json(&body)
        .send()
        .await
        .map_err(|err| format!("Failed to send HP report: {err}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "HP report failed with status {}",
            response.status()
        ));
    }

    {
        let mut store = store_state.write();
        store.seed_remote_hp(&remote_id, 0, Some(0));
    }

    Ok(())
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
pub fn reset_encounter(state: tauri::State<'_, EncounterMutex>) {
    let mut encounter = state.lock().unwrap();
    encounter.clone_from(&Encounter::default());
    info!("encounter reset");
}

#[tauri::command]
#[specta::specta]
pub fn toggle_pause_encounter(state: tauri::State<'_, EncounterMutex>) {
    let mut encounter = state.lock().unwrap();
    encounter.is_encounter_paused = !encounter.is_encounter_paused;
}

#[derive(Debug, Clone, Copy)]
pub enum StatType {
    Dmg,
    DmgBossOnly,
    Heal,
}

#[tauri::command]
#[specta::specta]
pub fn get_dps_player_window(state: tauri::State<'_, EncounterMutex>) -> PlayersWindow {
    let encounter = state.lock().unwrap();
    get_player_window(encounter, StatType::Dmg)
}

#[tauri::command]
#[specta::specta]
pub fn get_heal_player_window(state: tauri::State<'_, EncounterMutex>) -> PlayersWindow {
    let encounter = state.lock().unwrap();
    get_player_window(encounter, StatType::Heal)
}

#[tauri::command]
#[specta::specta]
pub fn get_dps_boss_only_player_window(state: tauri::State<'_, EncounterMutex>) -> PlayersWindow {
    let encounter = state.lock().unwrap();
    get_player_window(encounter, StatType::DmgBossOnly)
}

pub fn get_player_window(encounter: MutexGuard<Encounter>, stat_type: StatType) -> PlayersWindow {
    let time_elapsed_ms = encounter.time_last_combat_packet_ms - encounter.time_fight_start_ms;
    #[allow(clippy::cast_precision_loss)]
    let time_elapsed_secs = time_elapsed_ms as f64 / 1000.0;

    #[allow(clippy::cast_precision_loss)]
    let mut player_window = PlayersWindow {
        player_rows: Vec::new(),
        local_player_uid: encounter.local_player_uid.unwrap_or(-1) as f64,
        top_value: 0.0,
    };
    for (&entity_uid, entity) in &encounter.entity_uid_to_entity {
        // Select stats per player and encounter
        let (entity_stats, encounter_stats) = match stat_type {
            StatType::Dmg => (&entity.dmg_stats, &encounter.dmg_stats),
            StatType::DmgBossOnly => (&entity.dmg_stats_boss_only, &encounter.dmg_stats_boss_only),
            StatType::Heal => (&entity.heal_stats, &encounter.heal_stats),
        };
        let is_player = entity.entity_type == EEntityType::EntChar;
        let did_damage = entity_stats.value > 0;
        if !is_player || !did_damage {
            continue;
        }
        player_window.top_value = player_window.top_value.max(entity_stats.value as f64);
        #[allow(clippy::cast_precision_loss)]
        let damage_row = PlayerRow {
            uid: entity_uid as f64,
            name: entity.name.clone().unwrap_or(String::from("Unknown Name")),
            class_name: class::get_class_name(entity.class.unwrap_or(Class::Unknown)),
            class_spec_name: class::get_class_spec(entity.class_spec.unwrap_or(ClassSpec::Unknown)),
            ability_score: entity.ability_score.unwrap_or(-1) as f64,
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
    drop(encounter); // drop lock before expensive sort

    // Sort skills descending by damage dealt
    player_window.player_rows.sort_by(|this_row, other_row| {
        other_row.total_value
                 .partial_cmp(&this_row.total_value)
                 .unwrap_or(std::cmp::Ordering::Equal)
    });

    player_window
}

#[tauri::command]
#[specta::specta]
pub fn get_dps_skill_window(state: tauri::State<'_, EncounterMutex>, player_uid_str: &str) -> Result<SkillsWindow, String> {
    let player_uid = player_uid_str.parse().unwrap();
    get_skill_window(state, player_uid, StatType::Dmg)
}

#[tauri::command]
#[specta::specta]
pub fn get_dps_boss_only_skill_window(state: tauri::State<'_, EncounterMutex>, player_uid_str: &str) -> Result<SkillsWindow, String> {
    let player_uid = player_uid_str.parse().unwrap();
    get_skill_window(state, player_uid, StatType::DmgBossOnly)
}

#[tauri::command]
#[specta::specta]
pub fn get_heal_skill_window(state: tauri::State<'_, EncounterMutex>, player_uid_str: &str) -> Result<SkillsWindow, String> {
    let player_uid = player_uid_str.parse().unwrap();
    get_skill_window(state, player_uid, StatType::Heal)
}

pub fn get_skill_window(state: tauri::State<'_, EncounterMutex>, player_uid: i64, stat_type: StatType) -> Result<SkillsWindow, String> {
    let encounter = state.lock().unwrap();

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

    // Player DPS Stats
    #[allow(clippy::cast_precision_loss)]
    let mut skill_window = SkillsWindow {
        inspected_player: PlayerRow {
            uid: player_uid as f64,
            name: player.name.clone().unwrap_or(String::from("Unknown Name")),
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
    drop(encounter);  // drop before expensive sort

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
