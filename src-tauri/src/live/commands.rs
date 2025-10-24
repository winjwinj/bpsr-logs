use crate::WINDOW_LIVE_LABEL;
use crate::live::commands_models::{
    HeaderInfo, PlayerRow, PlayerRows, PlayersWindow, SkillRow, SkillsWindow,
};
use crate::live::opcodes_models::{Encounter, EncounterMutex, Skill, class};
use crate::packets::packet_capture::request_restart;
use blueprotobuf_lib::blueprotobuf::EEntityType;
use log::info;
use tauri::Manager;
use tauri_plugin_clipboard_manager::ClipboardExt;
use window_vibrancy::{apply_blur, clear_blur};

fn prettify_name(player_uid: i64, local_player_uid: i64, player_name: &String) -> String {
    if player_uid == local_player_uid && player_name.is_empty() {
        String::from("You")
    } else if player_uid == local_player_uid && !player_name.is_empty() {
        format!("{player_name} (You)")
    } else {
        player_name.clone()
    }
}

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
    let json = serde_json::to_string_pretty(&encounter.local_player).unwrap();
    app.clipboard().write_text(json).unwrap();
}

#[tauri::command]
#[specta::specta]
pub fn get_header_info(state: tauri::State<'_, EncounterMutex>) -> Result<HeaderInfo, String> {
    let encounter = state.lock().unwrap();

    if encounter.total_dmg == 0 {
        return Err("No damage found".to_string());
    }

    let time_elapsed_ms = encounter
        .time_last_combat_packet_ms
        .saturating_sub(encounter.time_fight_start_ms);
    #[allow(clippy::cast_precision_loss)]
    let time_elapsed_secs = time_elapsed_ms as f64 / 1000.0;

    #[allow(clippy::cast_precision_loss)]
    Ok(HeaderInfo {
        total_dps: nan_is_zero(encounter.total_dmg as f64 / time_elapsed_secs),
        total_dmg: encounter.total_dmg as f64,
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

	encounter.reset_stats();
    // encounter.clone_from(&Encounter::default());

    info!("encounter reset");
}

#[tauri::command]
#[specta::specta]
pub fn toggle_pause_encounter(state: tauri::State<'_, EncounterMutex>) {
    let mut encounter = state.lock().unwrap();
    encounter.is_encounter_paused = !encounter.is_encounter_paused;
}

#[tauri::command]
#[specta::specta]
pub fn get_dps_player_window(
    state: tauri::State<'_, EncounterMutex>,
) -> Result<PlayersWindow, String> {
    let encounter = state.lock().unwrap();

    let time_elapsed_ms = encounter
        .time_last_combat_packet_ms
        .saturating_sub(encounter.time_fight_start_ms);

    let mut dps_window = PlayersWindow {
        player_rows: PlayerRows::default(),
        // ..Default::default()
    };

    #[allow(clippy::cast_precision_loss)]
    let time_elapsed_secs = time_elapsed_ms as f64 / 1000.0;

    if encounter.total_dmg == 0 {
        return Err("No damage found".to_string());
    }

    for (&entity_uid, entity) in &encounter.entity_uid_to_entity {
        // calculate things like dps
        let is_player = entity.entity_type == EEntityType::EntChar;
        let did_damage = !entity.skill_uid_to_dmg_skill.is_empty();
        // info!("{}, {is_player}", entity.name);
        if is_player && did_damage {
            // Damage Stats per player
            #[allow(clippy::cast_precision_loss)]
            let damage_row = PlayerRow {
                uid: entity_uid as f64,
                name: prettify_name(entity_uid, encounter.local_player_uid, &entity.name),
                class_name: class::get_class_name(entity.class_id),
                class_spec_name: class::get_class_spec(entity.class_spec),
                ability_score: entity.ability_score as f64,
                total_dmg: entity.total_dmg as f64,
                dps: nan_is_zero(entity.total_dmg as f64 / time_elapsed_secs),
                dmg_pct: nan_is_zero(entity.total_dmg as f64 / encounter.total_dmg as f64 * 100.0),
                crit_rate: nan_is_zero(
                    entity.crit_hits_dmg as f64 / entity.hits_dmg as f64 * 100.0,
                ),
                crit_dmg_rate: nan_is_zero(
                    entity.crit_total_dmg as f64 / entity.total_dmg as f64 * 100.0,
                ),
                lucky_rate: nan_is_zero(
                    entity.lucky_hits_dmg as f64 / entity.hits_dmg as f64 * 100.0,
                ),
                lucky_dmg_rate: nan_is_zero(
                    entity.lucky_total_dmg as f64 / entity.total_dmg as f64 * 100.0,
                ),
                hits: entity.hits_dmg as f64,
                hits_per_minute: nan_is_zero(entity.hits_dmg as f64 / time_elapsed_secs * 60.0),
                // ..Default:default()
            };
            dps_window.player_rows.push(damage_row);
        }
    }
    drop(encounter); // todo: is this a good idea? dropping lock before expensive sort

    // Sort skills descending by damage dealt
    dps_window.player_rows.sort_by(|this_row, other_row| {
        other_row
            .total_dmg
            .partial_cmp(&this_row.total_dmg)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(dps_window)
}

#[tauri::command]
#[specta::specta]
pub fn get_dps_skill_window(
    state: tauri::State<'_, EncounterMutex>,
    player_uid_str: &str,
) -> Result<SkillsWindow, String> {
    let player_uid: i64 = player_uid_str.parse().unwrap();
    let encounter = state.lock().unwrap();

    let entity = encounter
        .entity_uid_to_entity
        .get(&player_uid)
        .ok_or_else(|| format!("Entity not found for player_uid {player_uid}"))?;

    let time_elapsed_ms = encounter
        .time_last_combat_packet_ms
        .saturating_sub(encounter.time_fight_start_ms);
    #[allow(clippy::cast_precision_loss)]
    let time_elapsed_secs = time_elapsed_ms as f64 / 1000.0;

    // Player DPS Stats
    #[allow(clippy::cast_precision_loss)]
    let mut skill_window = SkillsWindow {
        curr_player: vec![PlayerRow {
            uid: player_uid as f64,
            name: prettify_name(player_uid, encounter.local_player_uid, &entity.name),
            class_name: class::get_class_name(entity.class_id),
            class_spec_name: class::get_class_spec(entity.class_spec),
            ability_score: entity.ability_score as f64,
            total_dmg: entity.total_dmg as f64,
            dps: nan_is_zero(entity.total_dmg as f64 / time_elapsed_secs),
            dmg_pct: nan_is_zero(entity.total_dmg as f64 / encounter.total_dmg as f64 * 100.0),
            crit_rate: nan_is_zero(entity.crit_hits_dmg as f64 / entity.hits_dmg as f64 * 100.0),
            crit_dmg_rate: nan_is_zero(
                entity.crit_total_dmg as f64 / entity.total_dmg as f64 * 100.0,
            ),
            lucky_rate: nan_is_zero(entity.lucky_hits_dmg as f64 / entity.hits_dmg as f64 * 100.0),
            lucky_dmg_rate: nan_is_zero(
                entity.lucky_total_dmg as f64 / entity.total_dmg as f64 * 100.0,
            ),
            hits: entity.hits_dmg as f64,
            hits_per_minute: nan_is_zero(entity.hits_dmg as f64 / time_elapsed_secs * 60.0),
            // ..Default::default()
        }],
        ..Default::default()
    };

    // Skills for this player
    for (&skill_uid, skill) in &entity.skill_uid_to_dmg_skill {
        // info!("name: {}, {}", Skill::get_skill_name(skill_uid), skill.crit_hits as f64 / skill.hits as f64 * 100.0);
        #[allow(clippy::cast_precision_loss)]
        let skill_row = SkillRow {
            uid: skill_uid as f64,
            name: Skill::get_skill_name(skill_uid),
            total_dmg: skill.total_value as f64,
            dps: nan_is_zero(skill.total_value as f64 / time_elapsed_secs),
            dmg_pct: nan_is_zero(skill.total_value as f64 / entity.total_dmg as f64 * 100.0),
            crit_rate: nan_is_zero(skill.crit_hits as f64 / skill.hits as f64 * 100.0),
            crit_dmg_rate: nan_is_zero(
                skill.crit_total_value as f64 / skill.total_value as f64 * 100.0,
            ),
            lucky_rate: nan_is_zero(skill.lucky_hits as f64 / skill.hits as f64 * 100.0),
            lucky_dmg_rate: nan_is_zero(
                skill.lucky_total_value as f64 / skill.total_value as f64 * 100.0,
            ),
            hits: skill.hits as f64,
            hits_per_minute: nan_is_zero(skill.hits as f64 / time_elapsed_secs * 60.0),
            // ..Default::default()
        };
        skill_window.skill_rows.push(skill_row);
    }

    drop(encounter);

    // Sort skills descending by damage dealt
    skill_window.skill_rows.sort_by(|this_row, other_row| {
        other_row
            .total_dmg
            .partial_cmp(&this_row.total_dmg) // descending
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(skill_window)
}

#[tauri::command]
#[specta::specta]
pub fn get_heal_player_window(
    state: tauri::State<'_, EncounterMutex>,
) -> Result<PlayersWindow, String> {
    let encounter = state.lock().unwrap();

    let time_elapsed_ms = encounter
        .time_last_combat_packet_ms
        .saturating_sub(encounter.time_fight_start_ms);

    let mut dps_window = PlayersWindow {
        player_rows: PlayerRows::default(),
        // ..Default::default()
    };

    if encounter.total_heal == 0 {
        return Err("No healing found ".to_string());
    }

    #[allow(clippy::cast_precision_loss)]
    let time_elapsed_secs = time_elapsed_ms as f64 / 1000.0;

    for (&entity_uid, entity) in &encounter.entity_uid_to_entity {
        // calculate things like dps
        let is_player = entity.entity_type == EEntityType::EntChar;
        let did_damage = !entity.skill_uid_to_heal_skill.is_empty();
        // info!("{}, {is_player}", entity.name);
        if is_player && did_damage {
            // Damage Stats per player
            #[allow(clippy::cast_precision_loss)]
            let damage_row = PlayerRow {
                uid: entity_uid as f64,
                name: prettify_name(entity_uid, encounter.local_player_uid, &entity.name),
                class_name: class::get_class_name(entity.class_id),
                class_spec_name: class::get_class_spec(entity.class_spec),
                ability_score: entity.ability_score as f64,
                total_dmg: entity.total_heal as f64,
                dps: nan_is_zero(entity.total_heal as f64 / time_elapsed_secs),
                dmg_pct: nan_is_zero(
                    entity.total_heal as f64 / encounter.total_heal as f64 * 100.0,
                ),
                crit_rate: nan_is_zero(
                    entity.crit_hits_heal as f64 / entity.hits_heal as f64 * 100.0,
                ),
                crit_dmg_rate: nan_is_zero(
                    entity.crit_total_heal as f64 / entity.total_heal as f64 * 100.0,
                ),
                lucky_rate: nan_is_zero(
                    entity.lucky_hits_heal as f64 / entity.hits_heal as f64 * 100.0,
                ),
                lucky_dmg_rate: nan_is_zero(
                    entity.lucky_total_heal as f64 / entity.total_heal as f64 * 100.0,
                ),
                hits: entity.hits_heal as f64,
                hits_per_minute: nan_is_zero(entity.hits_heal as f64 / time_elapsed_secs * 60.0),
                // ..Default:default()
            };
            dps_window.player_rows.push(damage_row);
        }
    }
    drop(encounter); // todo: is this a good idea? dropping lock before expensive sort

    // Sort skills descending by damage dealt
    dps_window.player_rows.sort_by(|this_row, other_row| {
        other_row
            .total_dmg
            .partial_cmp(&this_row.total_dmg)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(dps_window)
}

#[tauri::command]
#[specta::specta]
pub fn get_heal_skill_window(
    state: tauri::State<'_, EncounterMutex>,
    player_uid_str: &str,
) -> Result<SkillsWindow, String> {
    let player_uid: i64 = player_uid_str.parse().unwrap();
    let encounter = state.lock().unwrap();

    let entity = encounter
        .entity_uid_to_entity
        .get(&player_uid)
        .ok_or_else(|| format!("Entity not found for player_uid {player_uid}"))?;

    let time_elapsed_ms = encounter
        .time_last_combat_packet_ms
        .saturating_sub(encounter.time_fight_start_ms);
    #[allow(clippy::cast_precision_loss)]
    let time_elapsed_secs = time_elapsed_ms as f64 / 1000.0;

    // Player DPS Stats
    #[allow(clippy::cast_precision_loss)]
    let mut skill_window = SkillsWindow {
        curr_player: vec![PlayerRow {
            uid: player_uid as f64,
            name: prettify_name(player_uid, encounter.local_player_uid, &entity.name),
            class_name: class::get_class_name(entity.class_id),
            class_spec_name: class::get_class_spec(entity.class_spec),
            ability_score: entity.ability_score as f64,
            total_dmg: entity.total_heal as f64,
            dps: nan_is_zero(entity.total_heal as f64 / time_elapsed_secs),
            dmg_pct: nan_is_zero(entity.total_heal as f64 / encounter.total_heal as f64 * 100.0),
            crit_rate: nan_is_zero(entity.crit_hits_heal as f64 / entity.hits_heal as f64 * 100.0),
            crit_dmg_rate: nan_is_zero(
                entity.crit_total_heal as f64 / entity.total_heal as f64 * 100.0,
            ),
            lucky_rate: nan_is_zero(
                entity.lucky_hits_heal as f64 / entity.hits_heal as f64 * 100.0,
            ),
            lucky_dmg_rate: nan_is_zero(
                entity.lucky_total_heal as f64 / entity.total_heal as f64 * 100.0,
            ),
            hits: entity.hits_heal as f64,
            hits_per_minute: nan_is_zero(entity.hits_heal as f64 / time_elapsed_secs * 60.0),
            // ..Default::default()
        }],
        ..Default::default()
    };

    // Skills for this player
    for (&skill_uid, skill) in &entity.skill_uid_to_heal_skill {
        // info!("name: {}, {}", Skill::get_skill_name(skill_uid), skill.crit_hits as f64 / skill.hits as f64 * 100.0);
        #[allow(clippy::cast_precision_loss)]
        let skill_row = SkillRow {
            uid: skill_uid as f64,
            name: Skill::get_skill_name(skill_uid),
            total_dmg: skill.total_value as f64,
            dps: nan_is_zero(skill.total_value as f64 / time_elapsed_secs),
            dmg_pct: nan_is_zero(skill.total_value as f64 / entity.total_heal as f64 * 100.0),
            crit_rate: nan_is_zero(skill.crit_hits as f64 / skill.hits as f64 * 100.0),
            crit_dmg_rate: nan_is_zero(
                skill.crit_total_value as f64 / skill.total_value as f64 * 100.0,
            ),
            lucky_rate: nan_is_zero(skill.lucky_hits as f64 / skill.hits as f64 * 100.0),
            lucky_dmg_rate: nan_is_zero(
                skill.lucky_total_value as f64 / skill.total_value as f64 * 100.0,
            ),
            hits: skill.hits as f64,
            hits_per_minute: nan_is_zero(skill.hits as f64 / time_elapsed_secs * 60.0),
            // ..Default::default()
        };
        skill_window.skill_rows.push(skill_row);
    }

    drop(encounter);

    // Sort skills descending by damage dealt
    skill_window.skill_rows.sort_by(|this_row, other_row| {
        other_row
            .total_dmg
            .partial_cmp(&this_row.total_dmg) // descending
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(skill_window)
}

#[tauri::command]
#[specta::specta]
#[allow(clippy::cast_precision_loss)]
#[allow(clippy::too_many_lines)]
pub fn get_test_player_window() -> Result<PlayersWindow, String> {
    Ok(PlayersWindow {
        player_rows: vec![
            PlayerRow {
                uid: 10_000_001.0,
                name: "Name Stormblade (You)".to_string(),
                class_name: "Stormblade".to_string(),
                class_spec_name: "".to_string(),
                ability_score: 1500.0,
                total_dmg: 100_000.0,
                dps: 10_000.6,
                dmg_pct: 100.0,
                crit_rate: 0.25,
                crit_dmg_rate: 2.0,
                lucky_rate: 0.10,
                lucky_dmg_rate: 1.5,
                hits: 200.0,
                hits_per_minute: 3.3,
            },
            PlayerRow {
                uid: 10_000_002.0,
                name: "Name Frost Mage".to_string(),
                class_name: "Frost Mage".to_string(),
                class_spec_name: "".to_string(),
                ability_score: 1500.0,
                total_dmg: 90_000.0,
                dps: 6_000.6,
                dmg_pct: 90.0,
                crit_rate: 0.25,
                crit_dmg_rate: 2.0,
                lucky_rate: 0.10,
                lucky_dmg_rate: 1.5,
                hits: 200.0,
                hits_per_minute: 3.3,
            },
            PlayerRow {
                uid: 10_000_003.0,
                name: "Name Wind Knight".to_string(),
                class_name: "Wind Knight".to_string(),
                class_spec_name: "".to_string(),
                ability_score: 1500.0,
                total_dmg: 80_000.0,
                dps: 6_000.6,
                dmg_pct: 80.0,
                crit_rate: 0.25,
                crit_dmg_rate: 2.0,
                lucky_rate: 0.10,
                lucky_dmg_rate: 1.5,
                hits: 200.0,
                hits_per_minute: 3.3,
            },
            PlayerRow {
                uid: 10_000_004.0,
                name: "Name Verdant Oracle".to_string(),
                class_name: "Verdant Oracle".to_string(),
                class_spec_name: "".to_string(),
                ability_score: 1500.0,
                total_dmg: 70_000.0,
                dps: 6_000.6,
                dmg_pct: 70.0,
                crit_rate: 0.25,
                crit_dmg_rate: 2.0,
                lucky_rate: 0.10,
                lucky_dmg_rate: 1.5,
                hits: 200.0,
                hits_per_minute: 3.3,
            },
            PlayerRow {
                uid: 10_000_005.0,
                name: "Name Heavy Guardian".to_string(),
                class_name: "Heavy Guardian".to_string(),
                class_spec_name: "".to_string(),
                ability_score: 1500.0,
                total_dmg: 60_000.0,
                dps: 6_000.6,
                dmg_pct: 60.0,
                crit_rate: 0.25,
                crit_dmg_rate: 2.0,
                lucky_rate: 0.10,
                lucky_dmg_rate: 1.5,
                hits: 200.0,
                hits_per_minute: 3.3,
            },
            PlayerRow {
                uid: 10_000_006.0,
                name: "Name Marksman".to_string(),
                class_name: "Marksman".to_string(),
                class_spec_name: "".to_string(),
                ability_score: 1500.0,
                total_dmg: 60_000.0,
                dps: 6_000.6,
                dmg_pct: 50.0,
                crit_rate: 0.25,
                crit_dmg_rate: 2.0,
                lucky_rate: 0.10,
                lucky_dmg_rate: 1.5,
                hits: 200.0,
                hits_per_minute: 3.3,
            },
            PlayerRow {
                uid: 10_000_007.0,
                name: "Name Shield Knight".to_string(),
                class_name: "Shield Knight".to_string(),
                class_spec_name: "".to_string(),
                ability_score: 1500.0,
                total_dmg: 50_000.0,
                dps: 6_000.6,
                dmg_pct: 40.0,
                crit_rate: 0.25,
                crit_dmg_rate: 2.0,
                lucky_rate: 0.10,
                lucky_dmg_rate: 1.5,
                hits: 200.0,
                hits_per_minute: 3.3,
            },
            PlayerRow {
                uid: 10_000_008.0,
                name: "Name Beat Performer".to_string(),
                class_name: "Beat Performer".to_string(),
                class_spec_name: "".to_string(),
                ability_score: 1500.0,
                total_dmg: 10_000.0,
                dps: 6_000.6,
                dmg_pct: 30.0,
                crit_rate: 0.25,
                crit_dmg_rate: 2.0,
                lucky_rate: 0.10,
                lucky_dmg_rate: 1.5,
                hits: 200.0,
                hits_per_minute: 3.3,
            },
            PlayerRow {
                uid: 10_000_009.0,
                name: "Blank Class".to_string(),
                class_name: "blank".to_string(),
                class_spec_name: "".to_string(),
                ability_score: 1500.0,
                total_dmg: 10_000.0,
                dps: 6_000.6,
                dmg_pct: 20.0,
                crit_rate: 0.25,
                crit_dmg_rate: 2.0,
                lucky_rate: 0.10,
                lucky_dmg_rate: 1.5,
                hits: 200.0,
                hits_per_minute: 3.3,
            },
        ],
    })
}

#[tauri::command]
#[specta::specta]
#[allow(clippy::too_many_lines)]
pub fn get_test_skill_window(_player_uid: String) -> Result<SkillsWindow, String> {
    Ok(SkillsWindow {
        curr_player: vec![PlayerRow {
            uid: 10_000_001.0,
            name: "Name Stormblade".to_string(),
            class_name: "Stormblade".to_string(),
            class_spec_name: "Iaido".to_string(),
            ability_score: 1500.0,
            total_dmg: 100_000.0,
            dps: 10_000.6,
            dmg_pct: 90.0,
            crit_rate: 0.25,
            crit_dmg_rate: 2.0,
            lucky_rate: 0.10,
            lucky_dmg_rate: 1.5,
            hits: 200.0,
            hits_per_minute: 3.3,
        }],
        skill_rows: vec![
            SkillRow {
                uid: 3602.0,
                name: "Skill 1".to_string(),
                total_dmg: 100_000.0,
                dps: 5_000.0,
                dmg_pct: 80.0,
                crit_rate: 0.30,
                crit_dmg_rate: 2.1,
                lucky_rate: 0.12,
                lucky_dmg_rate: 1.4,
                hits: 80.0,
                hits_per_minute: 1.5,
            },
            SkillRow {
                uid: 3602.0,
                name: "Skill 2".to_string(),
                total_dmg: 50_000.0,
                dps: 7_345.6,
                dmg_pct: 70.0,
                crit_rate: 0.20,
                crit_dmg_rate: 1.9,
                lucky_rate: 0.08,
                lucky_dmg_rate: 1.3,
                hits: 120.0,
                hits_per_minute: 1.8,
            },
            SkillRow {
                uid: 3602.0,
                name: "Skill 3".to_string(),
                total_dmg: 33_000.0,
                dps: 7_345.6,
                dmg_pct: 60.0,
                crit_rate: 0.20,
                crit_dmg_rate: 1.9,
                lucky_rate: 0.08,
                lucky_dmg_rate: 1.3,
                hits: 120.0,
                hits_per_minute: 1.8,
            },
            SkillRow {
                uid: 3602.0,
                name: "Skill 4".to_string(),
                total_dmg: 23_000.0,
                dps: 7_345.6,
                dmg_pct: 50.0,
                crit_rate: 0.20,
                crit_dmg_rate: 1.9,
                lucky_rate: 0.08,
                lucky_dmg_rate: 1.3,
                hits: 120.0,
                hits_per_minute: 1.8,
            },
            SkillRow {
                uid: 3602.0,
                name: "Skill 5".to_string(),
                total_dmg: 11_000.0,
                dps: 7_345.6,
                dmg_pct: 40.0,
                crit_rate: 0.20,
                crit_dmg_rate: 1.9,
                lucky_rate: 0.08,
                lucky_dmg_rate: 1.3,
                hits: 120.0,
                hits_per_minute: 1.8,
            },
            SkillRow {
                uid: 3602.0,
                name: "Skill 6".to_string(),
                total_dmg: 1_000.0,
                dps: 7_345.6,
                dmg_pct: 30.0,
                crit_rate: 0.20,
                crit_dmg_rate: 1.9,
                lucky_rate: 0.08,
                lucky_dmg_rate: 1.3,
                hits: 120.0,
                hits_per_minute: 1.8,
            },
            SkillRow {
                uid: 3602.0,
                name: "Skill 7".to_string(),
                total_dmg: 400.0,
                dps: 7_345.6,
                dmg_pct: 20.0,
                crit_rate: 0.20,
                crit_dmg_rate: 1.9,
                lucky_rate: 0.08,
                lucky_dmg_rate: 1.3,
                hits: 120.0,
                hits_per_minute: 1.8,
            },
        ],
    })
}
