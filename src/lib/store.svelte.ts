import { RuneStore } from '@tauri-store/svelte';


interface SettingSwitchConfig {
  id: string;
  label: string;
  description: string;
}

// Define all your groups and their settings here
export const SETTINGS_CONFIG_LIST = {
  general: [
    { id: "foogeneral", label: "foogeneral label", description: "foogeneral description" },
    { id: "bargeneral", label: "bargeneral label", description: "bargeneral description" },
  ],
  live: [
    { id: "foolive", label: "foolive label", description: "foolive description" },
    { id: "barlive", label: "barlive label", description: "barlive description" },
  ],
} as const;

// Type to automatically extract the IDs from each group
type DefaultStores<T extends Record<string, readonly SettingSwitchConfig[]>> = {
  [K in keyof T]: {
    [S in T[K][number]['id']]: boolean
  }
}

// Dynamically generate default stores
const defaultStores = Object.fromEntries(
  Object.entries(SETTINGS_CONFIG_LIST).map(([groupName, groupSettings]) => [
    groupName,
    Object.fromEntries(groupSettings.map(s => [s.id, false]))
  ])
) as DefaultStores<typeof SETTINGS_CONFIG_LIST>;

// Create a single RuneStore with all groups
export const settings = new RuneStore('settings', defaultStores, {
  autoStart: true,
  saveOnChange: true,
  hooks: {
    beforeBackendSync: (state) => {
      console.log(`beforeBackendSync: ${state}`);
      return state;
    },
  },
});
