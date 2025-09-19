use std::collections::HashMap;
use std::ops::{Div, DivAssign};
use crate::live::commands_models::{DPSRow, DPSRows, DPSWindow, SkillRow};
use crate::live::opcodes_models::{class, EncounterMutex, Skill};
use blueprotobuf_lib::blueprotobuf::EEntityType;
use log::info;
use once_cell::sync::Lazy;

#[tauri::command]
#[specta::specta]
pub fn get_damage_row(state: tauri::State<'_, EncounterMutex>) -> DPSWindow {
    DPSWindow {
        dps_rows: vec![
            DPSRow {
                uid: 1,
                name: "TestPlayer".to_string(),
                class: "Warrior".to_string(),
                ability_score: 1500,
                total_dmg: 123_456,
                dps: 12_345.6,
                crit_rate: 0.25,
                crit_dmg_rate: 2.0,
                lucky_rate: 0.10,
                lucky_dmg_rate: 1.5,
                hits: 200,
                hits_per_second: 3.3,
                skills: vec![
                    SkillRow {
                        name: "Slash".to_string(),
                        total_dmg: 50_000,
                        dps: 5_000.0,
                        crit_rate: 0.30,
                        crit_dmg_rate: 2.1,
                        lucky_rate: 0.12,
                        lucky_dmg_rate: 1.4,
                        hits: 80,
                        hits_per_second: 1.5,
                    },
                    SkillRow {
                        name: "Cleave".to_string(),
                        total_dmg: 73_456,
                        dps: 7_345.6,
                        crit_rate: 0.20,
                        crit_dmg_rate: 1.9,
                        lucky_rate: 0.08,
                        lucky_dmg_rate: 1.3,
                        hits: 120,
                        hits_per_second: 1.8,
                    },
                ],
            },
        ],
        total_dmg: 123_456,
        elapsed_ms: 10_000, // 10 seconds
    }
}

#[tauri::command]
#[specta::specta]
pub fn dafkpjjpaifget_damage_row(state: tauri::State<'_, EncounterMutex>) -> DPSWindow {
    let encounter = state.lock().unwrap();

    let mut dps_window = DPSWindow {
        dps_rows: DPSRows::default(),
        ..Default::default()
    };

    let time_elapsed_ms = encounter.time_last_combat_packet.saturating_sub(encounter.time_fight_start);
    let time_elapsed_secs = time_elapsed_ms as f64 / 1000.0;

    for (&entity_uid, entity) in &encounter.entity_uid_to_entity { // calculate things like dps
        let is_player = entity.entity_type == EEntityType::EntChar;
        let did_damage = !entity.skill_uid_to_skill.is_empty();
        // info!("{}, {is_player}", entity.name);
        if is_player && did_damage {


            // Damage Stats per player
            let mut damage_row = DPSRow {
                uid: entity_uid,
                name:
                if entity_uid == encounter.local_player_uid && entity.name.is_empty() {
                    String::from("You")
                } else if entity_uid == encounter.local_player_uid && !entity.name.is_empty() {
                    format!("{} (You)", entity.name)
                } else {
                    entity.name.clone()
                },
                class: class::to_string(entity.class_id),
                // class_style: String::new(),
                ability_score: entity.ability_score,
                ..Default::default()
            };

            let mut total_dmg = 0;
            let mut total_hits = 0;
            let mut crit_total_dmg_count = 0;
            let mut crit_hits_count = 0;
            let mut lucky_total_dmg_count = 0;
            let mut lucky_hits_count = 0;
            // Skills per player
            for (&skill_uid, skill) in &entity.skill_uid_to_skill {
                let skill_row = SkillRow {
                    name: Skill::get_skill_name(skill_uid),
                    total_dmg: skill.total_dmg,
                    dps: (skill.total_dmg as f64) / time_elapsed_secs.max(1.0) * 100.0,
                    hits: skill.hits,
                    hits_per_second: skill.hits as f64 / time_elapsed_secs.max(1.0) * 100.0,
                    crit_rate: skill.crit_hits as f64 / skill.hits as f64 * 100.0,
                    crit_dmg_rate: skill.crit_total_dmg as f64 / skill.total_dmg as f64 * 100.0,
                    lucky_rate: skill.lucky_hits as f64 / skill.hits as f64 * 100.0,
                    lucky_dmg_rate: skill.lucky_total_dmg as f64 / skill.total_dmg as f64 * 100.0,
                    ..Default::default()
                };
                damage_row.skills.push(skill_row);

                total_dmg += skill.total_dmg;
                total_hits += skill.hits;
                crit_total_dmg_count += skill.crit_total_dmg;
                crit_hits_count += skill.crit_hits;
                lucky_total_dmg_count += skill.lucky_total_dmg;
                lucky_hits_count += skill.lucky_hits;
            }

            damage_row.dps = total_dmg as f64 / time_elapsed_secs.max(1.0);
            damage_row.crit_rate = crit_hits_count as f64 / total_hits as f64 * 100.0;
            damage_row.crit_dmg_rate = crit_total_dmg_count as f64 / total_dmg as f64 * 100.0;
            damage_row.lucky_rate = lucky_hits_count as f64 / total_hits as f64 * 100.0;
            damage_row.lucky_dmg_rate = lucky_total_dmg_count as f64 / total_dmg as f64 * 100.0;
            damage_row.hits = total_hits;
            damage_row.hits_per_second = total_hits as f64 / time_elapsed_secs.max(1.0);

            // Sort skills descending by DPS
            damage_row.skills.sort_by(|this_row, other_row| {
                other_row.total_dmg
                    .partial_cmp(&this_row.total_dmg) // descending
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            damage_row.total_dmg += total_dmg;
            dps_window.dps_rows.push(damage_row);
            dps_window.total_dmg += total_dmg;
        }
    }
    dps_window.elapsed_ms = time_elapsed_ms;
    dps_window.dps_rows.sort_by(|this_row, other_row| { // sort descending by dps
        other_row.total_dmg
            .partial_cmp(&this_row.total_dmg) // descending
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    dps_window
}
