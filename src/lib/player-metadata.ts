import { invoke as TAURI_INVOKE } from '@tauri-apps/api/core';

export type PlayerMetadata = { name: string; class: string | null; class_spec: string | null; ability_score: number | null };

export async function getPlayerMetadata(playerUid: number): Promise<PlayerMetadata | null> {
  try {
  const res = await TAURI_INVOKE('get_player_metadata', { playerUid });
    // The Rust command returns Option<PlayerMetadata> which maps to null | object
    return res as PlayerMetadata | null;
  } catch (e) {
    console.warn('getPlayerMetadata invoke failed for', playerUid, e);
    return null;
  }
}
