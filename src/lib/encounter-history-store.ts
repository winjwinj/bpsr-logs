import { derived, writable } from 'svelte/store';
import { RuneStore } from '@tauri-store/svelte';
import { SETTINGS } from '$lib/settings-store';

// Stored as array of { id, timestamp, data }
const ENCOUNTER_STORE_KEY = 'encounterHistory';

const DEFAULT: any = {
  encounters: [],
  selectedId: 'current',
};

export const ENCOUNTER_STORE = new RuneStore(ENCOUNTER_STORE_KEY, DEFAULT, { autoStart: true, saveOnChange: true });

// Use a Svelte writable for correct typing and reactivity.
// RuneStore may initialize its `.state` asynchronously on startup, so read from it when available
// and keep the two in sync both directions.
const internal = writable(DEFAULT);

// When RuneStore already has state (synchronous), initialize internal from it.
try {
  const s = (ENCOUNTER_STORE as any).state as any;
  if (s) internal.set(s);
} catch (e) {
  // ignore
}

// Persist back to RuneStore.state whenever internal changes
internal.subscribe((v) => {
  try {
    // Only set when v is defined to avoid stomping on RuneStore during its own init
    if (v) (ENCOUNTER_STORE as any).state = v;
  } catch (e) {
    console.warn('Failed to persist encounter store to RuneStore', e);
  }
});

// If RuneStore updates its state later (async load), attempt to read it and apply into internal.
// RuneStore doesn't expose a subscribe API here, so poll briefly on startup to catch its
// loaded state. This keeps saved encounters visible in the UI.
(async () => {
  try {
    // Try a few times over 500ms for RuneStore to populate
    for (let i = 0; i < 10; i++) {
      const s = (ENCOUNTER_STORE as any).state as any;
      if (s && Array.isArray(s.encounters) && s.encounters.length > 0) {
        internal.set(s);
        break;
      }
      // small delay
      await new Promise((r) => setTimeout(r, 50));
    }
  } catch (e) {
    // ignore
  }
})();

export const encounters = derived(internal, ($s) => ($s?.encounters) || []);
export const selectedEncounterId = derived(internal, ($s) => $s?.selectedId ?? 'current');

export const selectedEncounter = derived([encounters, selectedEncounterId], ([$encs, $selId]) => {
  if ($selId === 'current') return { mode: 'current', data: null };
  const found = ($encs || []).find((e: any) => e.id === $selId);
  if (found) return { mode: 'history', data: found.data, timestamp: found.timestamp, id: found.id };
  return { mode: 'current', data: null };
});

function makeId() {
  return `${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;
}

export function pushEncounter(snapshot: any) {
  try {
    const id = makeId();
    const timestamp = Date.now();
    const item = { id, timestamp, data: snapshot };
    internal.update((s: any) => {
      const max = Number(SETTINGS.history.state.maxEncounters || 10);
      const arr = (s?.encounters || []).slice();
      arr.unshift(item);
      if (arr.length > max) arr.splice(max);
      return { ...s, encounters: arr };
    });
    return id;
  } catch (e) {
    console.warn('pushEncounter error', e);
    return null;
  }
}

export function selectEncounter(id: string) {
  internal.update((s: any) => ({ ...s, selectedId: id }));
}

export function clearHistory() {
  internal.update((s: any) => ({ ...s, encounters: [] }));
}

export default internal;
