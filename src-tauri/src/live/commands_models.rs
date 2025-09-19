use std::collections::HashMap;
use once_cell::sync::Lazy;

#[derive(specta::Type)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DPSWindow {
    pub dps_rows: DPSRows,
    pub total_dmg: u128,
    pub elapsed_ms: u128,
}

pub type DPSRows = Vec<DPSRow>;

#[derive(specta::Type)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DPSRow {
    pub uid: i64,
    pub name: String,
    pub class: String,
    // pub class_style: String,
    pub ability_score: i32,
    pub total_dmg: u128,
    pub dps: f64,
    pub crit_rate: f64,
    pub crit_dmg_rate: f64,
    pub lucky_rate: f64,
    pub lucky_dmg_rate: f64,
    pub hits: u128,
    pub hits_per_second: f64,
    pub skills: SkillRows,
}

pub type SkillRows = Vec<SkillRow>;

#[derive(specta::Type)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillRow {
    pub name: String,
    pub total_dmg: u128,
    pub dps: f64,
    pub crit_rate: f64,
    pub crit_dmg_rate: f64,
    pub lucky_rate: f64,
    pub lucky_dmg_rate: f64,
    pub hits: u128,
    pub hits_per_second: f64,
}
