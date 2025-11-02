<script lang="ts">
  import { onMount } from "svelte";
  import { commands, type SkillsWindow } from "$lib/bindings";
  import { getClassColor } from "$lib/utils.svelte";
  import { page } from "$app/state";
  import { goto } from "$app/navigation";
  import { createSvelteTable, FlexRender } from "$lib/svelte-table";
  import { dpsPlayersColumnDefs, dpsSkillsColumnDefs } from "$lib/table-info";
  import { getCoreRowModel } from "@tanstack/table-core";
  import { SETTINGS } from "$lib/settings-store";
  import { selectedEncounter, liveFallbackEncounterId } from "$lib/encounter-store";

  let playerUid = $derived(page.url.searchParams.get("playerUid") ?? "-1");
  // Optional forced encounter id passed via query when navigating from a historical fallback
  let forcedEncounterId = $derived(page.url.searchParams.get("encounterId") ?? undefined);

  let unsubscribe: (() => void) | null = null;
  let currentEncounterId: number | undefined = $state(undefined);
  let currentEncounterType: "live" | "historical" = $state("live");
  // avoid repeated fallback DB loads
  let lastFallbackEncounterId: number | null = $state(null);
  let fallbackUnsub: (() => void) | null = null;

  onMount(() => {
    unsubscribe = selectedEncounter.subscribe((value) => {
      currentEncounterType = value.type;
      currentEncounterId = value.encounterId;
      // Reset local fallback tracking only; do NOT clear the global fallback store here.
      lastFallbackEncounterId = null;
    });

    fallbackUnsub = liveFallbackEncounterId.subscribe((v) => {
      lastFallbackEncounterId = v;
    });

    // Only poll for live data, not historical
    const interval = setInterval(() => {
      if (currentEncounterType === "live") {
        fetchData();
      }
    }, 200);

    return () => {
      clearInterval(interval);
      unsubscribe?.();
      fallbackUnsub?.();
    };
  });

  let dpsSkillBreakdownWindow: SkillsWindow | undefined = $state(undefined);

  $effect(() => {
    // Establish dependencies by accessing them first
    const type = currentEncounterType;
    const encId = currentEncounterId;
    const pUid = playerUid;
    
    // Fetch when encounter changes in any mode
    console.log("DPS Skills effect triggered:", { playerUid: pUid, currentEncounterId: encId, type });
    fetchData();
  });

  async function fetchData() {
    try {
      console.log("DPS fetchData called:", { currentEncounterType, currentEncounterId, playerUid });
  // Prefer an explicit forced encounter id from the URL, otherwise fall back to the
  // shared live fallback id (if any).
  const forcedId = forcedEncounterId ? parseInt(forcedEncounterId) : (lastFallbackEncounterId ?? undefined);
      if (currentEncounterType === "historical" && currentEncounterId) {
        // Explicitly selected historical encounter
        const playerId = parseInt(playerUid);
        console.log("Fetching historical DPS skills:", { encounterId: currentEncounterId, playerId });
        const result = await commands.getHistoricalSkillsWindow(currentEncounterId, playerId);
        console.log("API result:", result);
        if (result.status !== "ok") {
          console.error("Failed to get historical skills window:", result.error);
          console.log("Player not found in encounter, navigating back to /live/dps");
          await goto("/live/dps");
          return;
        }
        dpsSkillBreakdownWindow = result.data;
      } else {
        const result = SETTINGS.misc.state.testingMode ? await commands.getTestSkillWindow(playerUid) : await commands.getDpsSkillWindow(playerUid);
        if (result.status !== "ok") {
          console.warn("Failed to get skill window: ", result.error);
          return;
        }

  const fetched = result.data;
  // Debug: log fetched sizes so we can compare live flow with Heal
  console.debug('DPS fetched:', { inspectedTotal: fetched.inspectedPlayer?.totalValue, skillRows: fetched.skillRows?.length, topValue: fetched.topValue });
        const fetchedHasAnyValue = (fetched.inspectedPlayer?.totalValue ?? 0) > 0 || (fetched.topValue ?? 0) > 0 || (fetched.skillRows ?? []).some(r => (r.totalValue ?? 0) > 0);
        const currentHasAnyValue = dpsSkillBreakdownWindow !== undefined && ((dpsSkillBreakdownWindow.inspectedPlayer?.totalValue ?? 0) > 0 || (dpsSkillBreakdownWindow.topValue ?? 0) > 0 || (dpsSkillBreakdownWindow.skillRows ?? []).some(r => (r.totalValue ?? 0) > 0));

        // If live returned no skill data for this player and we're in live mode, prefer a
        // historical fallback (if available) so navigation to the skills page shows the
        // last saved encounter instead of empty live results.
        if (!fetchedHasAnyValue && currentEncounterType === "live" && lastFallbackEncounterId && lastFallbackEncounterId > 0) {
          try {
            const playerId = parseInt(playerUid);
            const histRes = await commands.getHistoricalSkillsWindow(lastFallbackEncounterId, playerId);
            if (histRes.status === 'ok') {
              dpsSkillBreakdownWindow = histRes.data;
              return;
            }
          } catch (err) {
            console.error('Failed to load historical DPS skills for fallback id:', lastFallbackEncounterId, err);
          }
        }

        if (fetchedHasAnyValue || !currentHasAnyValue) {
          // Live has real data or we have nothing — use fetched
          dpsSkillBreakdownWindow = fetched;
          if (fetchedHasAnyValue) { lastFallbackEncounterId = null; liveFallbackEncounterId.set(null); }
        } else {
          // Live returned empty and we already have a view — try forced historical fallback first (if any), else generic one-time fallback
            if (forcedId) {
            try {
              const playerId = parseInt(playerUid);
              const histRes = await commands.getHistoricalSkillsWindow(forcedId, playerId);
              if (histRes.status === 'ok') {
                dpsSkillBreakdownWindow = histRes.data;
                  lastFallbackEncounterId = forcedId;
                  liveFallbackEncounterId.set(forcedId);
              }
            } catch (err) {
              console.error('Failed to load forced historical DPS skills as fallback:', err);
            }
          }

          if (dpsSkillBreakdownWindow === undefined && lastFallbackEncounterId === null) {
            try {
              const historyRes = await commands.getAllEncounterHistory();
              if (historyRes.status === 'ok' && historyRes.data && historyRes.data.length > 0) {
                const lastEncounter = historyRes.data[0]!;
                const playerId = parseInt(playerUid);
                const histRes = await commands.getHistoricalSkillsWindow(lastEncounter.id, playerId);
                if (histRes.status === 'ok') {
                  dpsSkillBreakdownWindow = histRes.data;
                  lastFallbackEncounterId = lastEncounter.id ?? -1;
                  liveFallbackEncounterId.set(lastFallbackEncounterId);
                } else {
                  lastFallbackEncounterId = -1;
                  liveFallbackEncounterId.set(-1);
                }
              } else {
                lastFallbackEncounterId = -1;
                liveFallbackEncounterId.set(-1);
              }
            } catch (err) {
              console.error('Failed to load last historical DPS skills as fallback:', err);
              lastFallbackEncounterId = -1;
              liveFallbackEncounterId.set(-1);
            }
          }

          // ensure we have a valid structure to render
          if (dpsSkillBreakdownWindow === undefined) {
            dpsSkillBreakdownWindow = fetched;
          }
        }
      }
    } catch (e) {
      console.error("Error fetching data: ", e);
    }
  }

  const inspectedPlayerTable = createSvelteTable({
    get data() {
      if (dpsSkillBreakdownWindow !== undefined) {
        return [dpsSkillBreakdownWindow.inspectedPlayer];
      } else {
        return [];
      }
    },
    columns: dpsPlayersColumnDefs,
    getCoreRowModel: getCoreRowModel(),
    state: {
      get columnVisibility() {
        return SETTINGS.live.dps.skillBreakdown.state;
      },
    },
  });

  const dpsSkillBreakdownTable = createSvelteTable({
    get data() {
      if (dpsSkillBreakdownWindow !== undefined) {
        return dpsSkillBreakdownWindow.skillRows;
      } else {
        return [];
      }
    },
    columns: dpsSkillsColumnDefs,
    getCoreRowModel: getCoreRowModel(),
    state: {
      get columnVisibility() {
        return SETTINGS.live.dps.skillBreakdown.state;
      },
    },
  });

