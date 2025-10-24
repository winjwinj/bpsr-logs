use crate::live::opcodes_models::class::ClassSpec;
use blueprotobuf_lib::blueprotobuf::{EEntityType, SyncContainerData};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Default, Clone)]
pub struct Encounter {
    pub is_encounter_paused: bool,
    pub time_last_combat_packet_ms: u128, // in ms
    pub time_fight_start_ms: u128,        // in ms
    pub total_dmg: i64,
    pub total_heal: i64,
    pub local_player_uid: i64,
    pub entity_uid_to_entity: HashMap<i64, Entity>, // key: entity uid
    pub local_player: SyncContainerData,
}

pub type EncounterMutex = Mutex<Encounter>;

#[derive(Debug, Default, Clone)]
pub struct Entity {
    pub name: String,
    pub entity_type: EEntityType,
    pub class_id: i32,
    pub class_spec: ClassSpec,
    pub ability_score: i32,
    pub level: i32,
    // Damage
    pub total_dmg: i64,
    pub crit_total_dmg: i64,
    pub crit_hits_dmg: i64,
    pub lucky_total_dmg: i64,
    pub lucky_hits_dmg: i64,
    pub hits_dmg: i64,
    pub skill_uid_to_dmg_skill: HashMap<i32, Skill>,
    // Healing
    pub total_heal: i64,
    pub crit_total_heal: i64,
    pub crit_hits_heal: i64,
    pub lucky_total_heal: i64,
    pub lucky_hits_heal: i64,
    pub hits_heal: i64,
    pub skill_uid_to_heal_skill: HashMap<i32, Skill>,
    // For Monsters
    pub curr_hp: i32,
    pub max_hp: i32,
    pub monster_id: i32,
}

#[derive(Debug, Default, Clone)]
pub struct Skill {
    pub total_value: i64,
    pub crit_total_value: i64,
    pub crit_hits: i64,
    pub lucky_total_value: i64,
    pub lucky_hits: i64,
    pub hits: i64,
}

static SKILL_NAMES: Lazy<HashMap<i32, String>> = Lazy::new(|| {
    let data = include_str!("../../../src/lib/data/json/SkillName.json");
    serde_json::from_str(data).expect("invalid SkillName.json")
});

impl Skill {
    pub fn get_skill_name(skill_uid: i32) -> String {
        SKILL_NAMES
            .get(&skill_uid)
            .cloned()
            .unwrap_or_else(|| format!("UNKNOWN SKILL ({skill_uid})"))
    }
}

pub static MONSTER_NAMES: Lazy<HashMap<i32, String>> = Lazy::new(|| {
    let data = include_str!("../../../src/lib/data/json/MonsterName.json");
    serde_json::from_str(data).expect("invalid MonsterName.json")
});

pub static MONSTER_NAMES_CROWDSOURCE: Lazy<HashMap<i32, String>> = Lazy::new(|| {
    let data = include_str!("../../../src/lib/data/json/MonsterNameCrowdsource.json");
    serde_json::from_str(data).expect("invalid MonsterName.json")
});

pub mod attr_type {
    pub const ATTR_NAME: i32 = 0x01;
    pub const ATTR_ID: i32 = 0x0a;
    pub const ATTR_PROFESSION_ID: i32 = 0xdc;
    pub const ATTR_FIGHT_POINT: i32 = 0x272e;
    pub const ATTR_LEVEL: i32 = 0x2710;
    // pub const ATTR_RANK_LEVEL: i32 = 0x274c;
    // pub const ATTR_CRI: i32 = 0x2b66;
    // pub const ATTR_LUCKY: i32 = 0x2b7a;
    pub const ATTR_HP: i32 = 0x2c2e;
    pub const ATTR_MAX_HP: i32 = 0x2c38;
    // pub const ATTR_ELEMENT_FLAG: i32 = 0x646d6c;
    // pub const ATTR_REDUCTION_LEVEL: i32 = 0x64696d;
    // pub const ATTR_REDUCTION_ID: i32 = 0x6f6c65;
    // pub const ATTR_ENERGY_FLAG: i32 = 0x543cd3c6;
}

// TODO: this logic needs to be severely cleaned up
pub mod class {
    pub const UNKNOWN: i32 = 0;
    pub const STORMBLADE: i32 = 1;
    pub const FROST_MAGE: i32 = 2;
    pub const WIND_KNIGHT: i32 = 4;
    pub const VERDANT_ORACLE: i32 = 5;
    pub const HEAVY_GUARDIAN: i32 = 9;
    pub const MARKSMAN: i32 = 11;
    pub const SHIELD_KNIGHT: i32 = 12;
    pub const BEAT_PERFORMER: i32 = 13;

