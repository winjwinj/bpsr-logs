use crate::live::bptimer_stream::{
    BPTIMER_BASE_URL, CREATE_HP_REPORT_ENDPOINT, CROWD_SOURCE_API_KEY,
};
use crate::live::opcodes_models::class::{get_class_from_spec, get_class_spec_from_skill_id, Class, ClassSpec};
use crate::live::opcodes_models::{
    attr_type, CombatStats, Encounter, Entity, MONSTER_NAMES, MONSTER_NAMES_BOSS,
    MONSTER_NAMES_CROWDSOURCE, MONSTER_UID_CROWDSOURCE_MAP,
};
use crate::packets::utils::BinaryReader;
use blueprotobuf_lib::blueprotobuf;
use log::{error, info, warn};
use std::default::Default;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn on_server_change(encounter: &mut Encounter) {
    info!("on server change");
    encounter.clone_from(&Encounter::default());
}

pub fn process_sync_near_entities(
    encounter: &mut Encounter,
    sync_near_entities: blueprotobuf::SyncNearEntities,
    is_bptimer_enabled: bool,
) -> Option<()> {
    for pkt_entity in sync_near_entities.appear {
        let target_uuid = pkt_entity.uuid?;
        let target_uid = target_uuid >> 16;
        let target_entity_type = blueprotobuf::EEntityType::from(target_uuid);

        let target_entity = encounter
            .entity_uid_to_entity
            .entry(target_uid)
            .or_default();
        target_entity.entity_type = target_entity_type;

        match target_entity_type {
            blueprotobuf::EEntityType::EntChar => process_player_attrs(target_entity, target_uid, pkt_entity.attrs?.attrs),
            blueprotobuf::EEntityType::EntMonster => process_monster_attrs(target_entity, pkt_entity.attrs?.attrs, encounter.local_player.as_ref(), is_bptimer_enabled),
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
    target_entity.name = Some(char_base.name?);
    target_entity.entity_type = blueprotobuf::EEntityType::EntChar;
    target_entity.class = Some(Class::from(v_data.profession_list?.cur_profession_id?));
    target_entity.ability_score = Some(char_base.fight_point?);

    Some(())
}

// pub fn process_sync_container_dirty_data(
//     encounter: &mut Encounter,
//     sync_container_dirty_data: blueprotobuf::SyncContainerDirtyData,
// ) -> Option<()> {
//     Some(())
// }

pub fn process_sync_to_me_delta_info(
    encounter: &mut Encounter,
    sync_to_me_delta_info: blueprotobuf::SyncToMeDeltaInfo,
    is_bptimer_enabled: bool,
) -> Option<()> {
    let delta_info = sync_to_me_delta_info.delta_info?;
    encounter.local_player_uid = Some(delta_info.uuid? >> 16); // UUID =/= uid (have to >> 16)
    process_aoi_sync_delta(encounter, delta_info.base_delta?, is_bptimer_enabled);
    Some(())
}

pub fn process_aoi_sync_delta(
    encounter: &mut Encounter,
    aoi_sync_delta: blueprotobuf::AoiSyncDelta,
    is_bptimer_enabled: bool,
) -> Option<()> {
    let target_uuid = aoi_sync_delta.uuid?; // UUID =/= uid (have to >> 16)
    let target_uid = target_uuid >> 16;

    // Process attributes
    let target_entity_type = blueprotobuf::EEntityType::from(target_uuid);
    {
        let target_entity = encounter
            .entity_uid_to_entity
            .entry(target_uid)
            .or_insert_with(|| Entity {
                entity_type: target_entity_type,
                ..Default::default()
            });

        if let Some(attrs_collection) = aoi_sync_delta.attrs {
            match target_entity_type {
                blueprotobuf::EEntityType::EntChar => process_player_attrs(target_entity, target_uid, attrs_collection.attrs),
                blueprotobuf::EEntityType::EntMonster => process_monster_attrs(target_entity, attrs_collection.attrs, encounter.local_player.as_ref(), is_bptimer_enabled),
                _ => {}
            }
        }
    }

    let Some(skill_effect) = aoi_sync_delta.skill_effects else {
        return Some(()); // return ok since this variable usually doesn't exist
    };

    // Process Damage
    for sync_damage_info in skill_effect.damages {
        let target_entity = encounter.entity_uid_to_entity.get(&target_uid);
        let monster_id = target_entity.and_then(|e| e.monster_id);
        let is_boss = monster_id
            .is_some_and(|id| MONSTER_NAMES_BOSS.contains_key(&id));
        let is_crowdsource = monster_id
            .is_some_and(|id| MONSTER_NAMES_CROWDSOURCE.contains_key(&id));
        let crowdsource_name = if is_crowdsource {
        monster_id
                .and_then(|id| MONSTER_NAMES.get(&id))
                .or_else(|| target_entity.and_then(|e| e.name.as_ref()))
                .cloned()
        } else {
            None
        };
        // info!("crowdsource_name={crowdsource_name:?}, monster_id={monster_id:?}");
        let crowdsource_remote_id = monster_id
            .and_then(|id| MONSTER_UID_CROWDSOURCE_MAP.get(&id).cloned());
        let attacker_uuid = sync_damage_info
            .top_summoner_id
            .or(sync_damage_info.attacker_uuid)?;
        let attacker_uid = attacker_uuid >> 16;
        let attacker_entity = encounter.entity_uid_to_entity
                                       .entry(attacker_uid)
                                       .or_insert_with(|| Entity {
                                           entity_type: blueprotobuf::EEntityType::from(attacker_uuid),
                                           ..Default::default()
                                       });

        let skill_uid = sync_damage_info.owner_id?;
        if attacker_entity.class_spec.is_none_or(|class_spec| class_spec == ClassSpec::Unknown) {
            let class_spec = get_class_spec_from_skill_id(skill_uid);
            attacker_entity.class = Some(get_class_from_spec(class_spec));
            attacker_entity.class_spec = Some(class_spec);
        }

        // Skills
        let is_heal = sync_damage_info.r#type.unwrap_or(0) == blueprotobuf::EDamageType::Heal as i32;
        if is_heal {
            let heal_skill = attacker_entity
                .skill_uid_to_dps_stats
                .entry(skill_uid)
                .or_default();
            process_stats(&sync_damage_info, heal_skill);
            process_stats(&sync_damage_info, &mut attacker_entity.heal_stats); // update total entity heal stats
            process_stats(&sync_damage_info, &mut encounter.heal_stats); // update total encounter heal stats
            // info!("dmg packet: {attacker_uid} to {target_uid}: {} total heal", heal_skill.value);
        } else {
            let dps_skill = attacker_entity
                .skill_uid_to_dps_stats
                .entry(skill_uid)
                .or_default();
            process_stats(&sync_damage_info, dps_skill);
            process_stats(&sync_damage_info, &mut attacker_entity.dmg_stats); // update total entity dmg stats
            process_stats(&sync_damage_info, &mut encounter.dmg_stats); // update total encounter heal stats
            
            // Track last hit crowdsourced monster name and ID
            if is_crowdsource {
                if crowdsource_remote_id.is_none() {
                    warn!(
                        "live::opcodes_process::handle_damage_packet - crowdsourced monster missing remote id for monster_id={monster_id:?}, monster_name={crowdsource_name:?}"
                    );
                }
                encounter.crowdsource_monster_name = crowdsource_name.clone();
                encounter.crowdsource_monster_id = monster_id;
                encounter.crowdsource_monster_remote_id = crowdsource_remote_id.clone();
            }
            
            if is_boss {
                let skill_boss_only = attacker_entity
                    .skill_uid_to_dps_stats_boss_only
                    .entry(skill_uid)
                    .or_default();
                process_stats(&sync_damage_info, skill_boss_only);
                process_stats(&sync_damage_info, &mut attacker_entity.dmg_stats_boss_only); // update total entity boss only dmg stats
                process_stats(&sync_damage_info, &mut encounter.dmg_stats_boss_only); // update total encounter heal stats
            }
            // info!("dmg packet: {attacker_uid} to {target_uid}: {} total dmg", dps_skill.value);
        }
    }

    // Figure out timestamps
    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();
    if encounter.time_fight_start_ms == Default::default() {
        encounter.time_fight_start_ms = timestamp_ms;
    }
    encounter.time_last_combat_packet_ms = timestamp_ms;
    Some(())
}

fn process_stats(sync_damage_info: &blueprotobuf::SyncDamageInfo, stats: &mut CombatStats) {
    // TODO: from testing, first bit is set when there's crit, 3rd bit for if it causes lucky (no idea what that means), require more testing here
    const CRIT_BIT: i32 = 0b00_00_00_01; // 1st bit

    let non_lucky_dmg = sync_damage_info.value;
    let lucky_value = sync_damage_info.lucky_value;
    let actual_value = non_lucky_dmg.or(lucky_value).unwrap_or(0); // The damage is either non-lucky or lucky (exclusive)

    let is_lucky = lucky_value.is_some();
    let flag = sync_damage_info.type_flag.unwrap_or_default();
    let is_crit = (flag & CRIT_BIT) != 0; // No idea why, but SyncDamageInfo.is_crit isn't correct
    if is_crit {
        stats.crit_hits += 1;
        stats.crit_value += actual_value;
    }
    if is_lucky {
        stats.lucky_hits += 1;
        stats.lucky_value += actual_value;
    }
    stats.hits += 1;
    stats.value += actual_value;
}

fn process_player_attrs(player_entity: &mut Entity, player_uid: i64, attrs: Vec<blueprotobuf::Attr>) {
    for attr in attrs {
        let Some(mut raw_bytes) = attr.raw_data else {
            continue;
        };
        let Some(attr_id) = attr.id else { continue; };

        // info!("{} {}", attr_type::(attr_id),hex::encode(raw_bytes.read_remaining()));
        match attr_id {
            attr_type::ATTR_NAME => {
                raw_bytes.remove(0); // not sure why, there's some weird character as the first e.g. "\u{6}Sketal"
                let player_name_result = BinaryReader::from(raw_bytes).read_string();
                if let Ok(player_name) = player_name_result {
                    player_entity.name = Some(player_name.clone());
                    info!("Found player {player_name} with UID {player_uid}");
                } else {
                    warn!("Failed to read player name for UID {player_uid}");
                }
            }
            #[allow(clippy::cast_possible_truncation)]
            attr_type::ATTR_PROFESSION_ID => player_entity.class = Some(Class::from(prost::encoding::decode_varint(&mut raw_bytes.as_slice()).unwrap() as i32)),
            #[allow(clippy::cast_possible_truncation)]
            attr_type::ATTR_FIGHT_POINT => player_entity.ability_score = Some(prost::encoding::decode_varint(&mut raw_bytes.as_slice()).unwrap() as i32),
            _ => (),
        }
    }
}

fn process_monster_attrs(
    monster_entity: &mut Entity,
    attrs: Vec<blueprotobuf::Attr>,
    local_player: Option<&blueprotobuf::SyncContainerData>,
    is_bptimer_enabled: bool,
) {
    for attr in attrs {
        let Some(raw_bytes) = attr.raw_data else { continue; };
        let Some(attr_id) = attr.id else { continue; };

        #[allow(clippy::cast_possible_truncation)]
        match attr_id {
            attr_type::ATTR_ID => monster_entity.monster_id = Some(prost::encoding::decode_varint(&mut raw_bytes.as_slice()).unwrap() as i32),
            attr_type::ATTR_HP => {
                let curr_hp = prost::encoding::decode_varint(&mut raw_bytes.as_slice()).unwrap() as i32;
                let prev_hp_opt = monster_entity.curr_hp;
                monster_entity.curr_hp = Some(curr_hp);

                if is_bptimer_enabled {
                    // Crowdsource Data: if people abuse this, we will change the security
                    #[allow(dead_code)]
                    const LOCAL_ENDPOINT: &str = "http://localhost:3000";
                    let endpoint = format!("{BPTIMER_BASE_URL}{CREATE_HP_REPORT_ENDPOINT}");
                    let (Some(monster_id), Some(local_player)) = (monster_entity.monster_id, &local_player) else {
                        continue;
                    };
                    let Some(max_hp) = monster_entity.max_hp else {
                        continue;
                    };
                    if MONSTER_NAMES_CROWDSOURCE.contains_key(&monster_id) { // only record if it's a world boss, magical creature, etc.
                        let monster_name = MONSTER_NAMES.get(&monster_id).map_or("Unknown Monster Name", |s| s.as_str());
                        let new_hp_pct = (curr_hp * 100 / max_hp).clamp(0, 100);
                        let old_hp_pct_opt =
                            prev_hp_opt.map(|prev_hp| (prev_hp * 100 / max_hp).clamp(0, 100));
                        let Some((Some(line), Some(pos_x), Some(pos_y))) = local_player.v_data.as_ref()
                                                                                       .and_then(|v| v.scene_data.as_ref())
                                                                                       .map(|s| (
                                                                                           s.line_id,
                                                                                           s.pos.as_ref().and_then(|p| p.x),
                                                                                           s.pos.as_ref().and_then(|p| p.y),
                                                                                       ))
                        else {
                            continue;
                        };

                        let should_report = match old_hp_pct_opt {
                            None => true,
                            Some(old_hp_pct) => old_hp_pct != new_hp_pct && new_hp_pct % 5 == 0,
                        };

                        // Rate limit: only report if hp% changed and hp% is divisible by 5 (e.g. 0%, 5%, etc.)
                        // Always report the first time we see HP data for a monster.
                        if should_report {
                            info!("Found crowdsourced monster with Name {monster_name} - ID {monster_id} - HP% {new_hp_pct}% on line {line} and pos ({pos_x},{pos_y})");
                            let body = serde_json::json!({
                                    "monster_id": monster_id,
                                    "hp_pct": new_hp_pct,
                                    "line": line,
                                    "pos_x": pos_x,
                                    "pos_y": pos_y,
                                });
                            tokio::spawn(async move {
                                let endpoint = endpoint.clone();
                                let client = reqwest::Client::new();
                                let res = client
                                    .post(endpoint)
                                    .header("X-API-Key", CROWD_SOURCE_API_KEY)
                                    .json(&body)
                                    .send().await;
                                match res {
                                    Ok(resp) => {
                                        if resp.status() != reqwest::StatusCode::OK {
                                            error!("POST monster info failed: status {}", resp.status());
                                        }
                                    }
                                    Err(e) => {
                                        error!("Failed to POST monster info: {e}");
                                    }
                                }
                            });
                        }
                    }
                }
            }
            #[allow(clippy::cast_possible_truncation)]
            attr_type::ATTR_MAX_HP => monster_entity.max_hp = Some(prost::encoding::decode_varint(&mut raw_bytes.as_slice()).unwrap() as i32),
            _ => (),
        }
    }
}
