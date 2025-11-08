use futures_util::StreamExt;
use log::{info, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json;
use specta::Type;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::watch;
use tauri_plugin_svelte::ManagerExt;
use super::opcodes_models::EncounterMutex;

pub const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36";
pub const BPTIMER_BASE_URL: &str = "https://db.bptimer.com";
pub const MOB_COLLECTION_AUTH_TOKEN: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJjb2xsZWN0aW9uSWQiOiJfcGJfdXNlcnNfYXV0aF8iLCJleHAiOjE3NjMxMTYwMTIsImlkIjoibmhtc2s3Z2g1ODhieXc3IiwicmVmcmVzaGFibGUiOnRydWUsInR5cGUiOiJhdXRoIn0.I81wYPhG0u8IUcQWZGBFsKS5abnQ1JOtFjIcjqkyO0A";
pub const MOB_CHANNEL_STATUS_ENDPOINT: &str =
    "/api/collections/mob_channel_status/records";
pub const REALTIME_ENDPOINT: &str = "/api/realtime";
pub const CREATE_HP_REPORT_ENDPOINT: &str = "/api/create-hp-report";
pub const CROWD_SOURCE_API_KEY: &str = "8fibznvjgf9vh29bg7g730fan9xaskf7h45lzdl2891vi0w1d2";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobHpUpdate {
    pub remote_id: String,
    pub server_id: i32,
    pub hp_percent: i32,
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct MobHpData {
    pub remote_id: String,
    pub server_id: i32,
    pub hp_percent: i32,
}

#[derive(Debug, Default)]
pub struct MobHpStore {
    // Primary storage: remote UID -> (server_id -> MobHpData, timestamp)
    instances: HashMap<String, HashMap<i32, (MobHpData, std::time::Instant)>>,
}

impl MobHpStore {
    pub fn update(&mut self, update: MobHpUpdate) {
        let entry = self
            .instances
            .entry(update.remote_id.clone())
            .or_default();

        let data = MobHpData {
            remote_id: update.remote_id.clone(),
            server_id: update.server_id,
            hp_percent: update.hp_percent,
        };

        entry.insert(update.server_id, (data, std::time::Instant::now()));

        if update.server_id != 0 {
            entry.remove(&0);
        }
    }

    pub fn seed_remote_hp(&mut self, remote_id: &str, server_id: i32, hp_percent: Option<i32>) {
        let entry = self
            .instances
            .entry(remote_id.to_string())
            .or_default();

        if server_id != 0 {
            entry.remove(&0);
        }

        let data = MobHpData {
            remote_id: remote_id.to_string(),
            server_id,
            hp_percent: hp_percent.unwrap_or(100),
        };

        entry.insert(server_id, (data, std::time::Instant::now()));
    }

    pub fn get_by_remote_id(&self, remote_id: &str) -> Vec<MobHpData> {
        let mut result = self
            .instances
            .get(remote_id)
            .map(|servers| {
                servers
                    .values()
                    .map(|(data, _)| data.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        result.sort_by_key(|entry| entry.server_id);
        result
    }

    #[allow(dead_code)]
    pub fn remove(&mut self, remote_id: &str) {
        self.instances.remove(remote_id);
    }

    #[allow(dead_code)]
    pub fn cleanup_old(&mut self, max_age_secs: u64) {
        let now = std::time::Instant::now();
        let mut empty_keys = Vec::new();

        for (remote_id, server_map) in self.instances.iter_mut() {
            server_map.retain(|_, (_, timestamp)| {
                now.duration_since(*timestamp).as_secs() <= max_age_secs
            });

            if server_map.is_empty() {
                empty_keys.push(remote_id.clone());
            }
        }

        for key in empty_keys {
            self.instances.remove(&key);
        }
    }
}

pub type MobHpStoreMutex = Arc<RwLock<MobHpStore>>;

pub type BpTimerStreamControlSender = Arc<watch::Sender<bool>>;
pub type BpTimerStreamControlReceiver = watch::Receiver<bool>;

pub fn stream_control_channel() -> (BpTimerStreamControlSender, BpTimerStreamControlReceiver) {
    let (sender, receiver) = watch::channel(false);
    (Arc::new(sender), receiver)
}

#[derive(Debug, Deserialize)]
struct MobChannelStatusResponse {
    items: Vec<MobChannelStatusItem>,
}

#[derive(Debug, Deserialize, Serialize, Type, Clone)]
pub struct MobChannelStatusItem {
    #[serde(rename = "channel_number")]
    pub channel_number: i32,
    #[serde(rename = "last_hp")]
    pub last_hp: Option<i32>,
    pub mob: String,
}

fn current_monster_info(app_handle: &AppHandle) -> (Option<String>, Option<String>) {
    app_handle
        .try_state::<EncounterMutex>()
        .and_then(|encounter_mutex| {
            encounter_mutex.lock().ok().map(|encounter| {
                (
                    encounter.crowdsource_monster_name.clone(),
                    encounter.crowdsource_monster_remote_id.clone(),
                )
            })
        })
        .unwrap_or((None, None))
}

pub async fn start_bptimer_stream(
    app_handle: AppHandle,
    store: MobHpStoreMutex,
    mut control_rx: BpTimerStreamControlReceiver,
) {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(30000))
        .pool_idle_timeout(std::time::Duration::from_secs(300))
        .pool_max_idle_per_host(1)
        .tcp_keepalive(std::time::Duration::from_secs(60))
        .build()
        .expect("Failed to create HTTP client");
    let realtime_url = format!("{BPTIMER_BASE_URL}{REALTIME_ENDPOINT}");
    let mut last_seeded_remote_id: Option<String> = None;

    loop {
        if !*control_rx.borrow() {
            last_seeded_remote_id = None;
            if control_rx.changed().await.is_err() {
                break;
            }
            continue;
        }

        let is_enabled = app_handle.svelte().get_or::<bool>("integration", "bptimerUI", true);

        if !is_enabled {
            last_seeded_remote_id = None;
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            continue;
        }

        let (_current_monster_name, current_remote_id) = current_monster_info(&app_handle);

        if current_remote_id != last_seeded_remote_id {
            if let Err(e) = seed_initial_mob_state(
                &app_handle,
                &client,
                store.clone(),
                current_remote_id.as_deref(),
            )
            .await
            {
                warn!("bptimer_stream::start_bptimer_stream - Failed to seed mob state: {e}");
            }

            last_seeded_remote_id = current_remote_id.clone();
        }

        info!("bptimer_stream::start_bptimer_stream - Connecting to BPTimer SSE");

        if !*control_rx.borrow() {
            last_seeded_remote_id = None;
            continue;
        }

        match client
            .get(&realtime_url)
            .header("accept", "text/event-stream")
            .header("accept-language", "en-US,en;q=0.8")
            .header("cache-control", "no-cache")
            .header("origin", "https://bptimer.com")
            .header("referer", "https://bptimer.com/")
            .header("pragma", "no-cache")
            .header("sec-fetch-dest", "empty")
            .header("sec-fetch-mode", "cors")
            .header("sec-fetch-site", "same-site")
            .send()
            .await
        {
            Ok(response) => {
                let stream_control_rx = control_rx.clone();
                if let Err(e) = stream_sse(
                    response,
                    &client,
                    app_handle.clone(),
                    store.clone(),
                    stream_control_rx,
                )
                .await
                {
                    warn!("bptimer_stream::start_bptimer_stream - Stream error: {e}");
                }
            }
            Err(e) => {
                warn!("bptimer_stream::start_bptimer_stream - Connection failed: {e}");
            }
        }

        if !*control_rx.borrow() {
            last_seeded_remote_id = None;
            continue;
        }

        // Wait before reconnecting while active
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }
}

async fn seed_initial_mob_state(
    app_handle: &AppHandle,
    client: &Client,
    store: MobHpStoreMutex,
    remote_id: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(remote_id) = remote_id else {
        info!(
            "bptimer_stream::seed_initial_mob_state - no remote id available; skipping preload"
        );
        return Ok(());
    };

    info!(
        "bptimer_stream::seed_initial_mob_state - preloading state for remote_id={}",
        remote_id
    );

    let items = fetch_mob_channel_status(client, remote_id).await?;

    let mut updates_to_emit = Vec::new();

    {
        let mut store = store.write();

        info!(
            "bptimer_stream::seed_initial_mob_state - applying {} channel entries for {}",
            items.len(),
            remote_id
        );

        if items.is_empty() {
            store.seed_remote_hp(remote_id, 0, None);
            updates_to_emit.push(MobHpUpdate {
                remote_id: remote_id.to_string(),
                server_id: 0,
                hp_percent: 100,
            });
        } else {
            for item in items {
                store.seed_remote_hp(&item.mob, item.channel_number, item.last_hp);
                updates_to_emit.push(MobHpUpdate {
                    remote_id: item.mob.clone(),
                    server_id: item.channel_number,
                    hp_percent: item.last_hp.unwrap_or(100),
                });
            }
        }
    }

    for update in updates_to_emit {
        let _ = app_handle.emit("mob-hp-update", &update);
    }

    Ok(())
}

pub async fn fetch_mob_channel_status(
    client: &Client,
    mob_uid: &str,
) -> Result<Vec<MobChannelStatusItem>, Box<dyn std::error::Error>> {
    let url = format!(
        "{base}{endpoint}?page=1&perPage=200&skipTotal=true&filter=mob%20%3D%20%27{uid}%27",
        base = BPTIMER_BASE_URL,
        endpoint = MOB_CHANNEL_STATUS_ENDPOINT,
        uid = mob_uid
    );

    info!(
        "bptimer_stream::fetch_mob_channel_status - fetching status for mob_uid={}",
        mob_uid
    );

    let response = client
        .get(url)
        .header("authorization", MOB_COLLECTION_AUTH_TOKEN)
        .header("accept", "*/*")
        .header("origin", "https://bptimer.com")
        .header("user-agent", USER_AGENT)
        .send()
        .await?;

    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(format!(
            "bptimer_stream::fetch_mob_channel_status - request failed with status {}: {}",
            status, body
        )
        .into());
    }

    let payload: MobChannelStatusResponse = match serde_json::from_str(&body) {
        Ok(parsed) => parsed,
        Err(err) => {
            warn!(
                "bptimer_stream::fetch_mob_channel_status - failed to decode body for {}: {err}; body={}",
                mob_uid,
                body
            );
            return Ok(vec![]);
        }
    };
    info!(
        "bptimer_stream::fetch_mob_channel_status - received {} records",
        payload.items.len()
    );
    Ok(payload.items)
}

async fn stream_sse(
    response: reqwest::Response,
    client: &Client,
    app_handle: AppHandle,
    store: MobHpStoreMutex,
    control: BpTimerStreamControlReceiver,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = response.bytes_stream();
    let mut buffer = Vec::<u8>::new();
    let mut current_event: Option<SseEvent> = None;
    let mut subscribed = false;
    let mut last_reseed = std::time::Instant::now();
    let mut last_remote_id: Option<String> = None;
    
    loop {
        let (_monster_name, current_remote_id) = current_monster_info(&app_handle);
        let reseed_due =
            current_remote_id != last_remote_id || last_reseed.elapsed() >= std::time::Duration::from_secs(120);

        if reseed_due {
            if let Err(e) = seed_initial_mob_state(
                &app_handle,
                client,
                store.clone(),
                current_remote_id.as_deref(),
            )
            .await
            {
                if let Some(remote_id) = current_remote_id.as_deref() {
                    warn!(
                        "bptimer_stream::stream_sse - Reseed failed for {}: {e}",
                        remote_id
                    );
                } else {
                    warn!(
                        "bptimer_stream::stream_sse - Reseed failed without a remote id: {e}"
                    );
                }
            }
            last_reseed = std::time::Instant::now();
            last_remote_id = current_remote_id.clone();
        }

        if !*control.borrow() {
            info!("bptimer_stream::stream_sse - control disabled, closing stream");
            return Ok(());
        }
        match tokio::time::timeout(std::time::Duration::from_secs(5), stream.next()).await {
            Ok(Some(Ok(chunk))) => {
                buffer.extend_from_slice(&chunk);
            }
            Ok(Some(Err(e))) => {
                warn!("bptimer_stream::stream_sse - Stream error: {e}");
                return Err(e.into());
            }
            Ok(None) => {
                break;
            }
            Err(_) => {
                if !*control.borrow() {
                    info!("bptimer_stream::stream_sse - timeout and control disabled, closing stream");
                    return Ok(());
                }
                continue;
            }
        }
        
        // Process complete lines
        while let Some(newline_pos) = buffer.iter().position(|&b| b == b'\n') {
            let line_bytes = buffer.drain(..=newline_pos).collect::<Vec<_>>();
            let line = String::from_utf8_lossy(&line_bytes[..line_bytes.len() - 1]).to_string();
            
            if line.trim().is_empty() {
                // Empty line means end of event
                if let Some(event) = current_event.take() {
                    if let Some(data) = event.data {
                        // Handle PB_CONNECT event - send subscription
                        if !subscribed && event.event_type == Some("PB_CONNECT".to_string()) {
                            info!("bptimer_stream::stream_sse - Received PB_CONNECT, sending subscription");
                            let client_id_opt = {
                                match parse_pb_connect(&data) {
                                    Ok(client_id) => Some(client_id),
                                    Err(e) => {
                                        warn!("bptimer_stream::stream_sse - Failed to parse PB_CONNECT: {}", e);
                                        None
                                    }
                                }
                            };
                            
                            if let Some(client_id) = client_id_opt {
                                match send_subscription(&client_id).await {
                                    Ok(_) => {
                                        info!("bptimer_stream::stream_sse - Subscribed successfully");
                                        subscribed = true;
                                    }
                                    Err(e) => {
                                        warn!("bptimer_stream::stream_sse - Failed to send subscription: {e}");
                                    }
                                }
                            }
                        } else if event.event_type == Some("mob_hp_updates".to_string()) {
                            if let Ok(update) = parse_mob_hp_update(&data) {
                                // Store the update
                                store.write().update(update.clone());
                                // Emit to frontend
                                let _ = app_handle.emit("mob-hp-update", &update);
                            }
                        } else if event.event_type == Some("PB_SUBSCRIBED".to_string()) {
                            subscribed = true;
                            info!("bptimer_stream::stream_sse - Subscription acknowledged by server");
                        }
                    }
                }
            } else if line.starts_with("id:") {
                let id = line[3..].trim().to_string();
                if let Some(ref mut event) = current_event {
                    event.id = Some(id);
                } else {
                    current_event = Some(SseEvent {
                        event_type: None,
                        data: None,
                        id: Some(id),
                    });
                }
            } else if line.starts_with("event:") {
                let event_type = line[6..].trim().to_string();
                if let Some(ref mut event) = current_event {
                    event.event_type = Some(event_type);
                } else {
                    current_event = Some(SseEvent {
                        event_type: Some(event_type),
                        data: None,
                        id: None,
                    });
                }
            } else if line.starts_with("data:") {
                let data = line[5..].trim().to_string();
                if let Some(ref mut event) = current_event {
                    // SSE allows multiple data lines - append them
                    if let Some(ref existing_data) = event.data {
                        event.data = Some(format!("{}\n{}", existing_data, data));
                    } else {
                        event.data = Some(data);
                    }
                } else {
                    current_event = Some(SseEvent {
                        event_type: None,
                        data: Some(data),
                        id: None,
                    });
                }
            }
        }
    }
    
    Ok(())
}

struct SseEvent {
    event_type: Option<String>,
    data: Option<String>,
    id: Option<String>,
}

fn parse_pb_connect(data: &str) -> Result<String, Box<dyn std::error::Error>> {
    #[derive(Deserialize)]
    struct PbConnect {
        #[serde(rename = "clientId")]
        client_id: String,
    }
    
    let parsed: PbConnect = serde_json::from_str(data)?;
    Ok(parsed.client_id)
}

async fn send_subscription(client_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    #[derive(Serialize)]
    struct Subscription {
        #[serde(rename = "clientId")]
        client_id: String,
        subscriptions: Vec<String>,
    }
    
    let subscription = Subscription {
        client_id: client_id.to_string(),
        subscriptions: vec!["mob_hp_updates".to_string(), "mob_resets".to_string()],
    };
    
    let client = Client::builder()
        .user_agent(USER_AGENT)
        .build()?;
    let response = client
        .post(format!("{BPTIMER_BASE_URL}{REALTIME_ENDPOINT}"))
        .header("content-type", "application/json")
        .header("origin", "https://bptimer.com")
        .header("referer", "https://bptimer.com/")
        .header("authorization", MOB_COLLECTION_AUTH_TOKEN)
        .header("accept", "*/*")
        .header("cache-control", "no-cache")
        .header("pragma", "no-cache")
        .header("priority", "u=1, i")
        .header("sec-ch-ua", "\"Chromium\";v=\"142\", \"Brave\";v=\"142\", \"Not_A Brand\";v=\"99\"")
        .header("sec-ch-ua-mobile", "?0")
        .header("sec-ch-ua-platform", "\"Windows\"")
        .header("sec-fetch-dest", "empty")
        .header("sec-fetch-mode", "cors")
        .header("sec-fetch-site", "same-site")
        .header("sec-gpc", "1")
        .json(&subscription)
        .send()
        .await?;
    
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Subscription failed with status {}: {}", status, body).into());
    }
    
    Ok(())
}

fn parse_mob_hp_update(data: &str) -> Result<MobHpUpdate, Box<dyn std::error::Error>> {
    let parsed: Vec<serde_json::Value> = serde_json::from_str(data)?;
    
    if parsed.len() < 3 {
        return Err("Expected array with 3 elements".into());
    }
    
    let remote_id = parsed[0]
        .as_str()
        .ok_or("Expected string for remote_id")?
        .to_string();
    let server_id = parsed[1]
        .as_i64()
        .ok_or("Expected number for server_id")? as i32;
    let hp_percent = parsed[2]
        .as_i64()
        .ok_or("Expected number for hp_percent")? as i32;
    
    Ok(MobHpUpdate {
        remote_id,
        server_id,
        hp_percent,
    })
}

