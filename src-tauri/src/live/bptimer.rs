use crate::live::opcodes_models::MONSTER_NAMES_CROWDSOURCE;
use log::{error, info};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
struct CacheEntry {
    timestamp: u128,
    last_reported_hp: Option<i32>,
    is_pending: bool,
}

static HP_REPORT_CACHE: Lazy<Mutex<HashMap<String, CacheEntry>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
const CACHE_EXPIRY_MS: u128 = 5 * 60 * 1000; // 5 minutes

pub struct BPTimerClient {
    api_url: String,
    api_key: String,
}

impl BPTimerClient {
    pub fn new(api_url: String, api_key: String) -> Self {
        Self { api_url, api_key }
    }

    /// Report HP to bptimer API
    pub fn report_hp(
        &self,
        monster_id: Option<i32>,
        curr_hp: Option<i32>,
        max_hp: Option<i32>,
        line: Option<u32>,
        pos_x: Option<f32>,
        pos_y: Option<f32>,
        pos_z: Option<f32>,
        account_id: Option<String>,
        uid: Option<i64>,
    ) {
        // Validate all required fields are present
        let Some(monster_id) = monster_id else {
            return;
        };
        let Some(curr_hp) = curr_hp else {
            return;
        };
        let Some(max_hp) = max_hp else {
            return;
        };
        let Some(line) = line else {
            return;
        };
        let Some(pos_x) = pos_x else {
            return;
        };
        let Some(pos_y) = pos_y else {
            return;
        };
        let Some(pos_z) = pos_z else {
            return;
        };

        // Only process crowdsourced monsters
        if !MONSTER_NAMES_CROWDSOURCE.contains_key(&monster_id) {
            return;
        }

        // Calculate HP percentage
        if max_hp == 0 {
            return;
        }
        let hp_pct = (curr_hp * 100 / max_hp).clamp(0, 100);

        // Round to nearest 5%
        let rounded_hp_pct = ((hp_pct as f32 / 5.0).round() * 5.0) as i32;
        let rounded_hp_pct = rounded_hp_pct.clamp(0, 100);

        let cache_key = format!("{}-{}", monster_id, line as i32);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();

        // Check cache
        let should_report = {
            let mut cache = HP_REPORT_CACHE.lock().unwrap();
            let entry = cache
                .entry(cache_key.clone())
                .or_insert_with(|| CacheEntry {
                    timestamp: now,
                    last_reported_hp: None,
                    is_pending: false,
                });

            // Reset expired entries
            if now - entry.timestamp > CACHE_EXPIRY_MS {
                entry.timestamp = now;
                entry.last_reported_hp = None;
                entry.is_pending = false;
            }

            // Skip if already reported this HP value or if report is pending
            if entry.last_reported_hp == Some(rounded_hp_pct) || entry.is_pending {
                false
            } else {
                entry.is_pending = true;
                true
            }
        };

        if should_report {
            let monster_name = MONSTER_NAMES_CROWDSOURCE
                .get(&monster_id)
                .map(|s| s.as_str())
                .unwrap_or("Unknown Monster");

            info!(
                "Reporting monster HP: {monster_name} (ID: {monster_id}) - HP: {rounded_hp_pct}% on line {line} at ({pos_x:.2}, {pos_y:.2}, {pos_z:.2})"
            );

            let body = serde_json::json!({
                "monster_id": monster_id,
                "hp_pct": rounded_hp_pct,
                "line": line,
                "pos_x": pos_x,
                "pos_y": pos_y,
                "pos_z": pos_z,
                "account_id": account_id,
                "uid": uid,
            });

            let api_url = format!("{}/api/create-hp-report", self.api_url);
            let api_key = self.api_key.clone();
            let cache_key_clone = cache_key.clone();

            std::thread::spawn(move || {
                let user_agent = format!("BPSR-Logs/{}", env!("CARGO_PKG_VERSION"));
                let client = reqwest::blocking::Client::builder()
                    .user_agent(&user_agent)
                    .use_rustls_tls()
                    .build()
                    .unwrap_or_else(|_| reqwest::blocking::Client::new());

                let res = client
                    .post(&api_url)
                    .header("X-API-Key", &api_key)
                    .header("Content-Type", "application/json")
                    .json(&body)
                    .send();

                match res {
                    Ok(resp) if resp.status().is_success() => {
                        // Success: Update cache
                        let mut cache = HP_REPORT_CACHE.lock().unwrap();
                        if let Some(entry) = cache.get_mut(&cache_key_clone) {
                            entry.last_reported_hp = Some(rounded_hp_pct);
                            entry.is_pending = false;
                        }
                        info!("Successfully reported HP for monster {monster_id}");
                    }
                    Ok(resp) => {
                        // HTTP error: Prevent retry spam
                        error!("HP report failed: HTTP {}", resp.status());
                        let mut cache = HP_REPORT_CACHE.lock().unwrap();
                        if let Some(entry) = cache.get_mut(&cache_key_clone) {
                            entry.last_reported_hp = Some(rounded_hp_pct); // Prevent retries
                            entry.is_pending = false;
                        }
                    }
                    Err(e) => {
                        // Network error: Prevent retry spam
                        error!("HP report failed: {}", e);
                        let mut cache = HP_REPORT_CACHE.lock().unwrap();
                        if let Some(entry) = cache.get_mut(&cache_key_clone) {
                            entry.last_reported_hp = Some(rounded_hp_pct); // Prevent retries
                            entry.is_pending = false;
                        }
                    }
                }
            });
        }
    }
}
