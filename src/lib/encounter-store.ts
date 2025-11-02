import { writable, type Writable } from "svelte/store";

export interface SelectedEncounter {
  type: "live" | "historical";
  encounterId?: number; // Only for historical
}

const STORAGE_KEY = "selectedEncounter";

function getInitialState(): SelectedEncounter {
  // Try to load from localStorage if available
  if (typeof window !== "undefined") {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      try {
        return JSON.parse(stored);
      } catch (e) {
        console.warn("Failed to parse stored selectedEncounter:", e);
      }
    }
  }
  return { type: "live" };
}

export const selectedEncounter: Writable<SelectedEncounter> = writable(getInitialState());

// Persist to localStorage whenever it changes
selectedEncounter.subscribe((value) => {
  if (typeof window !== "undefined") {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(value));
  }
});

// When the live view falls back to the most recent saved encounter (because there's no current
// combat), we keep that encounter id here so other live pages (DPS/Heal/Skills) can use the
// same historical data without switching the user's selectedEncounter to `historical`.
// This store is intentionally separate from `selectedEncounter` and is not persisted long-term.
export const liveFallbackEncounterId: Writable<number | null> = writable(null);
