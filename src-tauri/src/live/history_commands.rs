use crate::db::{self, DbConnection, EncounterDetail, EncounterRecord};
use crate::live::commands_models::{PlayerRow, PlayersWindow, SkillRow, SkillsWindow};
use log::error;

#[tauri::command]
#[specta::specta]
pub async fn get_all_encounter_history(state: tauri::State<'_, DbConnection>) -> Result<Vec<EncounterRecord>, String> {
    db::get_all_encounters(&state)
        .await
        .map_err(|e| {
            error!("Failed to get encounters: {}", e);
            format!("Failed to get encounters: {}", e)
        })
}

#[tauri::command]
#[specta::specta]
pub async fn get_encounter_detail(
    state: tauri::State<'_, DbConnection>,
    encounter_id: i64,
) -> Result<EncounterDetail, String> {
    db::get_encounter_detail(&state, encounter_id)
        .await
        .map_err(|e| {
            error!("Failed to get encounter detail: {}", e);
            format!("Failed to get encounter detail: {}", e)
        })?
        .ok_or_else(|| format!("Encounter {} not found", encounter_id))
}

#[tauri::command]
#[specta::specta]
pub async fn delete_encounter_history(state: tauri::State<'_, DbConnection>, encounter_id: i64) -> Result<(), String> {
    db::delete_encounter(&state, encounter_id)
        .await
        .map_err(|e| {
            error!("Failed to delete encounter: {}", e);
            format!("Failed to delete encounter: {}", e)
        })
}

#[tauri::command]
#[specta::specta]
pub async fn clear_all_data(
    state: tauri::State<'_, DbConnection>,
    encounter_state: tauri::State<'_, crate::live::opcodes_models::EncounterMutex>,
) -> Result<(), String> {
    // Clear all encounter history from the database
    db::clear_all_encounters(&state)
        .await
        .map_err(|e| {
            error!("Failed to clear all encounters: {}", e);
            format!("Failed to clear all encounters: {}", e)
        })?;

    // Reset the current encounter
    let mut encounter = encounter_state.lock().unwrap();
    encounter.clone_from(&crate::live::opcodes_models::Encounter::default());
    
    Ok(())
}

fn nan_is_zero(value: f64) -> f64 {
    if value.is_nan() || value.is_infinite() {
        0.0
    } else {
        value
    }
}

/// Look up ability_score from historical data for a player UID
async fn lookup_player_ability_score(db: &tauri::State<'_, DbConnection>, player_uid: i64) -> Option<i32> {
    db::lookup_player_ability_score_from_history(db, player_uid).await.ok().flatten()
}

