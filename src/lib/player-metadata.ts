import { commands, type PlayerMetadata } from '$lib/bindings';

export async function getPlayerMetadata(playerUid: number): Promise<PlayerMetadata | null> {
  try {
    const result = await commands.getPlayerMetadata(playerUid);
    if (result.status === 'ok') {
      return result.data;
    } else {
      console.warn('getPlayerMetadata failed for', playerUid, result.error);
      return null;
    }
  } catch (e) {
    console.warn('getPlayerMetadata invoke failed for', playerUid, e);
    return null;
  }
}

export async function updateLivePlayerMetadata(
  playerUid: number,
  name?: string | null,
  class_name?: string | null,
  class_spec?: string | null,
  ability_score?: number | null
): Promise<boolean> {
  try {
    // Update the live encounter cache
    const result = await commands.updatePlayerMetadata(playerUid, name ?? null, class_name ?? null, class_spec ?? null, ability_score ?? null);
    if (result.status !== 'ok') {
      console.warn(`Failed to update player ${playerUid} metadata:`, result.error);
      return false;
    }
    
    console.log(`Updated player ${playerUid} metadata in live encounter`);
    
    // Also persist to database immediately to avoid losing metadata on crash/reset
    const persistResult = await commands.persistPlayerMetadata(playerUid, name ?? null, class_name ?? null, class_spec ?? null, ability_score ?? null);
    if (persistResult.status !== 'ok') {
      console.warn(`Failed to persist player ${playerUid} metadata:`, persistResult.error);
      // Don't fail though - at least the live data is updated
    } else {
      console.log(`Persisted player ${playerUid} metadata to database`);
    }
    
    return true;
  } catch (e) {
    console.warn(`updateLivePlayerMetadata invoke failed for player ${playerUid}:`, e);
    return false;
  }
}

export async function persistPlayerMetadataDirectly(
  playerUid: number,
  name?: string | null,
  class_name?: string | null,
  class_spec?: string | null,
  ability_score?: number | null
): Promise<boolean> {
  try {
    const result = await commands.persistPlayerMetadata(playerUid, name ?? null, class_name ?? null, class_spec ?? null, ability_score ?? null);
    if (result.status === 'ok') {
      console.log(`Persisted player ${playerUid} metadata to database`);
      return true;
    } else {
      console.warn(`Failed to persist player ${playerUid} metadata:`, result.error);
      return false;
    }
  } catch (e) {
    console.warn(`persistPlayerMetadataDirectly invoke failed for player ${playerUid}:`, e);
    return false;
  }
}