    pub fn get_class_name(id: i32) -> String {
        String::from(match id {
            STORMBLADE => "Stormblade",
            FROST_MAGE => "Frost Mage",
            WIND_KNIGHT => "Wind Knight",
            VERDANT_ORACLE => "Verdant Oracle",
            HEAVY_GUARDIAN => "Heavy Guardian",
            MARKSMAN => "Marksman",
            SHIELD_KNIGHT => "Shield Knight",
            BEAT_PERFORMER => "Beat Performer",
            _ => "", // empty string for unknown
        })
    }

    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub enum ClassSpec {
        #[default]
        Unknown,
        // Stormblade
        Iaido,
        Moonstrike,
        // Frost Mage
        Icicle,
        Frostbeam,
        // Wind Knight
        Vanguard,
        Skyward,
        // Verdant Oracle
        Smite,
        Lifebind,
        // Heavy Guardian
        Earthfort,
        Block,
        // Marksman
        Wildpack,
        Falconry,
        // Shield Knight
        Recovery,
        Shield,
        // Beat Performer
        Dissonance,
        Concerto,
    }

    pub fn get_class_spec_from_skill_id(skill_id: i32) -> ClassSpec {
        match skill_id {
            1714 | 1734 => ClassSpec::Iaido,
            44701 | 179906 => ClassSpec::Moonstrike,

            120901 | 120902 => ClassSpec::Icicle,
            1241 => ClassSpec::Frostbeam,

            1405 | 1418 => ClassSpec::Vanguard,
            1419 => ClassSpec::Skyward,

            1518 | 1541 | 21402 => ClassSpec::Smite,
            20301 => ClassSpec::Lifebind,

            199902 => ClassSpec::Earthfort,
            1930 | 1931 | 1934 | 1935 => ClassSpec::Block,

            220112 | 2203622 => ClassSpec::Falconry,
            2292 | 1700820 | 1700825 | 1700827 => ClassSpec::Wildpack,

            2406 => ClassSpec::Recovery,
            2405 => ClassSpec::Shield,

            2306 => ClassSpec::Dissonance,
            2307 | 2361 | 55302 => ClassSpec::Concerto,
            _ => ClassSpec::Unknown,
        }
    }

    pub fn get_class_id_from_spec(class_spec: ClassSpec) -> i32 {
        match class_spec {
            ClassSpec::Iaido | ClassSpec::Moonstrike => STORMBLADE,
            ClassSpec::Icicle | ClassSpec::Frostbeam => FROST_MAGE,
            ClassSpec::Vanguard | ClassSpec::Skyward => WIND_KNIGHT,
            ClassSpec::Smite | ClassSpec::Lifebind => VERDANT_ORACLE,
            ClassSpec::Earthfort | ClassSpec::Block => HEAVY_GUARDIAN,
            ClassSpec::Wildpack | ClassSpec::Falconry => MARKSMAN,
            ClassSpec::Recovery | ClassSpec::Shield => SHIELD_KNIGHT,
            ClassSpec::Dissonance | ClassSpec::Concerto => BEAT_PERFORMER,
            ClassSpec::Unknown => UNKNOWN,
        }
    }

    pub fn get_class_spec(class_spec: ClassSpec) -> String {
        String::from(match class_spec {
            ClassSpec::Unknown => "",
            ClassSpec::Iaido => "Iaido",
            ClassSpec::Moonstrike => "Moonstrike",
            ClassSpec::Icicle => "Icicle",
            ClassSpec::Frostbeam => "Frostbeam",
            ClassSpec::Vanguard => "Vanguard",
            ClassSpec::Skyward => "Skyward",
            ClassSpec::Smite => "Smite",
            ClassSpec::Lifebind => "Lifebind",
            ClassSpec::Earthfort => "Earthfort",
            ClassSpec::Block => "Block",
            ClassSpec::Wildpack => "Wildpack",
            ClassSpec::Falconry => "Falconry",
            ClassSpec::Recovery => "Recovery",
            ClassSpec::Shield => "Shield",
            ClassSpec::Dissonance => "Dissonance",
            ClassSpec::Concerto => "Concerto",
        })
    }
}