#[tauri::command]
#[specta::specta]
pub async fn get_historical_players_window(
    state: tauri::State<'_, DbConnection>,
    encounter_id: i64,
) -> Result<PlayersWindow, String> {
    let detail = db::get_encounter_detail(&state, encounter_id)
        .await
        .map_err(|e| {
            error!("Failed to get encounter detail: {}", e);
            format!("Failed to get encounter detail: {}", e)
        })?
        .ok_or_else(|| format!("Encounter {} not found", encounter_id))?;

    #[allow(clippy::cast_precision_loss)]
    let time_elapsed_secs = detail.encounter.duration_ms as f64 / 1000.0;

    let mut player_window = PlayersWindow {
        player_rows: Vec::new(),
        local_player_uid: -1.0,
        top_value: 0.0,
    };

    for player_detail in &detail.players {
        let player = &player_detail.player;
        if player.total_damage == 0 && player.total_healing == 0 {
            continue;
        }

        player_window.top_value = player_window.top_value.max(player.total_damage as f64);

        // Try to get ability_score from current record, fall back to historical data if None
        let ability_score = match player.ability_score {
            Some(score) => Some(score),
            None => lookup_player_ability_score(&state, player.id).await,
        };

        #[allow(clippy::cast_precision_loss)]
        let player_row = PlayerRow {
            uid: player.id as f64,
            name: player.name.clone(),
            class_name: player.class.clone().unwrap_or_default(),
            class_spec_name: player.class_spec.clone().unwrap_or_default(),
            ability_score: ability_score.map(|score| score as f64).unwrap_or(-1.0),
            total_value: player.total_damage as f64,
            value_per_sec: nan_is_zero(player.total_damage as f64 / time_elapsed_secs),
            value_pct: nan_is_zero(player.total_damage as f64 / detail.encounter.total_damage as f64 * 100.0),
            crit_rate: nan_is_zero(player.crit_hits as f64 / player.damage_hits as f64 * 100.0),
            crit_value_rate: nan_is_zero(player.crit_value as f64 / player.total_damage as f64 * 100.0),
            lucky_rate: nan_is_zero(player.lucky_hits as f64 / player.damage_hits as f64 * 100.0),
            lucky_value_rate: nan_is_zero(player.lucky_value as f64 / player.total_damage as f64 * 100.0),
            hits: player.damage_hits as f64,
            hits_per_minute: nan_is_zero(player.damage_hits as f64 / time_elapsed_secs * 60.0),
        };
        player_window.player_rows.push(player_row);
    }

    // Sort players descending by damage dealt
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
pub async fn get_historical_skills_window(
    state: tauri::State<'_, DbConnection>,
    encounter_id: i64,
    player_id: i64,
) -> Result<SkillsWindow, String> {
    let detail = db::get_encounter_detail(&state, encounter_id)
        .await
        .map_err(|e| {
            error!("Failed to get encounter detail: {}", e);
            format!("Failed to get encounter detail: {}", e)
        })?
        .ok_or_else(|| format!("Encounter {} not found", encounter_id))?;

    #[allow(clippy::cast_precision_loss)]
    let time_elapsed_secs = detail.encounter.duration_ms as f64 / 1000.0;

    let player_detail = detail
        .players
        .iter()
        .find(|pd| pd.player.id == player_id)
        .ok_or_else(|| format!("Player {} not found in encounter", player_id))?;

    let player = &player_detail.player;

    // Try to get ability_score from current record, fall back to historical data if None
    let ability_score = match player.ability_score {
        Some(score) => Some(score),
        None => lookup_player_ability_score(&state, player.id).await,
    };

    #[allow(clippy::cast_precision_loss)]
    let mut skill_window = SkillsWindow {
        inspected_player: PlayerRow {
            uid: player.id as f64,
            name: player.name.clone(),
            class_name: player.class.clone().unwrap_or_default(),
            class_spec_name: player.class_spec.clone().unwrap_or_default(),
            ability_score: ability_score.map(|score| score as f64).unwrap_or(-1.0),
            total_value: player.total_damage as f64,
            value_per_sec: nan_is_zero(player.total_damage as f64 / time_elapsed_secs),
            value_pct: nan_is_zero(player.total_damage as f64 / detail.encounter.total_damage as f64 * 100.0),
            crit_rate: nan_is_zero(player.crit_hits as f64 / player.damage_hits as f64 * 100.0),
            crit_value_rate: nan_is_zero(player.crit_value as f64 / player.total_damage as f64 * 100.0),
            lucky_rate: nan_is_zero(player.lucky_hits as f64 / player.damage_hits as f64 * 100.0),
            lucky_value_rate: nan_is_zero(player.lucky_value as f64 / player.total_damage as f64 * 100.0),
            hits: player.damage_hits as f64,
            hits_per_minute: nan_is_zero(player.damage_hits as f64 / time_elapsed_secs * 60.0),
        },
        local_player_uid: -1.0,
        skill_rows: Vec::new(),
        top_value: 0.0,
    };

    // Add skills for this player
    for ability in &player_detail.abilities {
        skill_window.top_value = skill_window.top_value.max(ability.total_damage as f64);

        #[allow(clippy::cast_precision_loss)]
        let skill_row = SkillRow {
            uid: ability.skill_id as f64,
            name: ability.skill_name.clone(),
            total_value: ability.total_damage as f64,
            value_per_sec: nan_is_zero(ability.total_damage as f64 / time_elapsed_secs),
            value_pct: nan_is_zero(ability.total_damage as f64 / player.total_damage as f64 * 100.0),
            crit_rate: nan_is_zero(ability.crit_hits as f64 / ability.damage_hits as f64 * 100.0),
            crit_value_rate: nan_is_zero(ability.crit_value as f64 / ability.total_damage as f64 * 100.0),
            lucky_rate: nan_is_zero(ability.lucky_hits as f64 / ability.damage_hits as f64 * 100.0),
            lucky_value_rate: nan_is_zero(ability.lucky_value as f64 / ability.total_damage as f64 * 100.0),
            hits: ability.damage_hits as f64,
            hits_per_minute: nan_is_zero(ability.damage_hits as f64 / time_elapsed_secs * 60.0),
        };
        skill_window.skill_rows.push(skill_row);
    }

    // Sort skills descending by damage dealt
    skill_window.skill_rows.sort_by(|this_row, other_row| {
        other_row
            .total_value
            .partial_cmp(&this_row.total_value)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(skill_window)
}
