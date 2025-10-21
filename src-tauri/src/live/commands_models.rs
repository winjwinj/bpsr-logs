use crate::WINDOW_LIVE_LABEL;
use tauri::Manager;
use window_vibrancy::apply_blur;

// f64 is used here even when it doesn't make sense due to limitations with serde serializing u128 as a JSON number instead of a string

#[derive(specta::Type, serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HeaderInfo {
    pub total_dps: f64,
    pub total_dmg: f64,
    pub elapsed_ms: f64,
    pub time_last_combat_packet_ms: f64,
}

#[derive(specta::Type, serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayersWindow {
    pub player_rows: PlayerRows,
}

pub type PlayerRows = Vec<PlayerRow>;

#[derive(specta::Type, serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayerRow {
    pub uid: f64,
    pub name: String,
    pub class_name: String,
    pub class_spec_name: String,
    pub ability_score: f64,
    pub total_dmg: f64,
    pub dps: f64,
    pub dmg_pct: f64,
    pub crit_rate: f64,
    pub crit_dmg_rate: f64,
    pub lucky_rate: f64,
    pub lucky_dmg_rate: f64,
    pub hits: f64,
    pub hits_per_minute: f64,
}

#[derive(specta::Type, serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillsWindow {
    pub curr_player: PlayerRows,
    pub skill_rows: SkillRows,
}

pub type SkillRows = Vec<SkillRow>;

#[derive(specta::Type, serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillRow {
    pub uid: f64,
    pub name: String,
    pub total_dmg: f64,
    pub dps: f64,
    pub dmg_pct: f64,
    pub crit_rate: f64,
    pub crit_dmg_rate: f64,
    pub lucky_rate: f64,
    pub lucky_dmg_rate: f64,
    pub hits: f64,
    pub hits_per_minute: f64,
}
