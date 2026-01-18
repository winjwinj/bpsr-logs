use log::{error, info, warn};
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{self, Sender};
use std::sync::{LazyLock, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
struct CacheEntry {
    timestamp: u64,
    last_reported_hp: Option<i32>,
    is_pending: bool,
}

#[derive(serde::Deserialize)]
struct MobRecord {
    monster_id: u32,
    name: String,
    location: Option<bool>,
}

#[derive(serde::Deserialize)]
struct MobsResponse {
    items: Vec<MobRecord>,
}

pub struct BPTimerClient {
    api_url: String,
    api_key: String,
}

const CACHE_EXPIRY_MS: u64 = 5 * 60 * 1000; // 5 minutes

// Fallback mob mappings
const FALLBACK_MOB_MAPPINGS: &[(u32, &str)] = &[
    (10007, "Storm Goblin King"),
    (10009, "Frost Ogre"),
    (10010, "Tempest Ogre"),
    (10018, "Inferno Ogre"),
    (10029, "Muku King"),
    (10032, "Golden Juggernaut"),
    (10056, "Brigand Leader"),
    (10059, "Muku Chief"),
    (10069, "Phantom Arachnocrab"),
    (10077, "Venobzzar Incubator"),
    (10081, "Iron Fang"),
    (10084, "Celestial Flier"),
    (10085, "Lizardman King"),
    (10086, "Goblin King"),
    (10900, "Golden Nappo"),
    (10901, "Silver Nappo"),
    (10902, "Lovely Boarlet"),
    (10903, "Breezy Boarlet"),
    (10904, "Loyal Boarlet"),
];

// Fallback location-tracked mob IDs
const FALLBACK_LOCATION_TRACKED_MOBS: &[u32] = &[10900, 10901, 10904];

static HP_REPORT_CACHE: LazyLock<Mutex<HashMap<String, CacheEntry>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static MOB_MAPPING: LazyLock<Mutex<HashMap<u32, String>>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for (id, name) in FALLBACK_MOB_MAPPINGS {
        map.insert(*id, name.to_string());
    }
    Mutex::new(map)
});

static LOCATION_TRACKED_MOBS: LazyLock<Mutex<HashSet<u32>>> = LazyLock::new(|| {
    let mut set = HashSet::new();
    for &id in FALLBACK_LOCATION_TRACKED_MOBS {
        set.insert(id);
    }
    Mutex::new(set)
});

fn get_mob_name(mob_id: u32) -> Option<String> {
    MOB_MAPPING.lock().unwrap().get(&mob_id).cloned()
}

fn is_location_tracked_mob(mob_id: u32) -> bool {
    LOCATION_TRACKED_MOBS.lock().unwrap().contains(&mob_id)
}

fn is_mob_tracked(mob_id: u32) -> bool {
    MOB_MAPPING.lock().unwrap().contains_key(&mob_id)
}

fn set_mob_mapping(mapping: HashMap<u32, String>) {
    *MOB_MAPPING.lock().unwrap() = mapping;
}

fn set_location_tracked_mobs(mobs: HashSet<u32>) {
    *LOCATION_TRACKED_MOBS.lock().unwrap() = mobs;
}

struct HpReportTask {
    api_url: String,
    api_key: String,
    body: serde_json::Value,
    cache_key: String,
    rounded_hp_pct: i32,
}

static HP_REPORT_SENDER: OnceLock<Sender<HpReportTask>> = OnceLock::new();

fn get_hp_report_sender() -> &'static Sender<HpReportTask> {
    HP_REPORT_SENDER.get_or_init(|| {
        let (tx, rx) = mpsc::channel::<HpReportTask>();

        std::thread::spawn(move || {
            let user_agent = format!("BPSR-Logs/{}", env!("CARGO_PKG_VERSION"));
            let client = reqwest::blocking::Client::builder()
                .user_agent(&user_agent)
                .tls_backend_rustls()
                .build()
                .unwrap_or_else(|_| reqwest::blocking::Client::new());

            while let Ok(task) = rx.recv() {
                let res = client
                    .post(&task.api_url)
                    .header("X-API-Key", &task.api_key)
                    .header("Content-Type", "application/json")
                    .json(&task.body)
                    .send();

                if let Err(e) = &res {
                    error!("HP report failed: {}", e);
                } else if let Ok(resp) = &res {
                    if !resp.status().is_success() {
                        error!("HP report failed: HTTP {}", resp.status());
                    }
                }

                // Update cache on both success and error to prevent spam retries
                let mut cache = HP_REPORT_CACHE.lock().unwrap();
                if let Some(entry) = cache.get_mut(&task.cache_key) {
                    entry.last_reported_hp = Some(task.rounded_hp_pct);
                    entry.is_pending = false;
                }
            }
        });

        tx
    })
}

