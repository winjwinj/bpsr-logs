<script lang="ts">
  import { onMount } from "svelte";
  import { commands, type SkillsWindow } from "$lib/bindings";
  import { getClassColor } from "$lib/utils.svelte";
  import { page } from "$app/state";
  import { goto } from "$app/navigation";
  import { createSvelteTable, FlexRender } from "$lib/svelte-table";
  import { healPlayersColumnDefs, healSkillsColumnDefs } from "$lib/table-info";
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

  let healSkillBreakdownWindow: SkillsWindow = $state({ inspectedPlayer: { uid: 0, abilityScore: 0, className: "", classSpecName: "", name: "", totalValue: 0, valuePerSec: 0, valuePct: 0, critRate: 0, critValueRate: 0, luckyRate: 0, luckyValueRate: 0, hits: 0, hitsPerMinute: 0 }, skillRows: [], localPlayerUid: 0, topValue: 0 });

  $effect(() => {
    // Establish dependencies by accessing them first
    const type = currentEncounterType;
    const encId = currentEncounterId;
    const pUid = playerUid;
    
    // Fetch when encounter changes in any mode
    console.log("Heal Skills effect triggered:", { playerUid: pUid, currentEncounterId: encId, type });
    fetchData();
  });

  async function fetchData() {
    try {
      console.log("Heal fetchData called:", { currentEncounterType, currentEncounterId, playerUid });
  const forcedId = forcedEncounterId ? parseInt(forcedEncounterId) : (lastFallbackEncounterId ?? undefined);
      if (currentEncounterType === "historical" && currentEncounterId) {
        const playerId = parseInt(playerUid);
        console.log("Fetching historical Heal skills:", { encounterId: currentEncounterId, playerId });
        const result = await commands.getHistoricalSkillsWindow(currentEncounterId, playerId);
        if (result.status !== "ok") {
          console.error("Failed to get historical skills window:", result.error);
          console.log("Player not found in encounter, navigating back to /live/heal");
          await goto("/live/heal");
          return;
        }
        healSkillBreakdownWindow = result.data;
      } else {
        const result = SETTINGS.misc.state.testingMode ? await commands.getTestSkillWindow(playerUid) : await commands.getHealSkillWindow(playerUid);
        if (result.status !== "ok") {
          console.warn("Failed to get skill window: ", result.error);
          return;
        } else {
          const fetched = result.data;
            // Debug: log fetched sizes so we can compare live flow with DPS
            console.debug('Heal fetched:', { inspectedTotal: fetched.inspectedPlayer?.totalValue, skillRows: fetched.skillRows?.length, topValue: fetched.topValue });

          const fetchedHasAnyValue = (fetched.inspectedPlayer?.totalValue ?? 0) > 0 || (fetched.topValue ?? 0) > 0 || (fetched.skillRows ?? []).some(r => (r.totalValue ?? 0) > 0);
          const currentHasAnyValue = ((healSkillBreakdownWindow.inspectedPlayer?.totalValue ?? 0) > 0) || ((healSkillBreakdownWindow.topValue ?? 0) > 0) || ((healSkillBreakdownWindow.skillRows ?? []).some(r => (r.totalValue ?? 0) > 0));

          // If live returned no skill data and we have a fallback encounter id, use the
          // historical skills window so navigation shows the last saved encounter.
          if (!fetchedHasAnyValue && currentEncounterType === "live" && lastFallbackEncounterId && lastFallbackEncounterId > 0) {
            try {
              const playerId = parseInt(playerUid);
              const histRes = await commands.getHistoricalSkillsWindow(lastFallbackEncounterId, playerId);
              if (histRes.status === 'ok') {
                healSkillBreakdownWindow = histRes.data;
                return;
              }
            } catch (err) {
              console.error('Failed to load historical Heal skills for fallback id:', lastFallbackEncounterId, err);
            }
          }

          if (fetchedHasAnyValue || !currentHasAnyValue) {
            healSkillBreakdownWindow = fetched;
            if (fetchedHasAnyValue) { lastFallbackEncounterId = null; liveFallbackEncounterId.set(null); }
          } else {
            if (forcedId) {
              try {
                const playerId = parseInt(playerUid);
                const histRes = await commands.getHistoricalSkillsWindow(forcedId, playerId);
                if (histRes.status === 'ok') {
                  healSkillBreakdownWindow = histRes.data;
                  lastFallbackEncounterId = forcedId;
                  liveFallbackEncounterId.set(forcedId);
                }
              } catch (err) {
                console.error('Failed to load forced historical Heal skills as fallback:', err);
              }
            }

            if (healSkillBreakdownWindow === undefined && lastFallbackEncounterId === null) {
              try {
                const historyRes = await commands.getAllEncounterHistory();
                if (historyRes.status === 'ok' && historyRes.data && historyRes.data.length > 0) {
                  const lastEncounter = historyRes.data[0]!;
                  const playerId = parseInt(playerUid);
                  const histRes = await commands.getHistoricalSkillsWindow(lastEncounter.id, playerId);
                  if (histRes.status === 'ok') {
                    healSkillBreakdownWindow = histRes.data;
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
                console.error('Failed to load last historical Heal skills as fallback:', err);
                lastFallbackEncounterId = -1;
                liveFallbackEncounterId.set(-1);
              }
            }

            if (!healSkillBreakdownWindow || healSkillBreakdownWindow.skillRows.length === 0) {
              healSkillBreakdownWindow = fetched;
            }
          }
        }
      }
    } catch (e) {
      console.error("Error fetching data: ", e);
    }
  }

  const currPlayerTable = createSvelteTable({
    get data() {
      return [healSkillBreakdownWindow.inspectedPlayer];
    },
    columns: healPlayersColumnDefs,
    getCoreRowModel: getCoreRowModel(),
    state: {
      get columnVisibility() {
        return SETTINGS.live.heal.skillBreakdown.state;
      },
    },
  });

  const healSkillBreakdownTable = createSvelteTable({
    get data() {
      return healSkillBreakdownWindow.skillRows;
    },
    columns: healSkillsColumnDefs,
    getCoreRowModel: getCoreRowModel(),
    state: {
      get columnVisibility() {
        return SETTINGS.live.heal.skillBreakdown.state;
      },
    },
  });

  let SETTINGS_YOUR_NAME = $derived(SETTINGS.general.state.showYourName);
  let SETTINGS_OTHERS_NAME = $derived(SETTINGS.general.state.showOthersName);
</script>

<svelte:window oncontextmenu={() => window.history.back()} />

<!-- TODO: looks ugly when split, need to figure out logic to combine together https://imgur.com/COalJFe -->
<div class="relative flex flex-col">
  <table class="w-screen table-fixed">
    <thead class="z-1 sticky top-0 h-6">
      <tr class="bg-neutral-900">
        {#each healSkillBreakdownTable.getHeaderGroups() as headerGroup (headerGroup.id)}
          {#each headerGroup.headers as header (header.id)}
            <th class={header.column.columnDef.meta?.class}><FlexRender content={header.column.columnDef.header ?? "UNKNOWN HEADER"} context={header.getContext()} /></th>
          {/each}
        {/each}
      </tr>
    </thead>
    <tbody>
      {#each currPlayerTable.getRowModel().rows as row, rowIndex (currentEncounterType === "historical" ? currentEncounterId + "-" + rowIndex : rowIndex)}
        {@const currPlayer = healSkillBreakdownWindow.inspectedPlayer}
        {#if currPlayer}
          {@const className = row.original.name.includes("You") ? (SETTINGS_YOUR_NAME !== "Hide Your Name" ? currPlayer.className : "") : SETTINGS_OTHERS_NAME !== "Hide Others' Name" ? currPlayer.className : ""}
          <tr class="h-7 px-2 py-1 text-center">
            {#each row.getVisibleCells() as cell (cell.id)}
              <td><FlexRender content={cell.column.columnDef.cell ?? "UNKNOWN CELL"} context={cell.getContext()} /></td>
            {/each}
            <td class="-z-1 absolute left-0 h-7" style="background-color: {getClassColor(className)}; width: 100vw;"></td>
          </tr>
        {/if}
      {/each}
      {#each healSkillBreakdownTable.getRowModel().rows as row, i (currentEncounterType === "historical" ? currentEncounterId + "-" + i : i)}
        {@const currPlayer = healSkillBreakdownWindow.inspectedPlayer}
        {#if currPlayer}
          {@const className = row.original.name.includes("You") ? (SETTINGS_YOUR_NAME !== "Hide Your Name" ? currPlayer.className : "") : SETTINGS_OTHERS_NAME !== "Hide Others' Name" ? currPlayer.className : ""}
          <tr class="h-7 px-2 py-1 text-center">
            {#each row.getVisibleCells() as cell (cell.id)}
              <td><FlexRender content={cell.column.columnDef.cell ?? "UNKNOWN CELL"} context={cell.getContext()} /></td>
            {/each}
            <td class="-z-1 absolute left-0 h-7" style="background-color: {`color-mix(in srgb, ${getClassColor(className)} 80%, white ${i % 2 === 0 ? '50%' : '20%'})`}; width: {healSkillBreakdownWindow.topValue > 0 ? (row.original.totalValue / healSkillBreakdownWindow.topValue) * 100 : 0}%;"></td>
          </tr>
        {/if}
      {/each}
    </tbody>
  </table>
</div>
