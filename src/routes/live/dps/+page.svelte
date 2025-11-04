<script lang="ts">
  import { onMount } from "svelte";
  import { commands, type PlayersWindow } from "$lib/bindings";
  import { getClassColor } from "$lib/utils.svelte";
  import { goto } from "$app/navigation";
  import { getCoreRowModel } from "@tanstack/table-core";
  import { createSvelteTable } from "$lib/svelte-table";
  import { dpsPlayersColumnDefs } from "$lib/table-info";
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
    // Subscribe to encounter changes (do NOT clear the shared fallback store here; that
    // causes a race when pages mount and immediately wipe the fallback). Clearing the
    // shared fallback is done explicitly when the user switches encounters in the header.
    unsubscribe = selectedEncounter.subscribe((value) => {
      currentEncounterType = value.type;
      currentEncounterId = value.encounterId;
      // Reset local fallback tracking for this page; global store is preserved.
      lastFallbackEncounterId = null;
    });

    // Also subscribe to the shared live fallback id so it persists across navigation
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

  let dpsPlayersWindow: PlayersWindow = $state({ playerRows: [], localPlayerUid: -1, topValue: 0 });

  $effect(() => {
    // Re-fetch when encounter changes
    const encId = currentEncounterId;
    const type = currentEncounterType;
    console.log("DPS page effect triggered:", { type, encId });
    fetchData();
  });

  async function fetchData() {
    try {
      if (currentEncounterType === "historical" && currentEncounterId) {
        console.log("Fetching historical data for encounter:", currentEncounterId);
        const result = await commands.getHistoricalPlayersWindow(currentEncounterId);
        if (result.status !== "ok") {
          console.error("Failed to get historical players window:", result.error);
          return;
        }
        console.log("Historical players window result:", result);
        console.log("Extracted data:", result.data);
        dpsPlayersWindow = result.data;
        console.log("Table should have", dpsPlayersWindow.playerRows?.length ?? 0, "rows");
      } else {
        // For live data: fetch but do not overwrite an existing non-empty view with an empty/zeroed result.
        // This keeps the UI showing the last legitimate combat until a new combat with any non-zero
        // damage/healing arrives. This is purely a frontend guard and does not send any commands.
        let fetchedResult: any;
        if (SETTINGS.misc.state.testingMode) {
          fetchedResult = await commands.getTestPlayerWindow();
        } else if (SETTINGS.general.state.bossOnly) {
          fetchedResult = await commands.getDpsBossOnlyPlayerWindow();
        } else {
          fetchedResult = await commands.getDpsPlayerWindow();
        }

        // Extract the data from Result type
        let fetched: PlayersWindow;
        if (fetchedResult.status === 'ok') {
          fetched = fetchedResult.data;
        } else {
          console.error("Failed to fetch DPS player window:", fetchedResult.error);
          return;
        }

        const fetchedHasAnyValue = (fetched.playerRows ?? []).some((r) => (r.totalValue ?? 0) > 0) || (fetched.topValue ?? 0) > 0;
        const currentHasAnyValue = (dpsPlayersWindow.playerRows ?? []).some((r) => (r.totalValue ?? 0) > 0) || (dpsPlayersWindow.topValue ?? 0) > 0;

        // If we're in live mode and the live API returned no combat data, prefer the
        // previously saved historical encounter (if we have one) when navigating between
        // pages — this prevents a freshly-mounted page from immediately rendering an
        // empty live result when the selectedEncounter is still `live` but the backend
        // has already reset the encounter.
        if (!fetchedHasAnyValue && currentEncounterType === "live" && lastFallbackEncounterId && lastFallbackEncounterId > 0) {
          try {
            const histRes = await commands.getHistoricalPlayersWindow(lastFallbackEncounterId);
            if (histRes.status === 'ok') {
              dpsPlayersWindow = histRes.data;
              // keep lastFallbackEncounterId as-is (store already contains it)
              return;
            }
          } catch (err) {
            console.error('Failed to load historical players window for fallback id:', lastFallbackEncounterId, err);
          }
        }

        if (fetchedHasAnyValue || !currentHasAnyValue) {
          // If fetched contains real data, or we currently have nothing, replace the view.
          dpsPlayersWindow = fetched;
          // If we received real live data, allow future fallbacks to run again later if needed
          if (fetchedHasAnyValue) { lastFallbackEncounterId = null; liveFallbackEncounterId.set(null); }
        } else {
          // Otherwise keep the existing view until a non-zero packet arrives.
          // Only attempt the historical fallback once per switch to avoid continuous DB polling.
          if (lastFallbackEncounterId === null) {
            try {
              const historyRes = await commands.getAllEncounterHistory();
              if (historyRes.status === 'ok' && historyRes.data && historyRes.data.length > 0) {
                const lastEncounter = historyRes.data[0]!;
                console.log('No live data; loading last saved encounter:', lastEncounter.id);
                const histRes = await commands.getHistoricalPlayersWindow(lastEncounter.id);
                if (histRes.status === 'ok') {
                  dpsPlayersWindow = histRes.data;
                  lastFallbackEncounterId = lastEncounter.id ?? -1;
                  liveFallbackEncounterId.set(lastFallbackEncounterId);
                } else {
                  // mark attempted to prevent retry storm
                  lastFallbackEncounterId = -1;
                  liveFallbackEncounterId.set(-1);
                }
              } else {
                lastFallbackEncounterId = -1;
                liveFallbackEncounterId.set(-1);
              }
            } catch (err) {
              console.error('Failed to load last historical encounter as fallback:', err);
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

  const dpsTable = createSvelteTable({
    get data() {
      return dpsPlayersWindow.playerRows;
    },
    columns: dpsPlayersColumnDefs,
    getCoreRowModel: getCoreRowModel(),
    state: {
      get columnVisibility() {
        return SETTINGS.live.dps.players.state;
      },
    },
    meta: {
      get localPlayerUid() {
        return dpsPlayersWindow.localPlayerUid;
      },
    },
  });

  let SETTINGS_YOUR_NAME = $derived(SETTINGS.general.state.showYourName);
  let SETTINGS_OTHERS_NAME = $derived(SETTINGS.general.state.showOthersName);
  import { get } from 'svelte/store';

  function gotoSkillsForPlayer(uid: number) {
    const fallbackId = get(liveFallbackEncounterId);
    const encPart = fallbackId && fallbackId > 0 ? `&encounterId=${fallbackId}` : '';
    goto(`/live/dps/skills?playerUid=${uid}${encPart}`);
  }
</script>

<div class="relative flex flex-col">
  <table class="w-screen table-fixed">
    <thead class="z-1 sticky top-0 h-6">
      <tr class="bg-neutral-900">
        {#each dpsTable.getHeaderGroups() as headerGroup (headerGroup.id)}
          {#each headerGroup.headers as header (header.id)}
            <th class={header.column.columnDef.meta?.class}><FlexRender content={header.column.columnDef.header ?? "UNKNOWN HEADER"} context={header.getContext()} /></th>
          {/each}
        {/each}
      </tr>
    </thead>
    <tbody>
      {#each dpsTable.getRowModel().rows as row (row.id)}
      {@const isYou = row.original.uid !== -1 && row.original.uid == dpsPlayersWindow.localPlayerUid}
        {@const className = isYou ? (SETTINGS_YOUR_NAME !== "Hide Your Name" ? row.original.className : "Hidden Class") : SETTINGS_OTHERS_NAME !== "Hide Others' Name" ? row.original.className : "Hidden Class"}
  <tr class="h-7 px-2 py-1 text-center" onclick={() => gotoSkillsForPlayer(row.original.uid)}>
          {#each row.getVisibleCells() as cell (cell.id)}
            <td class="text-right"><FlexRender content={cell.column.columnDef.cell ?? "UNKNOWN CELL"} context={cell.getContext()} /></td>
          {/each}
          <td class="-z-1 absolute left-0 h-7" style="background-color: {getClassColor(className)}; width: {dpsPlayersWindow.topValue > 0 ? (row.original.totalValue / dpsPlayersWindow.topValue) * 100 : 0}%;"></td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>