impl BPTimerClient {
    pub fn new(api_url: String, api_key: String) -> Self {
        Self { api_url, api_key }
    }

    /// Report HP to bptimer API
    pub fn report_hp(
        &self,
        monster_id: Option<u32>,
        curr_hp: Option<u64>,
        max_hp: Option<u64>,
        line: Option<i32>,
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
        if line <= 0 {
            return;
        }

        // Only process tracked monsters
        if !is_mob_tracked(monster_id) {
            return;
        }

        // Check if this mob requires position data
        if is_location_tracked_mob(monster_id) {
            let has_all_positions = pos_x.is_some() && pos_y.is_some() && pos_z.is_some();
            if !has_all_positions {
                return;
            }
        }

        // Calculate HP percentage
        if max_hp == 0 {
            return;
        }
        let hp_pct = (curr_hp as f32 / max_hp as f32) * 100.0;

        // Round to nearest 5%
        let rounded_hp_pct = (((hp_pct / 5.0).round() * 5.0) as i32).clamp(0, 100);

        let cache_key = format!("{}-{}", monster_id, line);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64;

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
            let monster_name = get_mob_name(monster_id)
                .unwrap_or_else(|| format!("Unknown Monster ({monster_id})"));

            let pos_info = match (pos_x, pos_y, pos_z) {
                (Some(x), Some(y), Some(z)) => {
                    format!(" at ({x:.2}, {y:.2}, {z:.2})")
                }
                _ => String::new(),
            };
            info!(
                "Reporting monster HP: {monster_name} (ID: {monster_id}) - HP: {rounded_hp_pct}% on line {line}{pos_info}"
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

            let task = HpReportTask {
                api_url: format!("{}/api/create-hp-report", self.api_url),
                api_key: self.api_key.clone(),
                body,
                cache_key: cache_key.clone(),
                rounded_hp_pct,
            };

            if let Err(e) = get_hp_report_sender().send(task) {
                error!("Failed to queue HP report: {}", e);
                //  Worker thread died - reset cache to prevent blocking future reports
                let mut cache = HP_REPORT_CACHE.lock().unwrap();
                if let Some(entry) = cache.get_mut(&cache_key) {
                    entry.last_reported_hp = Some(rounded_hp_pct);
                    entry.is_pending = false;
                }
            }
        }
    }

    /// Prefetch mobs from the database endpoint
    pub fn prefetch_mobs(&self) {
        if self.api_url.is_empty() || self.api_key.is_empty() {
            return;
        }

        let api_url = self.api_url.clone();
        let user_agent = format!("BPSR-Logs/{}", env!("CARGO_PKG_VERSION"));

        std::thread::spawn(move || {
            let client = reqwest::blocking::Client::builder()
                .user_agent(&user_agent)
                .tls_backend_rustls()
                .build()
                .unwrap_or_else(|_| reqwest::blocking::Client::new());

            let fields = "monster_id,name,location";
            let url = format!(
                "{}/api/collections/mobs/records?fields={}&perPage=100&skipTotal=true",
                api_url, fields
            );

            match client.get(&url).send() {
                Ok(resp) => {
                    if !resp.status().is_success() {
                        warn!("Prefetch failed: status {}", resp.status());
                        return;
                    }

                    match resp.json::<MobsResponse>() {
                        Ok(data) => {
                            let mut mob_mapping = HashMap::new();
                            let mut location_tracked_mobs = HashSet::new();

                            for mob in data.items {
                                if mob.monster_id > 0 && !mob.name.is_empty() {
                                    mob_mapping.insert(mob.monster_id, mob.name.clone());

                                    if mob.location == Some(true) {
                                        location_tracked_mobs.insert(mob.monster_id);
                                    }
                                }
                            }

                            let mob_count = mob_mapping.len();
                            let location_count = location_tracked_mobs.len();

                            set_mob_mapping(mob_mapping);
                            set_location_tracked_mobs(location_tracked_mobs);

                            info!(
                                "Prefetched {} mobs ({} location-tracked)",
                                mob_count, location_count
                            );
                        }
                        Err(e) => {
                            warn!("Prefetch failed to parse response: {}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Prefetch failed: {}", e);
                }
            }
        });
    }
}
