use crate::live::opcodes_models::class::{Class, ClassSpec};
use blueprotobuf_lib::blueprotobuf::{EEntityType, SyncContainerData};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

pub type EncounterMutex = Mutex<Encounter>;

#[derive(Debug, Default, Clone)]
pub struct Encounter {
    pub is_encounter_paused: bool,
    pub time_last_combat_packet_ms: u128,
    pub time_fight_start_ms: u128,
    pub local_player_uid: Option<i64>,
    pub entity_uid_to_entity: HashMap<i64, Entity>,
    pub dmg_stats: CombatStats,
    pub dmg_stats_boss_only: CombatStats,
    pub heal_stats: CombatStats,
    pub local_player: Option<SyncContainerData>,
}

#[derive(Debug, Default, Clone)]
pub struct Entity {
    pub entity_type: EEntityType,

    pub dmg_stats: CombatStats,
    pub skill_uid_to_dps_stats: HashMap<i32, CombatStats>,

    pub dmg_stats_boss_only: CombatStats,
    pub skill_uid_to_dps_stats_boss_only: HashMap<i32, CombatStats>,

    pub heal_stats: CombatStats,
    pub skill_uid_to_heal_stats: HashMap<i32, CombatStats>,


    // Players
    pub name: Option<String>, // also available for monsters in packets
    pub class: Option<Class>,
    pub class_spec: Option<ClassSpec>,
    pub ability_score: Option<i32>,

    // Monsters
    pub monster_id: Option<i32>,
    pub curr_hp: Option<i32>, // also available for players in packets
    pub max_hp: Option<i32>, // also available for players in packets
}

#[derive(Debug, Default, Clone)]
pub struct CombatStats {
    pub value: i64,
    pub hits: i64,
    pub crit_value: i64,
    pub crit_hits: i64,
    pub lucky_value: i64,
    pub lucky_hits: i64,
}

static SKILL_NAMES: Lazy<HashMap<i32, String>> = Lazy::new(|| {
    let data = include_str!("../../../src/lib/data/json/SkillName.json");
    serde_json::from_str(data).expect("invalid SkillName.json")
});

impl CombatStats {
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

pub static MONSTER_NAMES_BOSS: Lazy<HashMap<i32, String>> = Lazy::new(|| {
    let data = include_str!("../../../src/lib/data/json/MonsterNameBoss.json");
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
    // pub const ATTR_LEVEL: i32 = 0x2710;
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

    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    #[repr(i32)]
    pub enum Class {
        Stormblade,
        FrostMage,
        WindKnight,
        VerdantOracle,
        HeavyGuardian,
        Marksman,
        ShieldKnight,
        BeatPerformer,
        Unimplemented,
        #[default]
        Unknown,
    }

    impl From<i32> for Class {
        fn from(class_id: i32) -> Self {
            match class_id {
                1 => Class::Stormblade,
                2 => Class::FrostMage,
                4 => Class::WindKnight,
                5 => Class::VerdantOracle,
                9 => Class::HeavyGuardian,
                11 => Class::Marksman,
                12 => Class::ShieldKnight,
                13 => Class::BeatPerformer,
                _ => Class::Unimplemented,
            }
        }
    }

    pub fn get_class_name(class: Class) -> String {
        String::from(match class {
            Class::Stormblade => "Stormblade",
            Class::FrostMage => "Frost Mage",
            Class::WindKnight => "Wind Knight",
            Class::VerdantOracle => "Verdant Oracle",
            Class::HeavyGuardian => "Heavy Guardian",
            Class::Marksman => "Marksman",
            Class::ShieldKnight => "Shield Knight",
            Class::BeatPerformer => "Beat Performer",
            Class::Unknown => "Unknown Class",
            Class::Unimplemented => "Unimplemented Class",
        })
    }

    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub enum ClassSpec {
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
        #[default]
        Unknown,
    }

    pub fn get_class_spec_from_skill_id(skill_id: i32) -> ClassSpec {
        match skill_id {
            1714 | 1734 => ClassSpec::Iaido, // Iaido Slash, Thunder Iaido Slash
            44701 | 179_906 => ClassSpec::Moonstrike, // AI: Moon Blade, Moonstrike Whirl

            120_901 | 120_902 => ClassSpec::Icicle, // AI: Through the ice spear, AI: Ice spear
            1241 => ClassSpec::Frostbeam, // Frostbeam

            1405 | 1418 => ClassSpec::Vanguard, // Gale Thrust, Gale Thrust
            1419 => ClassSpec::Skyward, // Skyfall

            1518 | 1541 | 21402 => ClassSpec::Smite, // Wild Bloom, Wild Bloom, AI: Blooming wildly
            20301 => ClassSpec::Lifebind, // AI: Life blooms

            199_902 => ClassSpec::Earthfort, // Rage Burst Stone
            1930 | 1931 | 1934 | 1935 => ClassSpec::Block, // Countercrush, Countercrush, Countercrush, Countercrush

            220_112 | 2_203_622 => ClassSpec::Falconry, // AI: Photoelectric cracks, AI: Light lip sputtering
            2292 | 1_700_820 | 1_700_825 | 1_700_827 => ClassSpec::Wildpack, // Phantom Direwolves, Wild Wolf - Assist, Pet - Foxen Pounce, Pet - Basic Attack

            2405 => ClassSpec::Recovery, // Valor Bash
            2406 => ClassSpec::Shield, // Vanguard Strike

            2306 => ClassSpec::Dissonance, // Amplified Beat
            2307 | 2361 | 55302 => ClassSpec::Concerto, // Healing Beat, Healing Beat copy, AI: Healing beat
            _ => ClassSpec::Unknown,
        }
    }

    pub fn get_class_from_spec(class_spec: ClassSpec) -> Class {
        match class_spec {
            ClassSpec::Iaido | ClassSpec::Moonstrike => Class::Stormblade,
            ClassSpec::Icicle | ClassSpec::Frostbeam => Class::FrostMage,
            ClassSpec::Vanguard | ClassSpec::Skyward => Class::WindKnight,
            ClassSpec::Smite | ClassSpec::Lifebind => Class::VerdantOracle,
            ClassSpec::Earthfort | ClassSpec::Block => Class::HeavyGuardian,
            ClassSpec::Wildpack | ClassSpec::Falconry => Class::Marksman,
            ClassSpec::Recovery | ClassSpec::Shield => Class::ShieldKnight,
            ClassSpec::Dissonance | ClassSpec::Concerto => Class::BeatPerformer,
            ClassSpec::Unknown => Class::Unknown,
        }
    }

    // TODO: is there a way to just do this automatically based on the name of the enum?
    pub fn get_class_spec(class_spec: ClassSpec) -> String {
        String::from(match class_spec {
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
            ClassSpec::Unknown => "Unknown Spec",
        })
    }
}
