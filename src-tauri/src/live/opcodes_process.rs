use crate::live::bptimer::BPTimerClient;
use crate::live::opcodes_models::class::{
    Class, ClassSpec, get_class_from_spec, get_class_spec_from_skill_id,
};
use crate::live::opcodes_models::{CombatStats, Encounter, Entity, MONSTER_NAMES_BOSS, attr_type};
use crate::live::player_state::{PlayerCacheMutex, PlayerState};
use crate::packets::utils::BinaryReader;
use blueprotobuf_lib::blueprotobuf;
use bytes::Bytes;
use log::{debug, info, warn};
use prost::Message;
use std::io::Cursor;
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

// Needed for Github Actions compile-time env vars
const COMPILE_TIME_ENDPOINT: Option<&str> = option_env!("BP_TIMER_ENDPOINT");
const COMPILE_TIME_API_KEY: Option<&str> = option_env!("BP_TIMER_API_KEY");

// Checks runtime env vars first, then falls back to compile-time env vars
static BP_TIMER_CLIENT: LazyLock<Option<BPTimerClient>> = LazyLock::new(|| {
    let endpoint = std::env::var("BP_TIMER_ENDPOINT")
        .ok()
        .or_else(|| COMPILE_TIME_ENDPOINT.map(String::from));

    let api_key = std::env::var("BP_TIMER_API_KEY")
        .ok()
        .or_else(|| COMPILE_TIME_API_KEY.map(String::from));

    match (endpoint, api_key) {
        (Some(endpoint), Some(api_key)) => {
            info!("BPTimer Client enabled: {}", endpoint);
            let client = BPTimerClient::new(endpoint, api_key);
            client.prefetch_mobs();
            Some(client)
        }
        _ => {
            warn!(
                "BPTimer Client disabled: missing ENV vars (set BP_TIMER_ENDPOINT and BP_TIMER_API_KEY)"
            );
            None
        }
    }
});

pub fn on_server_change(encounter: &mut Encounter) {
    info!("on server change");
    encounter.clone_from(&Encounter::default());
}

pub fn process_sync_near_entities(
    encounter: &mut Encounter,
    sync_near_entities: blueprotobuf::SyncNearEntities,
    player_state: &PlayerState,
    is_bptimer_enabled: bool,
    player_cache: Option<&PlayerCacheMutex>,
) -> Option<()> {
    for pkt_entity in sync_near_entities.appear {
        let target_uuid = pkt_entity.uuid;
        if target_uuid == 0 {
            continue;
        }
        let target_uid = target_uuid >> 16;
        let target_entity_type = blueprotobuf::EEntityType::from(target_uuid);

        let target_entity = encounter
            .entity_uid_to_entity
            .entry(target_uid)
            .or_default();
        target_entity.entity_type = target_entity_type;

        if let Some(attrs) = &pkt_entity.attrs {
            match target_entity_type {
                blueprotobuf::EEntityType::EntChar => process_player_attrs(
                    target_entity,
                    target_uid,
                    attrs.attrs.clone(),
                    player_cache,
                ),
                blueprotobuf::EEntityType::EntMonster => process_monster_attrs(
                    target_entity,
                    attrs.attrs.clone(),
                    player_state,
                    is_bptimer_enabled,
                ),
                _ => {}
            }
        }
    }
    Some(())
}

