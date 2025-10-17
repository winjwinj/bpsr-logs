use crate::live::opcodes_models::EncounterMutex;
use crate::live::opcodes_process::{
    on_server_change, process_aoi_sync_delta, process_sync_container_data,
    process_sync_container_dirty_data, process_sync_near_entities, process_sync_to_me_delta_info,
};
use crate::packets;
use blueprotobuf_lib::blueprotobuf;
use bytes::Bytes;
use log::{error, info, trace, warn};
use prost::Message;
use tauri::{AppHandle, Manager};

pub async fn start(app_handle: AppHandle) {
    // todo: add app_handle?
    // https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html
    // 1. Start capturing packets and send to rx
    let mut rx = packets::packet_capture::start_capture(); // Since live meter is not critical, it's ok to just log it // TODO: maybe bubble an error up to the frontend instead?

    // 2. Use the channel to receive packets back and process them
    while let Some((op, data)) = rx.recv().await {
        {
            let state = app_handle.state::<EncounterMutex>();
            let encounter = state.lock().unwrap();
            if encounter.is_encounter_paused {
                info!("packet dropped due to encounter paused");
                continue;
            }
        }
        // error!("Received Pkt {op:?}");
        match op {
            packets::opcodes::Pkt::ServerChangeInfo => {
                let encounter_state = app_handle.state::<EncounterMutex>();
                let mut encounter_state = encounter_state.lock().unwrap();
                on_server_change(&mut encounter_state);
            }
            packets::opcodes::Pkt::SyncNearEntities => {
                // info!("Received {op:?}");
                // info!("Received {op:?} and data {data:?}");
                // trace!("Received {op:?} and data {data:?}");
                let sync_near_entities =
                    match blueprotobuf::SyncNearEntities::decode(Bytes::from(data)) {
                        Ok(v) => v,
                        Err(e) => {
                            warn!("Error decoding SyncNearEntities.. ignoring: {e}");
                            continue;
                        }
                    };
                let encounter_state = app_handle.state::<EncounterMutex>();
                let mut encounter_state = encounter_state.lock().unwrap();
                if process_sync_near_entities(&mut encounter_state, sync_near_entities).is_none() {
                    warn!("Error processing SyncNearEntities.. ignoring.");
                }
            }
            packets::opcodes::Pkt::SyncContainerData => {
                // info!("Received {op:?}");
                // info!("Received {op:?} and data {data:?}");
                // trace!("Received {op:?} and data {data:?}");
                let sync_container_data =
                    match blueprotobuf::SyncContainerData::decode(Bytes::from(data)) {
                        Ok(v) => v,
                        Err(e) => {
                            warn!("Error decoding SyncContainerData.. ignoring: {e}");
                            continue;
                        }
                    };
                let encounter_state = app_handle.state::<EncounterMutex>();
                let mut encounter_state = encounter_state.lock().unwrap();
                encounter_state.local_player = sync_container_data.clone();
                if process_sync_container_data(&mut encounter_state, sync_container_data).is_none()
                {
                    warn!("Error processing SyncContainerData.. ignoring.");
                }
            }
            packets::opcodes::Pkt::SyncContainerDirtyData => {
                // info!("Received {op:?}");
                // trace!("Received {op:?} and data {data:?}");
                let sync_container_dirty_data =
                    match blueprotobuf::SyncContainerDirtyData::decode(Bytes::from(data)) {
                        Ok(v) => v,
                        Err(e) => {
                            warn!("Error decoding SyncContainerDirtyData.. ignoring: {e}");
                            continue;
                        }
                    };
                let encounter_state = app_handle.state::<EncounterMutex>();
                let mut encounter_state = encounter_state.lock().unwrap();
                if process_sync_container_dirty_data(
                    &mut encounter_state,
                    sync_container_dirty_data,
                )
                .is_none()
                {
                    warn!("Error processing SyncToMeDeltaInfo.. ignoring.");
                }
            }
            packets::opcodes::Pkt::SyncServerTime => {
                // info!("Received {op:?}");
                // trace!("Received {op:?} and data {data:?}");
                let _sync_server_time =
                    match blueprotobuf::SyncServerTime::decode(Bytes::from(data)) {
                        Ok(v) => v,
                        Err(e) => {
                            warn!("Error decoding SyncServerTime.. ignoring: {e}");
                            continue;
                        }
                    };
                // todo: this is skipped, not sure what info it has
            }
            packets::opcodes::Pkt::SyncToMeDeltaInfo => {
                // todo: fix this, attrs dont include name, no idea why
                // trace!("Received {op:?}");
                // info!("Received {op:?} and data {data:?}");
                let sync_to_me_delta_info =
                    match blueprotobuf::SyncToMeDeltaInfo::decode(Bytes::from(data)) {
                        Ok(sync_to_me_delta_info) => sync_to_me_delta_info,
                        Err(e) => {
                            warn!("Error decoding SyncToMeDeltaInfo.. ignoring: {e}");
                            continue;
                        }
                    };
                let encounter_state = app_handle.state::<EncounterMutex>();
                let mut encounter_state = encounter_state.lock().unwrap();
                if process_sync_to_me_delta_info(&mut encounter_state, sync_to_me_delta_info)
                    .is_none()
                {
                    warn!("Error processing SyncToMeDeltaInfo.. ignoring.");
                }
            }
            packets::opcodes::Pkt::SyncNearDeltaInfo => {
                // trace!("Received {op:?}");
                // info!("Received {op:?} and data {data:?}");
                let sync_near_delta_info =
                    match blueprotobuf::SyncNearDeltaInfo::decode(Bytes::from(data)) {
                        Ok(v) => v,
                        Err(e) => {
                            warn!("Error decoding SyncNearDeltaInfo.. ignoring: {e}");
                            continue;
                        }
                    };
                let encounter_state = app_handle.state::<EncounterMutex>();
                let mut encounter_state = encounter_state.lock().unwrap();
                for aoi_sync_delta in sync_near_delta_info.delta_infos {
                    if process_aoi_sync_delta(&mut encounter_state, aoi_sync_delta).is_none() {
                        warn!("Error processing SyncToMeDeltaInfo.. ignoring.");
                    }
                }
            }
        }
    }
}
