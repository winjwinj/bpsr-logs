use crate::live::opcodes_models::class::{get_class_from_spec, get_class_spec_from_skill_id, Class, ClassSpec};
use crate::live::opcodes_models::{attr_type, CombatStats, Encounter, Entity, MONSTER_NAMES, MONSTER_NAMES_BOSS, MONSTER_NAMES_CROWDSOURCE};
use crate::packets::utils::BinaryReader;
use blueprotobuf_lib::blueprotobuf;
use log::{error, info, warn, debug};
use std::default::Default;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::db::DbConnection;

/// Helper function to load historical class_spec from database for a player
fn load_historical_class_spec_for_player(
    entity: &mut Entity,
    _player_uid: i64,
    _db: &DbConnection,
) {
    // Skip if already has a valid class_spec
    if entity.class_spec.is_some() && entity.class_spec != Some(ClassSpec::Unknown) {
        return;
    }

}

pub fn on_server_change(encounter: &mut Encounter) {
    info!("on server change - preserving local_player data");
    let local_player = encounter.local_player.clone(); // Preserve local player data across zone changes
    encounter.clone_from(&Encounter::default());
    encounter.local_player = local_player; // Restore it after reset
}

/// Set an entity's name conservatively: only apply the incoming name when it is
/// non-empty and not a placeholder ("Unknown", "Unknown Name"). Do not overwrite an existing
/// useful name. Log prior and new values for visibility when changes occur.
fn set_entity_name(entity: &mut Entity, uid: i64, incoming_name: &str, _db: &crate::db::DbConnection) {
    if !crate::db::is_valid_player_name(incoming_name) {
        info!("Skipping invalid incoming name for UID {uid}: '{incoming_name}'");
        return;
    }

    match &entity.name {
        Some(existing) if crate::db::is_valid_player_name(existing) => {
            if existing != incoming_name {
                info!("Keeping existing name for UID {uid}: '{existing}' (incoming: '{incoming_name}')");
            }
        }
        _ => {
            let prev = entity.name.clone().unwrap_or_else(|| String::from("<none>"));
            entity.name = Some(incoming_name.to_string());
            info!("Set name for UID {uid}: '{prev}' -> '{incoming_name}'")
        }
    }
}

pub fn process_sync_near_entities(
    encounter: &mut Encounter,
    sync_near_entities: blueprotobuf::SyncNearEntities,
    is_bptimer_enabled: bool,
    db: &DbConnection,
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
            blueprotobuf::EEntityType::EntChar => {
                process_player_attrs(target_entity, target_uid, pkt_entity.attrs?.attrs, db);
                load_historical_class_spec_for_player(target_entity, target_uid as i64, db);
            },
            blueprotobuf::EEntityType::EntMonster => process_monster_attrs(target_entity, pkt_entity.attrs?.attrs, encounter.local_player.as_ref(), is_bptimer_enabled),
            _ => {}
        }
    }
    Some(())
}

pub fn process_sync_container_data(
    encounter: &mut Encounter,
    sync_container_data: blueprotobuf::SyncContainerData,
    db: &DbConnection,
) -> Option<()> {
    let v_data = sync_container_data.v_data?;
    let player_uid = v_data.char_id?;

    let target_entity = encounter
        .entity_uid_to_entity
        .entry(player_uid)
        .or_default();
    let char_base = v_data.char_base?;
    // Only set name if it's meaningful
    if let Ok(name) = std::panic::catch_unwind(|| char_base.name.clone()) {
        if let Some(name_str) = name {
            set_entity_name(target_entity, player_uid, &name_str, db);
        }
    }
    target_entity.entity_type = blueprotobuf::EEntityType::EntChar;

    // Profession id may be absent; only set class if it's meaningful and doesn't overwrite a better one
    if let Some(prof_list) = &v_data.profession_list {
        if let Some(prof_id) = prof_list.cur_profession_id {
            let new_class = Class::from(prof_id);
            if !matches!(new_class, Class::Unimplemented | Class::Unknown) {
                let should_set = match target_entity.class {
                    None => true,
                    Some(existing) => matches!(existing, Class::Unimplemented | Class::Unknown),
                };
                if should_set {
                    target_entity.class = Some(new_class);
                }
            }
        }
    }

    // Ability score from SyncContainerData
    if let Ok(fp) = std::panic::catch_unwind(|| char_base.fight_point) {
        if let Some(fp_val) = fp {
            if fp_val > 0 {
                let should_set = match target_entity.ability_score {
                    None => true,
                    Some(existing) => existing <= 0,
                };
                if should_set {
                    target_entity.ability_score = Some(fp_val);
                }
            }
        }
    }

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
    db: &DbConnection,
) -> Option<()> {
    let delta_info = sync_to_me_delta_info.delta_info.as_ref()?;
    let uuid = delta_info.uuid?;
    encounter.local_player_uid = Some(uuid >> 16); // UUID =/= uid (have to >> 16)
    
    let base_delta = delta_info.base_delta.as_ref()?;
    process_aoi_sync_delta(encounter, base_delta.clone(), is_bptimer_enabled, db);
    Some(())
}

