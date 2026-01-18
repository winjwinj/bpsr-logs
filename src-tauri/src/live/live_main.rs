use crate::live::bptimer_state::{BPTimerEnabledMutex, is_bptimer_enabled};
use crate::live::opcodes_models::EncounterMutex;
use crate::live::opcodes_process::{
    on_server_change, process_aoi_sync_delta, process_sync_container_data,
    process_sync_near_entities, process_sync_to_me_delta_info,
};
use crate::live::player_state::{PlayerCacheMutex, PlayerStateMutex};
use crate::packets;
use crate::protocol::pb;
use bytes::Bytes;
use log::warn;
use prost::Message;
use tauri::{AppHandle, Manager};

fn decode_packet<T: Message + Default>(data: Vec<u8>, packet_name: &str) -> Option<T> {
    match T::decode(Bytes::from(data)) {
        Ok(v) => Some(v),
        Err(e) => {
            warn!("Error decoding {packet_name}.. ignoring: {e}");
            None
        }
    }
}

pub async fn start(app_handle: AppHandle) {
    let mut rx = packets::packet_capture::start_capture();

    let bptimer_enabled_state = app_handle.state::<BPTimerEnabledMutex>();

    // 2. Use the channel to receive packets back and process them
    while let Some((op, data)) = rx.recv().await {
        {
            let state = app_handle.state::<EncounterMutex>();
            let encounter = state.lock().unwrap();
            if encounter.is_encounter_paused {
                continue;
            }
        }
        match op {
            packets::opcodes::Pkt::ServerChangeInfo => {
                let encounter_state = app_handle.state::<EncounterMutex>();
                let mut encounter_state = encounter_state.lock().unwrap();
                on_server_change(&mut encounter_state);
            }
            packets::opcodes::Pkt::SyncNearEntities => {
                let Some(sync_near_entities) =
                    decode_packet::<pb::SyncNearEntities>(data, "SyncNearEntities")
                else {
                    continue;
                };
                let player_state_mutex = app_handle.state::<PlayerStateMutex>();
                let player_state = player_state_mutex.lock().unwrap();
                let encounter_state = app_handle.state::<EncounterMutex>();
                let mut encounter_state = encounter_state.lock().unwrap();
                let player_cache_mutex = app_handle.state::<PlayerCacheMutex>();
                if process_sync_near_entities(
                    &mut encounter_state,
                    sync_near_entities,
                    &player_state,
                    is_bptimer_enabled(&bptimer_enabled_state),
                    Some(&player_cache_mutex),
                )
                .is_none()
                {
                    warn!("Error processing SyncNearEntities.. ignoring.");
                }
            }
            packets::opcodes::Pkt::SyncContainerData => {
                let Some(sync_container_data) =
                    decode_packet::<pb::SyncContainerData>(data, "SyncContainerData")
                else {
                    continue;
                };

                // Store persistent player identity data
                let mut should_clear_entities = false;
                if let Some(v_data) = &sync_container_data.v_data {
                    let player_state_mutex = app_handle.state::<PlayerStateMutex>();
                    let mut player_state = player_state_mutex.lock().unwrap();

                    // Extract and store account_id and uid
                    if let Some(char_base) = &v_data.char_base {
                        if !char_base.account_id.is_empty() && v_data.char_id != 0 {
                            player_state
                                .set_account_info(char_base.account_id.clone(), v_data.char_id);
                        }
                    }

                    // Extract and store line_id
                    if let Some(scene_data) = &v_data.scene_data {
                        if scene_data.line_id != 0 {
                            let old_line_id = player_state.get_line_id_opt();
                            player_state.set_line_id(scene_data.line_id);
                            if old_line_id != Some(scene_data.line_id) {
                                should_clear_entities = true;
                            }
                        }
                    }
                }

                let encounter_state = app_handle.state::<EncounterMutex>();
                let mut encounter_state = encounter_state.lock().unwrap();
                if should_clear_entities {
                    encounter_state.entity_uid_to_entity.clear();
                }
                let player_cache_mutex = app_handle.state::<PlayerCacheMutex>();
                encounter_state.local_player = Some(sync_container_data.clone());
                if process_sync_container_data(
                    &mut encounter_state,
                    sync_container_data,
                    Some(&player_cache_mutex),
                )
                .is_none()
                {
                    warn!("Error processing SyncContainerData.. ignoring.");
                }
            }
            // packets::opcodes::Pkt::SyncContainerDirtyData => {
            //     // info!("Received {op:?}");
            //     // trace!("Received {op:?} and data {data:?}");
            //     let sync_container_dirty_data =
            //         match blueprotobuf::SyncContainerDirtyData::decode(Bytes::from(data)) {
            //             Ok(v) => v,
            //             Err(e) => {
            //                 warn!("Error decoding SyncContainerDirtyData.. ignoring: {e}");
            //                 continue;
            //             }
            //         };
            //     let encounter_state = app_handle.state::<EncounterMutex>();
            //     let mut encounter_state = encounter_state.lock().unwrap();
            //     if process_sync_container_dirty_data(&mut encounter_state, sync_container_dirty_data).is_none() {
            //         warn!("Error processing SyncContainerDirtyData.. ignoring.");
            //     }
            // }
            packets::opcodes::Pkt::SyncToMeDeltaInfo => {
                let Some(sync_to_me_delta_info) =
                    decode_packet::<pb::SyncToMeDeltaInfo>(data, "SyncToMeDeltaInfo")
                else {
                    continue;
                };

                let player_state_mutex = app_handle.state::<PlayerStateMutex>();
                let mut player_state = player_state_mutex.lock().unwrap();

                // Update uid if present in delta_info
                if let Some(delta_info) = &sync_to_me_delta_info.delta_info {
                    let uuid = delta_info.uuid;
                    if uuid != 0 {
                        let local_player_uid =
                            crate::protocol::constants::entity::get_player_uid(uuid);
                        let current_uid = player_state.get_uid_opt();
                        if current_uid != Some(local_player_uid) {
                            player_state.set_uid(local_player_uid);
                        }
                    }
                }

                let encounter_state = app_handle.state::<EncounterMutex>();
                let mut encounter_state = encounter_state.lock().unwrap();
                let player_cache_mutex = app_handle.state::<PlayerCacheMutex>();
                if process_sync_to_me_delta_info(
                    &mut encounter_state,
                    sync_to_me_delta_info,
                    &player_state,
                    is_bptimer_enabled(&bptimer_enabled_state),
                    Some(&player_cache_mutex),
                )
                .is_none()
                {
                    warn!("Error processing SyncToMeDeltaInfo.. ignoring.");
                }
            }
            packets::opcodes::Pkt::SyncNearDeltaInfo => {
                let Some(sync_near_delta_info) =
                    decode_packet::<pb::SyncNearDeltaInfo>(data, "SyncNearDeltaInfo")
                else {
                    continue;
                };
                let player_state_mutex = app_handle.state::<PlayerStateMutex>();
                let player_state = player_state_mutex.lock().unwrap();
                let encounter_state = app_handle.state::<EncounterMutex>();
                let mut encounter_state = encounter_state.lock().unwrap();
                let player_cache_mutex = app_handle.state::<PlayerCacheMutex>();
                for aoi_sync_delta in sync_near_delta_info.delta_infos {
                    if process_aoi_sync_delta(
                        &mut encounter_state,
                        aoi_sync_delta,
                        &player_state,
                        is_bptimer_enabled(&bptimer_enabled_state),
                        Some(&player_cache_mutex),
                    )
                    .is_none()
                    {
                        warn!("Error processing SyncNearDeltaInfo.. ignoring.");
                    }
                }
            }
        }
    }
}
