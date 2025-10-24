use crate::live::opcodes_models;
use crate::live::opcodes_models::class::{
    ClassSpec, get_class_id_from_spec, get_class_spec_from_skill_id,
};
use crate::live::utils::{is_boss};
use crate::live::opcodes_models::{
    Encounter, Entity, MONSTER_NAMES, MONSTER_NAMES_CROWDSOURCE, Skill, attr_type,
};
use crate::packets::utils::BinaryReader;
use blueprotobuf_lib::blueprotobuf;
use blueprotobuf_lib::blueprotobuf::{Attr, EDamageType, EEntityType, SyncContainerData};
use log::info;
use serde::Serialize;
use std::default::Default;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn on_server_change(encounter: &mut Encounter) {
    info!("on server change");
    encounter.clone_from(&Encounter::default());
}

pub fn process_sync_near_entities(
    encounter: &mut Encounter,
    sync_near_entities: blueprotobuf::SyncNearEntities,
) -> Option<()> {
    for pkt_entity in sync_near_entities.appear {
        let target_uuid = pkt_entity.uuid?;
        let target_uid = target_uuid >> 16;
        let target_entity_type = EEntityType::from(target_uuid);

        let target_entity = encounter
            .entity_uid_to_entity
            .entry(target_uid)
            .or_default();
        target_entity.entity_type = target_entity_type;

        match target_entity_type {
            EEntityType::EntChar => {
                process_player_attrs(target_entity, target_uid, pkt_entity.attrs?.attrs)
            }
            EEntityType::EntMonster => process_monster_attrs(
                target_entity,
                target_uid,
                pkt_entity.attrs?.attrs,
                &encounter.local_player,
            ),
            _ => {}
        }
    }
    Some(())
}

pub fn process_sync_container_data(
    encounter: &mut Encounter,
    sync_container_data: blueprotobuf::SyncContainerData,
) -> Option<()> {
    let v_data = sync_container_data.v_data?;
    let player_uid = v_data.char_id?;

    let target_entity = encounter
        .entity_uid_to_entity
        .entry(player_uid)
        .or_default();
    let char_base = v_data.char_base?;
    target_entity.name = char_base.name?;
    target_entity.entity_type = EEntityType::EntChar;
    target_entity.class_id = v_data.profession_list?.cur_profession_id?;
    target_entity.ability_score = char_base.fight_point?;
    target_entity.level = v_data.role_level?.level?;

    Some(())
}

pub fn process_sync_container_dirty_data(
    encounter: &mut Encounter,
    sync_container_dirty_data: blueprotobuf::SyncContainerDirtyData,
) -> Option<()> {
    Some(())
}

pub fn process_sync_to_me_delta_info(
    encounter: &mut Encounter,
    sync_to_me_delta_info: blueprotobuf::SyncToMeDeltaInfo,
) -> Option<()> {
    let delta_info = sync_to_me_delta_info.delta_info?;
    encounter.local_player_uid = delta_info.uuid? >> 16; // UUID =/= uid (have to >> 16) // todo: add my UID here
    process_aoi_sync_delta(encounter, delta_info.base_delta?);
    Some(())
}

