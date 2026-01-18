use crate::live::opcodes_models::class::{Class, ClassSpec};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Default)]
pub struct PlayerState {
    account_id: Option<String>,
    uid: Option<i64>,
    line_id: Option<u32>,
}

impl PlayerState {
    pub fn set_account_info(&mut self, account_id: String, uid: i64) {
        self.set_account_id(account_id);
        self.set_uid(uid);
    }

    pub fn set_account_id(&mut self, account_id: String) {
        if self.account_id.as_deref() != Some(account_id.as_str()) {
            self.account_id = Some(account_id);
        }
    }

    pub fn set_uid(&mut self, uid: i64) {
        if self.uid != Some(uid) {
            self.uid = Some(uid);
        }
    }

    pub fn set_line_id(&mut self, line_id: u32) {
        if self.line_id != Some(line_id) {
            self.line_id = Some(line_id);
        }
    }

    pub fn get_account_id(&self) -> Option<String> {
        self.account_id.clone()
    }

    pub fn get_uid(&self) -> i64 {
        self.uid.unwrap_or(-1)
    }

    pub fn get_uid_opt(&self) -> Option<i64> {
        self.uid
    }

    pub fn get_line_id_opt(&self) -> Option<u32> {
        self.line_id
    }
}

#[derive(Debug, Default, Clone)]
pub struct PlayerCacheEntry {
    pub name: Option<String>,
    pub class: Option<Class>,
    pub class_spec: Option<ClassSpec>,
    pub ability_score: Option<i32>,
}

#[derive(Debug, Default)]
pub struct PlayerCache {
    cache: HashMap<i64, PlayerCacheEntry>,
}

impl PlayerCache {
    pub fn set_name(&mut self, uid: i64, name: String) {
        let entry = self.cache.entry(uid).or_default();
        if entry.name.as_deref() != Some(name.as_str()) {
            entry.name = Some(name);
        }
    }

    pub fn set_class(&mut self, uid: i64, class: Class) {
        let entry = self.cache.entry(uid).or_default();
        if entry.class != Some(class) {
            entry.class = Some(class);
        }
    }

    pub fn set_class_spec(&mut self, uid: i64, class_spec: ClassSpec) {
        let entry = self.cache.entry(uid).or_default();
        if entry.class_spec != Some(class_spec) {
            entry.class_spec = Some(class_spec);
        }
    }

    pub fn set_ability_score(&mut self, uid: i64, ability_score: i32) {
        let entry = self.cache.entry(uid).or_default();
        if entry.ability_score != Some(ability_score) {
            entry.ability_score = Some(ability_score);
        }
    }

    pub fn set_both(&mut self, uid: i64, name: Option<String>, class: Option<Class>) {
        let entry = self.cache.entry(uid).or_default();
        if let Some(n) = name {
            if entry.name.as_deref() != Some(n.as_str()) {
                entry.name = Some(n);
            }
        }
        if let Some(c) = class {
            if entry.class != Some(c) {
                entry.class = Some(c);
            }
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

    pub fn get_ability_score(&self, uid: i64) -> Option<i32> {
        self.cache.get(&uid).and_then(|e| e.ability_score)
    }

    pub fn get(&self, uid: i64) -> Option<&PlayerCacheEntry> {
        self.cache.get(&uid)
    }
}

pub type PlayerStateMutex = Mutex<PlayerState>;
pub type PlayerCacheMutex = Mutex<PlayerCache>;
