use std::default::Default;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::live::opcodes_models::{attr_type, Encounter, Entity, Skill};
use blueprotobuf_lib::blueprotobuf;
use blueprotobuf_lib::blueprotobuf::{Attr, EEntityType};
use log::info;
use crate::packets::utils::BinaryReader;

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

        let target_entity = encounter.entity_uid_to_entity.entry(target_uid).or_default();
        target_entity.entity_type = target_entity_type;

        match target_entity_type {
            EEntityType::EntChar => {process_player_attrs(target_entity, target_uid, pkt_entity.attrs?.attrs);}
            // EEntityType::EntMonster => {process_monster_attrs();} // todo
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

    let target_entity = encounter.entity_uid_to_entity.entry(player_uid).or_default();
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

    // Process attributes
    let target_entity_type = EEntityType::from(target_uuid);
    let mut target_entity = encounter.entity_uid_to_entity
        .entry(target_uid)
        .or_insert_with(|| Entity {
            entity_type: target_entity_type,
            ..Default::default()
        });

    if let Some(attrs_collection) = aoi_sync_delta.attrs {
        match target_entity_type {
            EEntityType::EntChar => {
                process_player_attrs(&mut target_entity, target_uid, attrs_collection.attrs);
            }
            // EEntityType::EntMonster => { process_monster_attrs(attrs); } // todo
            _ => {}
        }
    }

    let Some(skill_effect) = aoi_sync_delta.skill_effects else {
        return Some(()); // return ok since this variable usually doesn't exist
    };

    // Process Damage
    for sync_damage_info in skill_effect.damages {
        let non_lucky_dmg = sync_damage_info.value;
        let lucky_dmg = sync_damage_info.lucky_value;

        #[allow(clippy::cast_sign_loss)]
        let actual_dmg = if let Some(actual_dmg) = non_lucky_dmg.or(lucky_dmg) {
            actual_dmg as u128
        } else {
            continue; // skip this iteration
        };

        let attacker_uuid = sync_damage_info
            .top_summoner_id
            .or(sync_damage_info.attacker_uuid)?;
        let attacker_uid = attacker_uuid >> 16;
        let attacker_entity = encounter.entity_uid_to_entity
            .entry(attacker_uid)
            .or_insert_with(|| Entity {
                // name: format!("dummy-name-{attacker_uid}"),
                entity_type: EEntityType::from(attacker_uuid),
                ..Default::default()
            });

        // Skills
        let skill_uid = sync_damage_info.owner_id?;
        let skill = attacker_entity.skill_uid_to_skill.entry(skill_uid).or_insert_with(|| Skill {
            ..Default::default()
        });

        // TODO: from testing, first bit is set when there's crit, 3rd bit for if it causes lucky (no idea what that means), require more testing here
        const CRIT_BIT: i32 = 0b00000001;   // 1st bit
        let is_lucky = lucky_dmg.is_some();
        let flag = sync_damage_info.type_flag.unwrap_or_default();
        let is_crit = (flag & CRIT_BIT) != 0; // No idea why but SyncDamageInfo.is_crit isn't correct
        if is_crit {
            skill.crit_hits += 1;
            skill.crit_total_dmg += actual_dmg;
        }
        if is_lucky {
            skill.lucky_hits += 1;
            skill.lucky_total_dmg += actual_dmg;
        }
        skill.hits += 1;
        skill.total_dmg += actual_dmg;
        info!("dmgpacket: {attacker_uid} to {target_uid}: {actual_dmg} dmg {} total dmg", skill.total_dmg);
    }
    // Figure out timestamps
    let timestamp_ms = SystemTime::now().duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();
    if encounter.time_fight_start == Default::default() {
        encounter.time_fight_start = timestamp_ms;
    }
    encounter.time_last_combat_packet = timestamp_ms;
    // info!("buff_effect {:?}", aoi_sync_delta.buff_effect?);
    // info!("buff_infos {:?}", aoi_sync_delta.buff_infos?);
    Some(())
}

fn process_player_attrs(
    player_entity: &mut Entity,
    target_uid: i64,
    attrs: Vec<Attr>)
{
    for attr in attrs {
        let Some(mut raw_bytes) = attr.raw_data else { continue };
        let Some(attr_id) = attr.id else { continue };

        // info!("{} {}", attr_type::(attr_id),hex::encode(raw_bytes.read_remaining()));
        match attr_id {
            attr_type::ATTR_NAME => { // todo: fix these brackets
                player_entity.name = BinaryReader::from(raw_bytes).read_string().unwrap();
                info!{"Found player {} with UID {}", player_entity.name, target_uid} // todo: remove comment
            },
            #[allow(clippy::cast_possible_truncation)]
            attr_type::ATTR_PROFESSION_ID => {
                player_entity.class_id = prost::encoding::decode_varint(&mut raw_bytes.as_slice()).unwrap() as i32;
            },
            #[allow(clippy::cast_possible_truncation)]
            attr_type::ATTR_FIGHT_POINT => {
                player_entity.ability_score = prost::encoding::decode_varint(&mut raw_bytes.as_slice()).unwrap() as i32;
            },
            #[allow(clippy::cast_possible_truncation)]
            attr_type::ATTR_LEVEL => {
                player_entity.level = prost::encoding::decode_varint(&mut raw_bytes.as_slice()).unwrap() as i32;
            },
            _ => ()
        }
    }
}

// fn process_monster_attrs() -> Option<()> {
//     Some(())
// }