pub fn process_aoi_sync_delta(
    encounter: &mut Encounter,
    aoi_sync_delta: blueprotobuf::AoiSyncDelta,
) -> Option<()> {
    let target_uuid = aoi_sync_delta.uuid?; // UUID =/= uid (have to >> 16)
    let target_uid = target_uuid >> 16;
    let boss = is_boss(target_uid);
    // Process attributes
    let target_entity_type = EEntityType::from(target_uuid);
    let mut target_entity = encounter
        .entity_uid_to_entity
        .entry(target_uid)
        .or_insert_with(|| Entity {
            entity_type: target_entity_type,
            ..Default::default()
        });

    if let Some(attrs_collection) = aoi_sync_delta.attrs {
        match target_entity_type {
            EEntityType::EntChar => {
                process_player_attrs(&mut target_entity, target_uid, attrs_collection.attrs)
            }
            EEntityType::EntMonster => process_monster_attrs(
                &mut target_entity,
                target_uid,
                attrs_collection.attrs,
                &encounter.local_player,
            ),
            _ => {}
        }
    }

    let Some(skill_effect) = aoi_sync_delta.skill_effects else {
        return Some(()); // return ok since this variable usually doesn't exist
    };

    // Process Damage
    for sync_damage_info in skill_effect.damages {
        let non_lucky_dmg = sync_damage_info.value;
        let lucky_value = sync_damage_info.lucky_value;

        #[allow(clippy::cast_sign_loss)]
        let actual_value = if let Some(actual_dmg) = non_lucky_dmg.or(lucky_value) {
            actual_dmg
        } else {
            continue; // skip this iteration
        };

        let attacker_uuid = sync_damage_info
            .top_summoner_id
            .or(sync_damage_info.attacker_uuid)?;
        let attacker_uid = attacker_uuid >> 16;
        let attacker_entity = encounter
            .entity_uid_to_entity
            .entry(attacker_uid)
            .or_insert_with(|| Entity {
                // name: format!("dummy-name-{attacker_uid}"),
                entity_type: EEntityType::from(attacker_uuid),
                ..Default::default()
            });

        // Skills
        let skill_uid = sync_damage_info.owner_id?;
        if attacker_entity.class_spec == ClassSpec::Unknown {
            let class_spec = get_class_spec_from_skill_id(skill_uid);
            attacker_entity.class_id = get_class_id_from_spec(class_spec);
            attacker_entity.class_spec = class_spec;
        }

        let is_heal = sync_damage_info.r#type.unwrap_or(0) == EDamageType::Heal as i32;
        if is_heal {
            let skill = attacker_entity
                .skill_uid_to_heal_skill
                .entry(skill_uid)
                .or_insert_with(|| Skill::default());
            // TODO: from testing, first bit is set when there's crit, 3rd bit for if it causes lucky (no idea what that means), require more testing here
            const CRIT_BIT: i32 = 0b00_00_00_01; // 1st bit
            let is_lucky = lucky_value.is_some();
            let flag = sync_damage_info.type_flag.unwrap_or_default();
            let is_crit = (flag & CRIT_BIT) != 0; // No idea why, but SyncDamageInfo.is_crit isn't correct
            if is_crit {
                attacker_entity.crit_hits_heal += 1;
                attacker_entity.crit_total_heal += actual_value;
                skill.crit_hits += 1;
                skill.crit_total_value += actual_value;
            }
            if is_lucky {
                attacker_entity.lucky_hits_heal += 1;
                attacker_entity.lucky_total_heal += actual_value;
                skill.lucky_hits += 1;
                skill.lucky_total_value += actual_value;
            }
            encounter.total_heal += actual_value;
            attacker_entity.hits_heal += 1;
            attacker_entity.total_heal += actual_value;
            skill.hits += 1;
            skill.total_value += actual_value;
            info!(
                "heal packet: {attacker_uid} to {target_uid}: {actual_value} heal {} total heal",
                skill.total_value
            );
        } else {
            let skill = attacker_entity
                .skill_uid_to_dmg_skill
                .entry(skill_uid)
                .or_insert_with(|| Skill::default());
            // TODO: from testing, first bit is set when there's crit, 3rd bit for if it causes lucky (no idea what that means), require more testing here
            const CRIT_BIT: i32 = 0b00_00_00_01; // 1st bit
            let is_lucky = lucky_value.is_some();
            let flag = sync_damage_info.type_flag.unwrap_or_default();
            let is_crit = (flag & CRIT_BIT) != 0; // No idea why, but SyncDamageInfo.is_crit isn't correct
            if is_crit {
                if boss {
                    attacker_entity.crit_hits_dmg_boss += 1;
                    attacker_entity.crit_total_dmg_boss += actual_value;
                    skill.crit_hits_boss += 1;
                    skill.crit_total_value_boss += actual_value;
                }
                attacker_entity.crit_hits_dmg += 1;
                attacker_entity.crit_total_dmg += actual_value;
                skill.crit_hits += 1;
                skill.crit_total_value += actual_value;
            }
            if is_lucky {
                if boss {
                    attacker_entity.lucky_hits_dmg_boss += 1;
                    attacker_entity.lucky_total_dmg_boss += actual_value;
                    skill.lucky_hits_boss += 1;
                    skill.lucky_total_value_boss += actual_value;
                }
                attacker_entity.lucky_hits_dmg += 1;
                attacker_entity.lucky_total_dmg += actual_value;
                skill.lucky_hits += 1;
                skill.lucky_total_value += actual_value;
            }
            if boss {
                encounter.total_dmg_boss += actual_value;
                attacker_entity.hits_dmg_boss += 1;
                attacker_entity.total_dmg_boss += actual_value;
                skill.hits_boss += 1;
                skill.total_value_boss += actual_value;
            }
            encounter.total_dmg += actual_value;
            attacker_entity.hits_dmg += 1;
            attacker_entity.total_dmg += actual_value;
            skill.hits += 1;
            skill.total_value += actual_value;
            info!(
                "dmg packet: {attacker_uid} to {target_uid}: {actual_value} dmg {} total dmg",
                skill.total_value
            );
        } 
    }

    // Figure out timestamps
    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();
    if encounter.time_fight_start_ms == Default::default() { 
        encounter.time_fight_start_ms = timestamp_ms
    }
    if encounter.time_fight_start_ms_boss == Default::default() && boss {
            encounter.time_fight_start_ms_boss = timestamp_ms
    }
    if boss {
        encounter.time_last_combat_packet_ms_boss = timestamp_ms;
    }
    encounter.time_last_combat_packet_ms = timestamp_ms;
    
    Some(())
}

