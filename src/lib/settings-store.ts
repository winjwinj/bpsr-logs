import { version } from '@tauri-apps/plugin-os';
import { RuneStore } from '@tauri-store/svelte';

const IS_WIN_11 = parseInt(version().split(".")[2] || "0", 10) >= 22000;

export const DEFAULT_STATS = {
  totalValue: true,
  valuePerSec: true,
  valuePct: true,
  critRate: true,
  critValueRate: true,
  luckyRate: false,
  luckyValueRate: false,
  hits: false,
  hitsPerMinute: false,
};

const DEFAULT_SETTINGS = {
  general: {
    showYourName: "Show Your Name", // ["Show Your Name", "Show Your Class", "Hide Your Name"]
    showOthersName: "Show Others' Name", // ["Show Others' Name", "Show Others' Class", "Hide Others' Name"]
    showYourAbilityScore: true,
    showOthersAbilityScore: true,
    resetElapsed: 60,
    shortenAbilityScore: false,
    bossOnly: false,
    autostart: true,
  },
  accessibility: {
    blur: !IS_WIN_11,
    transparency: false,
  },
  shortcuts: {
    showLiveMeter: "",
    hideLiveMeter: "",
    toggleLiveMeter: "",
    showDpsTab: "",
    showHealTab: "",
    enableClickthrough: "",
    disableClickthrough: "",
    toggleClickthrough: "",
    resetEncounter: "",
    hardReset: "",
    markCurrentMonsterDead: "",
  },
  live: {
    dpsPlayers: { ...DEFAULT_STATS },
    dpsSkillBreakdown: { ...DEFAULT_STATS },
    healPlayers: { ...DEFAULT_STATS },
    healSkillBreakdown: { ...DEFAULT_STATS },
  },
  misc: {
    testingMode: false,
  },
  integration: {
    bptimer: true,
    bptimerUI: true,
  }
};

// We need flattened settings for every update to be able to auto-detect new changes
const RUNE_STORE_OPTIONS = { autoStart: true, saveOnChange: true };
export const SETTINGS = {
  general: new RuneStore(
    'general',
    DEFAULT_SETTINGS.general,
    RUNE_STORE_OPTIONS
  ),
  accessibility: new RuneStore(
    'accessibility',
    DEFAULT_SETTINGS.accessibility,
    RUNE_STORE_OPTIONS
  ),
  shortcuts: new RuneStore(
    'shortcuts',
    DEFAULT_SETTINGS.shortcuts,
    RUNE_STORE_OPTIONS
  ),
  live: {
    dps: {
      players: new RuneStore(
        'liveDpsPlayers',
        DEFAULT_SETTINGS.live.dpsPlayers,
        RUNE_STORE_OPTIONS
      ),
      skillBreakdown: new RuneStore(
        'liveDpsSkillBreakdown',
        DEFAULT_SETTINGS.live.dpsSkillBreakdown,
        RUNE_STORE_OPTIONS
      ),
    },
    heal: {
      players: new RuneStore(
        'liveHealPlayers',
        DEFAULT_SETTINGS.live.healPlayers,
        RUNE_STORE_OPTIONS
      ),
      skillBreakdown: new RuneStore(
        'liveHealSkillBreakdown',
        DEFAULT_SETTINGS.live.healSkillBreakdown,
        RUNE_STORE_OPTIONS
      ),
    },
  },
  misc: new RuneStore(
    'misc',
    DEFAULT_SETTINGS.misc,
    RUNE_STORE_OPTIONS
  ),
  integration: new RuneStore(
    'integration',
    DEFAULT_SETTINGS.integration,
    RUNE_STORE_OPTIONS
  ),
};