pub fn process_aoi_sync_delta(
    encounter: &mut Encounter,
    aoi_sync_delta: blueprotobuf::AoiSyncDelta,
    is_bptimer_enabled: bool,
    db: &DbConnection,
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
            let attr_count = attrs_collection.attrs.len();
            if attr_count > 0 {
                debug!("Processing {} attributes for entity {} (type: {:?})", attr_count, target_uid, target_entity_type);
            }
            match target_entity_type {
                blueprotobuf::EEntityType::EntChar => process_player_attrs(target_entity, target_uid, attrs_collection.attrs, db),
                blueprotobuf::EEntityType::EntMonster => process_monster_attrs(target_entity, attrs_collection.attrs, encounter.local_player.as_ref(), is_bptimer_enabled),
                _ => {}
            }
        } else {
            debug!("No attributes in delta for entity {} (type: {:?})", target_uid, target_entity_type);
        }
    }

    let Some(skill_effect) = aoi_sync_delta.skill_effects else {
        return Some(()); // return ok since this variable usually doesn't exist
    };

    // Process Damage
    for sync_damage_info in skill_effect.damages {
        let is_boss = encounter.entity_uid_to_entity
                               .get(&target_uid)
                               .and_then(|e| e.monster_id)
                               .is_some_and(|id| MONSTER_NAMES_BOSS.contains_key(&id));
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
            // First try to load from database
            load_historical_class_spec_for_player(attacker_entity, attacker_uid as i64, db);
            
            // If still no class_spec, try to infer from the skill being used
            if attacker_entity.class_spec.is_none_or(|class_spec| class_spec == ClassSpec::Unknown) {
                let class_spec = get_class_spec_from_skill_id(skill_uid);
                attacker_entity.class = Some(get_class_from_spec(class_spec));
                attacker_entity.class_spec = Some(class_spec);
            }
        }

        // Skills
        let is_heal = sync_damage_info.r#type.unwrap_or(0) == blueprotobuf::EDamageType::Heal as i32;
        if is_heal {
            // Record heal stats in the heal-specific map (was incorrectly using the dps map)
            let heal_skill = attacker_entity
                .skill_uid_to_heal_stats
                .entry(skill_uid)
                .or_default();
            process_stats(&sync_damage_info, heal_skill);
            process_stats(&sync_damage_info, &mut attacker_entity.heal_stats); // update total entity heal stats
            process_stats(&sync_damage_info, &mut encounter.heal_stats); // update total encounter heal stats
            info!("heal packet: {attacker_uid} to {target_uid}: {} total heal", heal_skill.value);
        } else {
            let dps_skill = attacker_entity
                .skill_uid_to_dps_stats
                .entry(skill_uid)
                .or_default();
            process_stats(&sync_damage_info, dps_skill);
            process_stats(&sync_damage_info, &mut attacker_entity.dmg_stats); // update total entity dmg stats
            process_stats(&sync_damage_info, &mut encounter.dmg_stats); // update total encounter heal stats
            if is_boss {
                let skill_boss_only = attacker_entity
                    .skill_uid_to_dps_stats_boss_only
                    .entry(skill_uid)
                    .or_default();
                process_stats(&sync_damage_info, skill_boss_only);
                process_stats(&sync_damage_info, &mut attacker_entity.dmg_stats_boss_only); // update total entity boss only dmg stats
                process_stats(&sync_damage_info, &mut encounter.dmg_stats_boss_only); // update total encounter heal stats
            }
            info!("dmg packet: {attacker_uid} to {target_uid}: {} total dmg", dps_skill.value);
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

fn process_player_attrs(player_entity: &mut Entity, player_uid: i64, attrs: Vec<blueprotobuf::Attr>, db: &crate::db::DbConnection) {
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
                    // Only set the name if it's useful. Avoid overwriting a previously-known
                    // name with an empty or placeholder value (e.g. "Unknown", "Unknown Name"). 
                    // This prevents later packets from degrading previously-captured player names.
                    if crate::db::is_valid_player_name(&player_name) {
                        set_entity_name(player_entity, player_uid, &player_name, db);
                        info!("Found player {player_name} with UID {player_uid}");
                    } else {
                        info!("Skipping invalid name for UID {player_uid}: '{player_name}'");
                    }
                } else {
                    warn!("Failed to read player name for UID {player_uid}");
                }
            }
            #[allow(clippy::cast_possible_truncation)]
            attr_type::ATTR_PROFESSION_ID => {
                let prof_id = prost::encoding::decode_varint(&mut raw_bytes.as_slice()).unwrap() as i32;
                let new_class = Class::from(prof_id);
                // Only set class if the new value is meaningful and we don't already have
                // a better class recorded. Class::Unimplemented (and Unknown) are not
                // considered useful values to overwrite an existing one.
                if !matches!(new_class, Class::Unimplemented | Class::Unknown) {
                    let should_set = match player_entity.class {
                        None => true,
                        Some(existing) => matches!(existing, Class::Unimplemented | Class::Unknown),
                    };
                    if should_set {
                        player_entity.class = Some(new_class);
                    }
                }
            }
            #[allow(clippy::cast_possible_truncation)]
            attr_type::ATTR_FIGHT_POINT => {
                let fp = prost::encoding::decode_varint(&mut raw_bytes.as_slice()).unwrap() as i32;
                // Only set ability_score if it's positive and we don't already have a positive value.
                if fp > 0 {
                    let should_set = match player_entity.ability_score {
                        None => true,
                        Some(existing) => existing <= 0,
                    };
                    if should_set {
                        player_entity.ability_score = Some(fp);
                    }
                }
            }
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
                let prev_hp = monster_entity.curr_hp.unwrap_or(curr_hp); // If previous hp doesn't exist, just use the current hp
                monster_entity.curr_hp = Some(curr_hp);

                if is_bptimer_enabled {
                    // Crowdsource Data: if people abuse this, we will change the security
                    // const ENDPOINT: &str = "http://localhost:3000";
                    const ENDPOINT: &str = "https://db.bptimer.com/api/create-hp-report";
                    const API_KEY: &str = "8fibznvjgf9vh29bg7g730fan9xaskf7h45lzdl2891vi0w1d2";
                    let (Some(monster_id), Some(local_player)) = (monster_entity.monster_id, &local_player) else {
                        continue;
                    };
                    let Some(max_hp) = monster_entity.max_hp else {
                        continue;
                    };
                    if MONSTER_NAMES_CROWDSOURCE.contains_key(&monster_id) { // only record if it's a world boss, magical creature, etc.
                        let monster_name = MONSTER_NAMES.get(&monster_id).map_or("Unknown Monster Name", |s| s.as_str());
                        let old_hp_pct = (prev_hp * 100 / max_hp).clamp(0, 100);
                        let new_hp_pct = (curr_hp * 100 / max_hp).clamp(0, 100);
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

                        // Rate limit: only report if hp% changed and hp% is divisible by 5 (e.g. 0%, 5%, etc.)
                        if old_hp_pct != new_hp_pct && new_hp_pct % 5 == 0 {
                            info!("Found crowdsourced monster with Name {monster_name} - ID {monster_id} - HP% {new_hp_pct}% on line {line} and pos ({pos_x},{pos_y})");
                            let body = serde_json::json!({
                                    "monster_id": monster_id,
                                    "hp_pct": new_hp_pct,
                                    "line": line,
                                    "pos_x": pos_x,
                                    "pos_y": pos_y,
                                });
                            tokio::spawn(async move {
                                let client = reqwest::Client::new();
                                let res = client
                                    .post(ENDPOINT)
                                    .header("X-API-Key", API_KEY)
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
