use crate::live::opcodes_models::class::{Class, ClassSpec};
use std::collections::HashMap;
use std::sync::Mutex;

/// Persistent player identity data that survives combat/encounter resets
/// This stores the LOCAL player's account information separately from combat stats
#[derive(Debug, Default)]
pub struct PlayerState {
    pub account_id: Option<String>,
    pub uid: Option<i64>,
    pub line_id: Option<u32>,
}

impl PlayerState {
    pub fn set_account_info(&mut self, account_id: String, uid: i64) {
        if self.account_id.is_none() || self.account_id.as_ref() != Some(&account_id) {
            self.account_id = Some(account_id);
        }
        if self.uid.is_none() || self.uid != Some(uid) {
            self.uid = Some(uid);
        }
    }

    pub fn set_line_id(&mut self, line_id: u32) {
        self.line_id = Some(line_id);
    }

    pub fn get_account_id(&self) -> Option<String> {
        self.account_id.clone()
    }

    pub fn get_uid(&self) -> Option<i64> {
        self.uid
    }

    pub fn get_line_id(&self) -> Option<u32> {
        self.line_id
    }

    // Just for semantic clarity
    pub fn get_local_player_uid(&self) -> Option<i64> {
        self.uid
    }
}

pub type PlayerStateMutex = Mutex<PlayerState>;

/// Cached player data entry
#[derive(Debug, Default, Clone)]
pub struct PlayerCacheEntry {
    pub name: Option<String>,
    pub class: Option<Class>,
    pub class_spec: Option<ClassSpec>,
}

/// Global player cache (uid => player data)
/// Persists across encounter resets until app closes
#[derive(Debug, Default)]
pub struct PlayerCache {
    cache: HashMap<i64, PlayerCacheEntry>,
}

impl PlayerCache {
    pub fn set_name(&mut self, uid: i64, name: String) {
        self.cache.entry(uid).or_default().name = Some(name);
    }

    pub fn set_class(&mut self, uid: i64, class: Class) {
        self.cache.entry(uid).or_default().class = Some(class);
    }

    pub fn set_class_spec(&mut self, uid: i64, class_spec: ClassSpec) {
        self.cache.entry(uid).or_default().class_spec = Some(class_spec);
    }

    pub fn set_both(&mut self, uid: i64, name: Option<String>, class: Option<Class>) {
        let entry = self.cache.entry(uid).or_default();
        if let Some(n) = name {
            entry.name = Some(n);
        }
        if let Some(c) = class {
            entry.class = Some(c);
        }
    }

    pub fn get_name(&self, uid: i64) -> Option<String> {
        self.cache.get(&uid).and_then(|e| e.name.clone())
    }

    pub fn get_class(&self, uid: i64) -> Option<Class> {
        self.cache.get(&uid).and_then(|e| e.class)
    }

    pub fn get_class_spec(&self, uid: i64) -> Option<ClassSpec> {
        self.cache.get(&uid).and_then(|e| e.class_spec)
    }

    pub fn get(&self, uid: i64) -> Option<&PlayerCacheEntry> {
        self.cache.get(&uid)
    }
}

pub type PlayerCacheMutex = Mutex<PlayerCache>;
