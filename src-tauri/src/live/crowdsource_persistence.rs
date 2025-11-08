use crate::live::opcodes_models::Encounter;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

const SNAPSHOT_FILENAME: &str = "crowdsource_monster.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrowdsourceMonsterSnapshot {
    pub monster_id: i32,
    pub monster_name: String,
    pub remote_id: String,
}

fn snapshot_path(app_handle: &AppHandle) -> Option<PathBuf> {
    match app_handle.path().app_data_dir() {
        Ok(mut path) => {
            path.push(SNAPSHOT_FILENAME);
            Some(path)
        }
        Err(err) => {
            warn!(
                "crowdsource_persistence::snapshot_path - failed to resolve app data dir: {err}"
            );
            None
        }
    }
}

fn ensure_parent(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

pub fn load_snapshot(app_handle: &AppHandle) -> Option<CrowdsourceMonsterSnapshot> {
    let path = snapshot_path(app_handle)?;
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(err) => {
            if err.kind() != std::io::ErrorKind::NotFound {
                warn!(
                    "crowdsource_persistence::load_snapshot - failed to read snapshot file {}: {err}",
                    path.display()
                );
            }
            return None;
        }
    };

    match serde_json::from_str::<CrowdsourceMonsterSnapshot>(&contents) {
        Ok(snapshot) => {
            info!(
                "crowdsource_persistence::load_snapshot - loaded snapshot for monster '{}' ({})",
                snapshot.monster_name, snapshot.monster_id
            );
            Some(snapshot)
        }
        Err(err) => {
            warn!(
                "crowdsource_persistence::load_snapshot - failed to parse snapshot file {}: {err}",
                path.display()
            );
            None
        }
    }
}

pub fn save_snapshot(
    app_handle: &AppHandle,
    snapshot: &CrowdsourceMonsterSnapshot,
) -> Result<(), String> {
    let path = snapshot_path(app_handle)
        .ok_or_else(|| "Failed to resolve app data directory".to_string())?;

    ensure_parent(&path)
        .map_err(|err| format!("Failed to create snapshot directory: {err}"))?;

    let data = serde_json::to_string_pretty(snapshot)
        .map_err(|err| format!("Failed to serialize snapshot: {err}"))?;

    fs::write(&path, data)
        .map_err(|err| format!("Failed to write snapshot file {}: {err}", path.display()))?;

    info!(
        "crowdsource_persistence::save_snapshot - saved snapshot for monster '{}' ({})",
        snapshot.monster_name, snapshot.monster_id
    );

    Ok(())
}

pub fn snapshot_from_encounter(encounter: &Encounter) -> Option<CrowdsourceMonsterSnapshot> {
    Some(CrowdsourceMonsterSnapshot {
        monster_id: encounter.crowdsource_monster_id?,
        monster_name: encounter.crowdsource_monster_name.clone()?,
        remote_id: encounter.crowdsource_monster_remote_id.clone()?,
    })
}

pub fn apply_snapshot_to_encounter(snapshot: &CrowdsourceMonsterSnapshot, encounter: &mut Encounter) {
    encounter.crowdsource_monster_id = Some(snapshot.monster_id);
    encounter.crowdsource_monster_name = Some(snapshot.monster_name.clone());
    encounter.crowdsource_monster_remote_id = Some(snapshot.remote_id.clone());
}