pub fn process_sync_container_data(
    encounter: &mut Encounter,
    sync_container_data: blueprotobuf::SyncContainerData,
    player_cache: Option<&PlayerCacheMutex>,
) -> Option<()> {
    let Some(v_data) = &sync_container_data.v_data else {
        return None;
    };

    let player_uid = v_data.char_id;
    if player_uid == 0 {
        return None;
    }

    let target_entity = encounter
        .entity_uid_to_entity
        .entry(player_uid)
        .or_default();
    target_entity.entity_type = blueprotobuf::EEntityType::EntChar;

    if let Some(char_base) = &v_data.char_base {
        if !char_base.name.is_empty() {
            target_entity.name = Some(char_base.name.clone());
        }
        if char_base.fight_point != 0 {
            target_entity.ability_score = Some(char_base.fight_point);
        }
    }

    if let Some(profession_list) = &v_data.profession_list {
        if profession_list.cur_profession_id != 0 {
            let player_class = Class::from(profession_list.cur_profession_id);
            target_entity.class = Some(player_class);

            if let Some(cache) = player_cache {
                if let Ok(mut cache) = cache.lock() {
                    if let Some(name) = &target_entity.name {
                        cache.set_both(player_uid, Some(name.clone()), Some(player_class));
                    }
                    if let Some(ability_score) = target_entity.ability_score {
                        cache.set_ability_score(player_uid, ability_score);
                    }
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
    player_state: &PlayerState,
    is_bptimer_enabled: bool,
    player_cache: Option<&PlayerCacheMutex>,
) -> Option<()> {
    let Some(delta_info) = &sync_to_me_delta_info.delta_info else {
        return None;
    };
    let Some(base_delta) = &delta_info.base_delta else {
        return None;
    };
    process_aoi_sync_delta(
        encounter,
        base_delta.clone(),
        player_state,
        is_bptimer_enabled,
        player_cache,
    )
}

pub fn process_aoi_sync_delta(
    encounter: &mut Encounter,
    aoi_sync_delta: blueprotobuf::AoiSyncDelta,
    player_state: &PlayerState,
    is_bptimer_enabled: bool,
    player_cache: Option<&PlayerCacheMutex>,
) -> Option<()> {
    let target_uuid = aoi_sync_delta.uuid;
    if target_uuid == 0 {
        return None;
    }
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
                blueprotobuf::EEntityType::EntChar => process_player_attrs(
                    target_entity,
                    target_uid,
                    attrs_collection.attrs,
                    player_cache,
                ),
                blueprotobuf::EEntityType::EntMonster => process_monster_attrs(
                    target_entity,
                    attrs_collection.attrs,
                    player_state,
                    is_bptimer_enabled,
                ),
                _ => {}
            }
        }
    }

    let Some(skill_effect) = aoi_sync_delta.skill_effects else {
        return Some(()); // return ok since this variable usually doesn't exist
    };

    // Process Damage
    for sync_damage_info in skill_effect.damages {
        let is_boss = encounter
            .entity_uid_to_entity
            .get(&target_uid)
            .and_then(|e| e.monster_id)
            .is_some_and(|id| MONSTER_NAMES_BOSS.contains_key(&id));

        let attacker_uuid = if sync_damage_info.top_summoner_id != 0 {
            sync_damage_info.top_summoner_id
        } else if sync_damage_info.attacker_uuid != 0 {
            sync_damage_info.attacker_uuid
        } else {
            continue; // Skip this damage packet if no attacker
        };
        let attacker_uid = attacker_uuid >> 16;
        let attacker_entity = encounter
            .entity_uid_to_entity
            .entry(attacker_uid)
            .or_insert_with(|| Entity {
                entity_type: blueprotobuf::EEntityType::from(attacker_uuid),
                ..Default::default()
            });

        let skill_uid = sync_damage_info.owner_id;
        if skill_uid == 0 {
            continue; // Skip this damage packet if no skill_uid
        }
        if attacker_entity
            .class_spec
            .is_none_or(|class_spec| class_spec == ClassSpec::Unknown)
        {
            let class_spec = get_class_spec_from_skill_id(skill_uid);
            attacker_entity.class_spec = Some(class_spec);

            // Only infer/overwrite class if it's not already set or is Unknown/Unimplemented
            let should_cache_class = if attacker_entity
                .class
                .is_none_or(|class| matches!(class, Class::Unknown | Class::Unimplemented))
            {
                let inferred_class = get_class_from_spec(class_spec);
                attacker_entity.class = Some(inferred_class);
                Some(inferred_class)
            } else {
                None
            };

            // Cache the inferred class and class_spec (only for players)
            if blueprotobuf::EEntityType::from(attacker_uuid) == blueprotobuf::EEntityType::EntChar
            {
                if let Some(cache) = player_cache {
                    if let Ok(mut cache) = cache.lock() {
                        if let Some(inferred_class) = should_cache_class {
                            cache.set_class(attacker_uid, inferred_class);
                        }
                        cache.set_class_spec(attacker_uid, class_spec);
                    }
                }
            }
        }

        // Skills
        let is_heal = sync_damage_info.r#type == blueprotobuf::EDamageType::Heal as i32;
        if is_heal {
            let heal_skill = attacker_entity
                .skill_uid_to_heal_stats
                .entry(skill_uid)
                .or_default();
            process_stats(&sync_damage_info, heal_skill);
            process_stats(&sync_damage_info, &mut attacker_entity.heal_stats); // update total entity heal stats
            process_stats(&sync_damage_info, &mut encounter.heal_stats); // update total encounter heal stats
        } else {
            let dps_skill = attacker_entity
                .skill_uid_to_dps_stats
                .entry(skill_uid)
                .or_default();
            process_stats(&sync_damage_info, dps_skill);
            process_stats(&sync_damage_info, &mut attacker_entity.dmg_stats); // update total entity dmg stats
            process_stats(&sync_damage_info, &mut encounter.dmg_stats); // update total encounter dmg stats
            if is_boss {
                let skill_boss_only = attacker_entity
                    .skill_uid_to_dps_stats_boss_only
                    .entry(skill_uid)
                    .or_default();
                process_stats(&sync_damage_info, skill_boss_only);
                process_stats(&sync_damage_info, &mut attacker_entity.dmg_stats_boss_only); // update total entity boss only dmg stats
                process_stats(&sync_damage_info, &mut encounter.dmg_stats_boss_only); // update total encounter dmg stats
            }
        }
    }

    // Figure out timestamps
    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();
    if encounter.time_fight_start_ms == 0 {
        encounter.time_fight_start_ms = timestamp_ms;
    }
    encounter.time_last_combat_packet_ms = timestamp_ms;
    Some(())
}

fn process_stats(sync_damage_info: &blueprotobuf::SyncDamageInfo, stats: &mut CombatStats) {
    // TODO: from testing, first bit is set when there's crit, 3rd bit for if it causes lucky (no idea what that means), require more testing here
    const CRIT_BIT: i32 = 0b00_00_00_01; // 1st bit

    // Prefer lucky damage value if available (non-zero), otherwise use regular value
    let actual_value = if sync_damage_info.lucky_value != 0 {
        sync_damage_info.lucky_value
    } else {
        sync_damage_info.value
    };

    let is_lucky = sync_damage_info.lucky_value != 0;
    let flag = sync_damage_info.type_flag;
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

fn decode_protobuf_int32(data: &[u8]) -> Result<i32, Box<dyn std::error::Error>> {
    if data.is_empty() {
        return Err("Empty data".into());
    }
    let mut cursor = Cursor::new(data);
    prost::encoding::decode_varint(&mut cursor)
        .map(|v| v as i32)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

fn decode_protobuf_int64(data: &[u8]) -> Result<i64, Box<dyn std::error::Error>> {
    if data.is_empty() {
        return Err("Empty data".into());
    }
    let mut cursor = Cursor::new(data);
    prost::encoding::decode_varint(&mut cursor)
        .map(|v| v as i64)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

fn process_player_attrs(
    player_entity: &mut Entity,
    player_uid: i64,
    attrs: Vec<blueprotobuf::Attr>,
    player_cache: Option<&PlayerCacheMutex>,
) {
    // Restore from cache if not already set
    if let Some(cache) = player_cache {
        if let Ok(cache) = cache.lock() {
            if let Some(cached_entry) = cache.get(player_uid) {
                if player_entity.name.is_none() {
                    player_entity.name = cached_entry.name.clone();
                }
                if player_entity.class.is_none() {
                    player_entity.class = cached_entry.class;
                }
                if player_entity.class_spec.is_none() {
                    player_entity.class_spec = cached_entry.class_spec;
                }
            }
        }
    }

    for attr in attrs {
        if attr.raw_data.is_empty() {
            continue;
        }
        if attr.id == 0 {
            continue;
        }

        // info!("{} {}", attr_type::(attr_id),hex::encode(raw_bytes.read_remaining()));
        match attr.id {
            attr_type::ATTR_NAME => {
                let mut raw_bytes = attr.raw_data;
                raw_bytes.remove(0); // not sure why, there's some weird character as the first e.g. "\u{6}Sketal"
                let player_name_result = BinaryReader::from(raw_bytes).read_string();
                if let Ok(player_name) = player_name_result {
                    player_entity.name = Some(player_name.clone());
                    debug!("Found player {player_name} with UID {player_uid}");
                    if let Some(cache) = player_cache {
                        if let Ok(mut cache) = cache.lock() {
                            cache.set_name(player_uid, player_name);
                        }
                    }
                } else {
                    warn!("Failed to read player name for UID {player_uid}");
                }
            }
            attr_type::ATTR_PROFESSION_ID => {
                if let Ok(class_id) = decode_protobuf_int32(&attr.raw_data) {
                    let player_class = Class::from(class_id);
                    player_entity.class = Some(player_class);

                    // Cache the class
                    if let Some(cache) = player_cache {
                        if let Ok(mut cache) = cache.lock() {
                            cache.set_class(player_uid, player_class);
                        }
                    }
                }
            }
            attr_type::ATTR_FIGHT_POINT => {
                if let Ok(ability_score) = decode_protobuf_int32(&attr.raw_data) {
                    player_entity.ability_score = Some(ability_score);

                    if let Some(cache) = player_cache {
                        if let Ok(mut cache) = cache.lock() {
                            cache.set_ability_score(player_uid, ability_score);
                        }
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
    player_state: &PlayerState,
    is_bptimer_enabled: bool,
) {
    // Track if HP was updated during this attribute batch
    let mut hp_updated = false;

    // Process all attributes and update entity state
    for attr in attrs {
        if attr.raw_data.is_empty() {
            continue;
        }
        if attr.id == 0 {
            continue;
        }

        match attr.id {
            attr_type::ATTR_ID => {
                if let Ok(id) = decode_protobuf_int32(&attr.raw_data) {
                    if id >= 0 {
                        monster_entity.monster_id = Some(id as u32);
                    }
                }
            }
            attr_type::ATTR_HP => {
                if let Ok(curr_hp) = decode_protobuf_int64(&attr.raw_data) {
                    if curr_hp >= 0 {
                        monster_entity.curr_hp = Some(curr_hp as u64);
                        hp_updated = true;
                    }
                }
            }
            attr_type::ATTR_MAX_HP => {
                if let Ok(max_hp) = decode_protobuf_int64(&attr.raw_data) {
                    if max_hp >= 0 {
                        monster_entity.max_hp = Some(max_hp as u64);
                    }
                }
            }
            attr_type::ATTR_POS => {
                if let Ok(pos) =
                    blueprotobuf::Vector3::decode(Bytes::copy_from_slice(&attr.raw_data))
                {
                    monster_entity.monster_pos = pos;
                }
            }
            _ => (),
        }
    }

    // Report to bptimer if HP was updated and both current_hp and max_hp are available
    // bptimer client handles all validation internally
    if hp_updated
        && monster_entity.curr_hp.is_some()
        && monster_entity.max_hp.is_some()
        && is_bptimer_enabled
    {
        if let Some(client) = BP_TIMER_CLIENT.as_ref() {
            let line = player_state.get_line_id_opt().and_then(|id| {
                if id <= i32::MAX as u32 {
                    Some(id as i32)
                } else {
                    None
                }
            });
            let account_id = player_state.get_account_id();
            let uid = player_state.get_uid_opt();

            client.report_hp(
                monster_entity.monster_id,
                monster_entity.curr_hp,
                monster_entity.max_hp,
                line,
                Some(monster_entity.monster_pos.x),
                Some(monster_entity.monster_pos.y),
                Some(monster_entity.monster_pos.z),
                account_id,
                uid,
            );
        }
    }
}