fn process_player_attrs(player_entity: &mut Entity, player_uid: i64, attrs: Vec<Attr>) {
    for attr in attrs {
        let Some(mut raw_bytes) = attr.raw_data else {
            continue;
        };
        let Some(attr_id) = attr.id else { continue };

        // info!("{} {}", attr_type::(attr_id),hex::encode(raw_bytes.read_remaining()));
        match attr_id {
            attr_type::ATTR_NAME => {
                raw_bytes.remove(0); // not sure why, there's some weird character as the first e.g. "\u{6}Sketal"
                let player_name = BinaryReader::from(raw_bytes).read_string().unwrap();
                player_entity.name = player_name;
                info! {"Found player {} with UID {}", player_entity.name, player_uid}
            }
            #[allow(clippy::cast_possible_truncation)]
            attr_type::ATTR_PROFESSION_ID => {
                player_entity.class_id =
                    prost::encoding::decode_varint(&mut raw_bytes.as_slice()).unwrap() as i32
            }
            #[allow(clippy::cast_possible_truncation)]
            attr_type::ATTR_FIGHT_POINT => {
                player_entity.ability_score =
                    prost::encoding::decode_varint(&mut raw_bytes.as_slice()).unwrap() as i32
            }
            #[allow(clippy::cast_possible_truncation)]
            attr_type::ATTR_LEVEL => {
                player_entity.level =
                    prost::encoding::decode_varint(&mut raw_bytes.as_slice()).unwrap() as i32
            }
            _ => (),
        }
    }
}

fn process_monster_attrs(
    monster_entity: &mut Entity,
    monster_uid: i64,
    attrs: Vec<Attr>,
    local_player: &SyncContainerData,
) {
    for attr in attrs {
        let Some(mut raw_bytes) = attr.raw_data else {
            continue;
        };
        let Some(attr_id) = attr.id else { continue };

        match attr_id {
            attr_type::ATTR_ID => {
                monster_entity.monster_id =
                    prost::encoding::decode_varint(&mut raw_bytes.as_slice()).unwrap() as i32
            }
            #[allow(clippy::cast_possible_truncation)]
            attr_type::ATTR_HP => {
                let curr_hp =
                    prost::encoding::decode_varint(&mut raw_bytes.as_slice()).unwrap() as i32;
                // Crowdsource Data: if people abuse this, we will change the security
                // const ENDPOINT: &str = "http://localhost:3000";
                const ENDPOINT: &str = "https://db.bptimer.com/api/create-hp-report";
                const API_KEY: &str = "8fibznvjgf9vh29bg7g730fan9xaskf7h45lzdl2891vi0w1d2";
                if monster_entity.curr_hp != curr_hp {
                    // only record if hp changed
                    let monster_id = monster_entity.monster_id;
                    if MONSTER_NAMES_CROWDSOURCE.get(&monster_id).is_some() {
                        // only record if it's a world boss, magical creature, etc.
                        let monster_name = MONSTER_NAMES
                            .get(&monster_id)
                            .map(|s| s.as_str())
                            .unwrap_or("Unknown Monster Name");
                        let hp_pct = if monster_entity.curr_hp > 0 && monster_entity.max_hp > 0 {
                            Some(
                                (monster_entity.curr_hp * 100 / monster_entity.max_hp)
                                    .clamp(0, 100),
                            )
                        } else {
                            None
                        };
                        let line = local_player
                            .v_data
                            .as_ref()
                            .and_then(|v| v.scene_data.as_ref().and_then(|s| s.line_id));
                        // TODO: this position is snapshot based on when SyncContainerData is detected (e.g. line change), figure out if there's a way to get the monster's position instead
                        let pos_x = local_player.v_data.as_ref().and_then(|v| {
                            v.scene_data
                                .as_ref()
                                .and_then(|v| v.pos.as_ref().and_then(|s| s.x))
                        });
                        let pos_y = local_player.v_data.as_ref().and_then(|v| {
                            v.scene_data
                                .as_ref()
                                .and_then(|v| v.pos.as_ref().and_then(|s| s.y))
                        });
                        if let (Some(hp_pct), Some(line), Some(pos_x), Some(pos_y)) =
                            (hp_pct, line, pos_x, pos_y)
                        {
                            info!(
                                "Found crowdsourced monster with Name {monster_name} - ID {monster_id} - HP% {hp_pct}% on line {line} and pos ({pos_x},{pos_y})"
                            );
                            let body = serde_json::json!({
                                "monster_id": monster_id,
                                "hp_pct": hp_pct,
                                "line": line,
                                "pos_x": pos_x,
                                "pos_y": pos_y,
                            });
                            let _ = tokio::spawn(async move {
                                let client = reqwest::Client::new();
                                let res = client
                                    .post(ENDPOINT)
                                    .header("X-API-Key", API_KEY)
                                    .json(&body)
                                    .send()
                                    .await;
                                match res {
                                    Ok(resp) => {
                                        if resp.status() != reqwest::StatusCode::OK {
                                            log::error!(
                                                "POST monster info failed: status {}",
                                                resp.status()
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        log::error!("Failed to POST monster info: {}", e);
                                    }
                                }
                            });
                        }
                    }
                }
                monster_entity.curr_hp = curr_hp;
            }
            #[allow(clippy::cast_possible_truncation)]
            attr_type::ATTR_MAX_HP => {
                monster_entity.max_hp =
                    prost::encoding::decode_varint(&mut raw_bytes.as_slice()).unwrap() as i32
            }
            _ => (),
        }
    }
}
