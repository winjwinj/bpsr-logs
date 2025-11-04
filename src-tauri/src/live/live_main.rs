use crate::live::opcodes_models::EncounterMutex;
use crate::live::opcodes_process::{
    on_server_change, process_aoi_sync_delta, process_sync_container_data,
    process_sync_near_entities, process_sync_to_me_delta_info,
};
use crate::packets;
use blueprotobuf_lib::blueprotobuf;
use bytes::Bytes;
use log::{info, warn, debug};
use prost::Message;
use tauri::{AppHandle, Manager};
use tauri_plugin_svelte::ManagerExt;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Periodically refresh metadata for unknown players during combat
/// This runs every 10 seconds and checks the database for names
async fn refresh_unknown_player_metadata(
    app_handle: Arc<AppHandle>,
    should_continue: Arc<Mutex<bool>>,
) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
    
    loop {
        interval.tick().await;
        
        // Check if we should continue
        let continue_running = *should_continue.lock().await;
        if !continue_running {
            break;
        }
        
        // Get list of unknown players from encounter
        let unknown_players = {
            let encounter_state = app_handle.state::<EncounterMutex>();
            let encounter = encounter_state.lock().unwrap();
            
            // Skip if not in active combat
            if encounter.dmg_stats.value == 0 && encounter.heal_stats.value == 0 {
                continue;
            }
            
            // Collect UIDs for players with unknown/empty names
            encounter
                .entity_uid_to_entity
                .iter()
                .filter_map(|(uid, entity)| {
                    let has_invalid_name = match &entity.name {
                        Some(n) => !crate::db::is_valid_player_name(n),
                        None => true,
                    };
                    
                    if has_invalid_name && entity.dmg_stats.value > 0 {
                        Some(*uid)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        };
        
        if unknown_players.is_empty() {
            continue;
        }
        
        debug!("Refreshing metadata for {} unknown players", unknown_players.len());
        
        // Look up metadata for unknown players
        let db = app_handle.state::<crate::db::DbConnection>();
        for uid in unknown_players {
            if let Ok(Some(metadata)) = crate::db::lookup_player_metadata(&db, uid).await {
                // Update the entity with the discovered metadata
                let encounter_state = app_handle.state::<EncounterMutex>();
                let mut encounter = encounter_state.lock().unwrap();
                
                if let Some(entity) = encounter.entity_uid_to_entity.get_mut(&uid) {
                    // Update name if valid and current name is invalid
                    if crate::db::is_valid_player_name(&metadata.name) {
                        if !entity.name.as_ref().map_or(false, |n| crate::db::is_valid_player_name(n)) {
                            info!("Updated player {} name to: {}", uid, metadata.name);
                            entity.name = Some(metadata.name);
                        }
                    }
                    
                    // Update class if current is missing
                    if entity.class.is_none() && metadata.class.is_some() {
                        if let Some(class_str) = metadata.class {
                            let class_enum = match class_str.as_str() {
                                "Stormblade" => crate::live::opcodes_models::class::Class::Stormblade,
                                "Frost Mage" => crate::live::opcodes_models::class::Class::FrostMage,
                                "Wind Knight" => crate::live::opcodes_models::class::Class::WindKnight,
                                "Verdant Oracle" => crate::live::opcodes_models::class::Class::VerdantOracle,
                                "Heavy Guardian" => crate::live::opcodes_models::class::Class::HeavyGuardian,
                                "Marksman" => crate::live::opcodes_models::class::Class::Marksman,
                                "Shield Knight" => crate::live::opcodes_models::class::Class::ShieldKnight,
                                "Beat Performer" => crate::live::opcodes_models::class::Class::BeatPerformer,
                                _ => crate::live::opcodes_models::class::Class::Unknown,
                            };
                            if class_enum != crate::live::opcodes_models::class::Class::Unknown {
                                info!("Updated player {} class to: {:?}", uid, class_enum);
                                entity.class = Some(class_enum);
                            }
                        }
                    }
                    
                    // Update spec if current is missing
                    if entity.class_spec.is_none() && metadata.class_spec.is_some() {
                        if let Some(spec_str) = metadata.class_spec {
                            let spec_enum = match spec_str.as_str() {
                                "Iaido" => crate::live::opcodes_models::class::ClassSpec::Iaido,
                                "Moonstrike" => crate::live::opcodes_models::class::ClassSpec::Moonstrike,
                                "Icicle" => crate::live::opcodes_models::class::ClassSpec::Icicle,
                                "Frostbeam" => crate::live::opcodes_models::class::ClassSpec::Frostbeam,
                                "Vanguard" => crate::live::opcodes_models::class::ClassSpec::Vanguard,
                                "Skyward" => crate::live::opcodes_models::class::ClassSpec::Skyward,
                                "Smite" => crate::live::opcodes_models::class::ClassSpec::Smite,
                                "Lifebind" => crate::live::opcodes_models::class::ClassSpec::Lifebind,
                                "Earthfort" => crate::live::opcodes_models::class::ClassSpec::Earthfort,
                                "Block" => crate::live::opcodes_models::class::ClassSpec::Block,
                                "Wildpack" => crate::live::opcodes_models::class::ClassSpec::Wildpack,
                                "Falconry" => crate::live::opcodes_models::class::ClassSpec::Falconry,
                                "Recovery" => crate::live::opcodes_models::class::ClassSpec::Recovery,
                                "Shield" => crate::live::opcodes_models::class::ClassSpec::Shield,
                                "Dissonance" => crate::live::opcodes_models::class::ClassSpec::Dissonance,
                                "Concerto" => crate::live::opcodes_models::class::ClassSpec::Concerto,
                                _ => crate::live::opcodes_models::class::ClassSpec::Unknown,
                            };
                            if spec_enum != crate::live::opcodes_models::class::ClassSpec::Unknown {
                                info!("Updated player {} spec to: {:?}", uid, spec_enum);
                                entity.class_spec = Some(spec_enum);
                            }
                        }
                    }
                    
                    // Update ability_score if current is missing
                    if entity.ability_score.is_none() && metadata.ability_score.is_some() {
                        info!("Updated player {} ability_score to: {:?}", uid, metadata.ability_score);
                        entity.ability_score = metadata.ability_score;
                    }
                }
            }
        }
    }
}

pub async fn start(app_handle: AppHandle) {
    // todo: add app_handle?
    // https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html
    // 1. Start capturing packets and send to rx
    let mut rx = packets::packet_capture::start_capture(); // Since live meter is not critical, it's ok to just log it // TODO: maybe bubble an error up to the frontend instead?

    let is_bptimer_enabled = app_handle.svelte().get_or::<bool>("integration", "bptimer", true);

    // Spawn the periodic metadata refresh task for unknown players during combat
    let should_continue = Arc::new(Mutex::new(true));
    let app_handle_clone = Arc::new(app_handle.clone());
    let should_continue_clone = should_continue.clone();
    tokio::spawn(async move {
        refresh_unknown_player_metadata(app_handle_clone, should_continue_clone).await;
    });

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
                info!("Server change detected, saving encounter before reset");
                
                // Get a copy of the encounter and drop the lock before awaiting
                let encounter_copy = {
                    let encounter_state = app_handle.state::<EncounterMutex>();
                    let encounter = encounter_state.lock().unwrap();
                    
                    // Only save if there's combat data
                    if encounter.dmg_stats.value > 0 || encounter.heal_stats.value > 0 {
                        Some(encounter.clone())
                    } else {
                        None
                    }
                };
                
                // Now we can await without holding the lock
                if let Some(encounter_copy) = encounter_copy {
                    let db = app_handle.state::<crate::db::DbConnection>();
                    if let Ok(encounter_id) = crate::db::save_encounter(&db, &encounter_copy).await {
                        info!("Encounter saved due to server change with ID: {}", encounter_id);
                    } else {
                        info!("Failed to save encounter on server change");
                    }
                }
                
                // Reset the encounter after saving
                let encounter_state = app_handle.state::<EncounterMutex>();
                let mut encounter_state_lock = encounter_state.lock().unwrap();
                on_server_change(&mut encounter_state_lock);
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
                let db = app_handle.state::<crate::db::DbConnection>();
                let mut encounter_state = encounter_state.lock().unwrap();
                if process_sync_near_entities(&mut encounter_state, sync_near_entities, is_bptimer_enabled, &db).is_none() {
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
                let db = app_handle.state::<crate::db::DbConnection>();
                let mut encounter_state = encounter_state.lock().unwrap();
                encounter_state.local_player = Some(sync_container_data.clone());
                if process_sync_container_data(&mut encounter_state, sync_container_data, &db).is_none() {
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
            //         warn!("Error processing SyncToMeDeltaInfo.. ignoring.");
            //     }
            // }
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
                let db = app_handle.state::<crate::db::DbConnection>();
                let mut encounter_state = encounter_state.lock().unwrap();
                if process_sync_to_me_delta_info(&mut encounter_state, sync_to_me_delta_info, is_bptimer_enabled, &db).is_none() {
                    warn!("Error processing SyncToMeDeltaInfo - missing delta_info or base_delta in packet.");
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
                let db = app_handle.state::<crate::db::DbConnection>();
                let mut encounter_state = encounter_state.lock().unwrap();
                let delta_count = sync_near_delta_info.delta_infos.len();
                let mut processing_errors = 0;
                for aoi_sync_delta in sync_near_delta_info.delta_infos {
                    if process_aoi_sync_delta(&mut encounter_state, aoi_sync_delta, is_bptimer_enabled, &db).is_none() {
                        processing_errors += 1;
                    }
                }
                if processing_errors > 0 {
                    debug!("SyncNearDeltaInfo: {} of {} delta packets had missing uuid or skill_effects.", processing_errors, delta_count);
                }
            }
        }
    }
}
