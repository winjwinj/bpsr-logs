use crate::live::opcodes_models::class::{Class, ClassSpec};
use crate::protocol::pb;
use crate::protocol::pb::{EEntityType, SyncContainerData};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

pub type EncounterMutex = Mutex<Encounter>;

#[derive(Debug, Default, Clone)]
pub struct Encounter {
    pub is_encounter_paused: bool,
    pub time_last_combat_packet_ms: u128,
    pub time_fight_start_ms: u128,
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
    pub monster_id: Option<u32>,
    pub curr_hp: Option<u64>, // also available for players in packets
    pub max_hp: Option<u64>,  // also available for players in packets
    pub monster_pos: pb::Vector3,
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

static SKILL_NAMES: LazyLock<HashMap<i32, String>> = LazyLock::new(|| {
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

pub static MONSTER_NAMES_BOSS: LazyLock<HashMap<u32, String>> = LazyLock::new(|| {
    let data = include_str!("../../../src/lib/data/json/MonsterNameBoss.json");
    serde_json::from_str(data).expect("invalid MonsterName.json")
});

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
            2405 => ClassSpec::Recovery,
            2406 => ClassSpec::Shield,
            2306 => ClassSpec::Dissonance,
            2307 | 2361 | 55302 => ClassSpec::Concerto,
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