</script>

<svelte:window oncontextmenu={() => window.history.back()} />

{#if dpsSkillBreakdownWindow !== undefined}
  <div class="relative flex flex-col">
    <table class="w-screen table-fixed">
      <thead class="z-1 sticky top-0 h-6">
        <tr class="bg-neutral-900">
          {#each dpsSkillBreakdownTable.getHeaderGroups() as headerGroup (headerGroup.id)}
            {#each headerGroup.headers as header (header.id)}
              <th class={header.column.columnDef.meta?.class}><FlexRender content={header.column.columnDef.header ?? "UNKNOWN HEADER"} context={header.getContext()} /></th>
            {/each}
          {/each}
        </tr>
      </thead>
      <tbody>
        {#each inspectedPlayerTable.getRowModel().rows as row, rowIndex (currentEncounterType === "historical" ? currentEncounterId + "-" + rowIndex : rowIndex)}
          <tr class="h-7 px-2 py-1 text-center">
            {#each row.getVisibleCells() as cell (cell.id)}
              <td><FlexRender content={cell.column.columnDef.cell ?? "UNKNOWN CELL"} context={cell.getContext()} /></td>
            {/each}
            <td class="-z-1 absolute left-0 h-7" style="background-color: {getClassColor(dpsSkillBreakdownWindow.inspectedPlayer.className)}; width: 100vw;"></td>
          </tr>
        {/each}
        {#each dpsSkillBreakdownTable.getRowModel().rows as row, i (currentEncounterType === "historical" ? currentEncounterId + "-" + i : i)}
          <tr class="h-7 px-2 py-1 text-center">
            {#each row.getVisibleCells() as cell (cell.id)}
              <td><FlexRender content={cell.column.columnDef.cell ?? "UNKNOWN CELL"} context={cell.getContext()} /></td>
            {/each}
            <td class="-z-1 absolute left-0 h-7" style="background-color: {`color-mix(in srgb, ${getClassColor(dpsSkillBreakdownWindow.inspectedPlayer.className)} 80%, white ${i % 2 === 0 ? '50%' : '20%'})`}; width: {(row.original.totalValue / dpsSkillBreakdownWindow.topValue) * 100}%;"></td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/if}
