<script lang="ts">
  import { onMount } from "svelte";
  import { commands, type PlayersWindow } from "$lib/bindings";
  import { getClassColor } from "$lib/utils.svelte";
  import { goto } from "$app/navigation";
  import { getCoreRowModel } from "@tanstack/table-core";
  import { createSvelteTable } from "$lib/svelte-table";
  import { healPlayersColumnDefs } from "$lib/table-info";
  import FlexRender from "$lib/svelte-table/flex-render.svelte";
  import { SETTINGS } from "$lib/settings-store";
  import { selectedEncounter, liveFallbackEncounterId } from "$lib/encounter-store";

  let unsubscribe: (() => void) | null = null;
  let currentEncounterId: number | undefined = $state(undefined);
  let currentEncounterType: "live" | "historical" = $state("live");
  // Track whether we've already attempted to load a historical fallback for the current live session
  let lastFallbackEncounterId: number | null = $state(null);
  let fallbackUnsub: (() => void) | null = null;

  onMount(() => {
    unsubscribe = selectedEncounter.subscribe((value) => {
      currentEncounterType = value.type;
      currentEncounterId = value.encounterId;
      // Reset fallback when user switches encounter mode (local only). Do NOT clear
      // the global fallback store here; header actions handle explicit clears.
      lastFallbackEncounterId = null;
    });

    // mirror global fallback id so it persists across tab/footer navigation and back
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

  let healPlayersWindow: PlayersWindow = $state({ playerRows: [], localPlayerUid: -1, topValue: 0 });

  $effect(() => {
    // Re-fetch when encounter changes
    const encId = currentEncounterId;
    const type = currentEncounterType;
    console.log("Heal page effect triggered:", { type, encId });
    fetchData();
  });

  async function fetchData() {
    try {
      if (currentEncounterType === "historical" && currentEncounterId) {
        const result = await commands.getHistoricalPlayersWindow(currentEncounterId);
        if (result.status !== "ok") {
          console.error("Failed to get historical players window:", result.error);
          return;
        }
        healPlayersWindow = result.data;
      } else {
        // Similar guard as DPS: keep last non-empty heal view until new non-zero heal data arrives.
        let fetchedResult = SETTINGS.misc.state.testingMode ? await commands.getTestPlayerWindow() : await commands.getHealPlayerWindow();
        
        // Extract the data from Result type
        let fetched: PlayersWindow;
        if ('status' in fetchedResult && fetchedResult.status === 'ok') {
          fetched = fetchedResult.data;
        } else if ('status' in fetchedResult) {
          console.error("Failed to fetch heal player window:", fetchedResult.error);
          return;
        } else {
          // Direct PlayersWindow object
          fetched = fetchedResult;
        }

        const fetchedHasAnyValue = (fetched.playerRows ?? []).some((r) => (r.totalValue ?? 0) > 0) || (fetched.topValue ?? 0) > 0;
        const currentHasAnyValue = (healPlayersWindow.playerRows ?? []).some((r) => (r.totalValue ?? 0) > 0) || (healPlayersWindow.topValue ?? 0) > 0;

        // If live returned no data and we have a known fallback encounter id, immediately
        // load the historical players window so navigation shows the last saved encounter
        // instead of an empty live view.
        if (!fetchedHasAnyValue && currentEncounterType === "live" && lastFallbackEncounterId && lastFallbackEncounterId > 0) {
          try {
            const histRes = await commands.getHistoricalPlayersWindow(lastFallbackEncounterId);
            if (histRes.status === 'ok') {
              healPlayersWindow = histRes.data;
              return;
            }
          } catch (err) {
            console.error('Failed to load historical heal players window for fallback id:', lastFallbackEncounterId, err);
          }
        }

        if (fetchedHasAnyValue || !currentHasAnyValue) {
          healPlayersWindow = fetched;
          if (fetchedHasAnyValue) { lastFallbackEncounterId = null; liveFallbackEncounterId.set(null); }
        } else {
          // keep previous non-empty heal view
          // Only attempt fallback once per switch to avoid continuous DB polling.
          if (lastFallbackEncounterId === null) {
            try {
              const historyRes = await commands.getAllEncounterHistory();
              if (historyRes.status === 'ok' && historyRes.data && historyRes.data.length > 0) {
                const lastEncounter = historyRes.data[0]!;
                console.log('No live heal data; loading last saved encounter:', lastEncounter.id);
                const histRes = await commands.getHistoricalPlayersWindow(lastEncounter.id);
                if (histRes.status === 'ok') {
                  healPlayersWindow = histRes.data;
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
              console.error('Failed to load last historical encounter as fallback for heal page:', err);
              lastFallbackEncounterId = -1;
              liveFallbackEncounterId.set(-1);
            }
          }
        }
      }
    } catch (error) {
      console.error("Failed to fetch data:", error);
    }
  }

  const healTable = createSvelteTable({
    get data() {
      return healPlayersWindow.playerRows;
    },
    columns: healPlayersColumnDefs,
    getCoreRowModel: getCoreRowModel(),
    state: {
      get columnVisibility() {
        return SETTINGS.live.heal.players.state;
      },
    },
    meta: {
      get localPlayerUid() {
        return healPlayersWindow.localPlayerUid;
      },
    },
  });

  let SETTINGS_YOUR_NAME = $derived(SETTINGS.general.state.showYourName);
  let SETTINGS_OTHERS_NAME = $derived(SETTINGS.general.state.showOthersName);
  import { get } from 'svelte/store';

  function gotoSkillsForPlayer(uid: number) {
    const fallbackId = get(liveFallbackEncounterId);
    const encPart = fallbackId && fallbackId > 0 ? `&encounterId=${fallbackId}` : '';
    goto(`/live/heal/skills?playerUid=${uid}${encPart}`);
  }
</script>

<div class="relative flex flex-col">
  <table class="w-screen table-fixed">
    <thead class="z-1 sticky top-0 h-6">
      <tr class="bg-neutral-900">
        {#each healTable.getHeaderGroups() as headerGroup (headerGroup.id)}
          {#each headerGroup.headers as header (header.id)}
            <th class={header.column.columnDef.meta?.class}><FlexRender content={header.column.columnDef.header ?? "UNKNOWN HEADER"} context={header.getContext()} /></th>
          {/each}
        {/each}
      </tr>
    </thead>
    <tbody>
      {#each healTable.getRowModel().rows as row (row.id)}
      {@const isYou = row.original.uid !== -1 && row.original.uid == healPlayersWindow.localPlayerUid}
        {@const className = isYou ? (SETTINGS_YOUR_NAME !== "Hide Your Name" ? row.original.className : "Hidden Class") : SETTINGS_OTHERS_NAME !== "Hide Others' Name" ? row.original.className : "Hidden Class"}
  <tr class="h-7 px-2 py-1 text-center" onclick={() => gotoSkillsForPlayer(row.original.uid)}>
          {#each row.getVisibleCells() as cell (cell.id)}
            <td class="text-right"><FlexRender content={cell.column.columnDef.cell ?? "UNKNOWN CELL"} context={cell.getContext()} /></td>
          {/each}
          <td class="-z-1 absolute left-0 h-7" style="background-color: {getClassColor(className)}; width: {healPlayersWindow.topValue > 0 ? (row.original.totalValue / healPlayersWindow.topValue) * 100 : 0}%;"></td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>
