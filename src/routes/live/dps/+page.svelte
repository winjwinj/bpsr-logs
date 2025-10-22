<script lang="ts">
  import { onMount } from "svelte";
  import { commands, type PlayersWindow } from "$lib/bindings";
  import { selectedEncounter, selectedEncounterId, encounters } from "$lib/encounter-history-store";
  import { selectEncounter } from '$lib/encounter-history-store';
  import { page } from '$app/state';
  import { getClassColor } from "$lib/utils.svelte";
  import { goto } from "$app/navigation";
  import { getCoreRowModel } from "@tanstack/table-core";
  import { createSvelteTable } from "$lib/svelte-table";
  import { dpsPlayersColumnDefs } from "$lib/table-info";
  import FlexRender from "$lib/svelte-table/flex-render.svelte";
  import { SETTINGS } from "$lib/settings-store";

  onMount(() => {
    // Restore selected encounter from query param if provided
    const params = new URLSearchParams(page.url.search);
    const enc = params.get('enc');
    if (enc) selectEncounter(enc);

    fetchData();
    const interval = setInterval(fetchData, 200);

    return () => clearInterval(interval);
  });

  let dpsPlayersWindow: PlayersWindow = $state({ playerRows: [] });

  async function fetchData() {
    try {
      const result = SETTINGS.misc.state.testingMode ? await commands.getTestPlayerWindow() : await commands.getDpsPlayerWindow();
      if (result.status !== "ok") {
        console.warn("timestamp: ", +Date.now(), " Failed to get dps window: ", +Date.now(), result.error);
        return;
      } else {
        // Always assign new references to trigger reactivity
        dpsPlayersWindow = result.data;
        // dpsPlayersWindow = {
        //  ...result.data,
        //   playerRows: [...result.data.playerRows],
        // };
        // console.log("timestamp: ", +Date.now(), " dpsPlayersWindow: ", $state.snapshot(dpsPlayersWindow));
      }
    } catch (e) {
      console.error("Error fetching data: ", e);
    }
  }

  const dpsTable = createSvelteTable({
    get data() {
      // if a history encounter is selected and has dpsPlayersWindow, show its rows; otherwise show live
      if ($selectedEncounter?.mode === 'history' && $selectedEncounter?.data?.dpsPlayersWindow) {
        return $selectedEncounter.data.dpsPlayersWindow.playerRows;
      }
      // If the user has 'current' selected but enabled the overlay setting, and live data is empty,
      // display the newest history snapshot (transient) while keeping selection on 'current'.
      if ($selectedEncounterId === 'current' && SETTINGS.general.state.useNewestHistoryOverlayWhenCurrent) {
        const newest = ($encounters || [])[0];
        if (newest?.data?.dpsPlayersWindow && (!dpsPlayersWindow.playerRows || dpsPlayersWindow.playerRows.length === 0)) {
          return newest.data.dpsPlayersWindow.playerRows;
        }
      }
      return dpsPlayersWindow.playerRows;
    },
    columns: dpsPlayersColumnDefs,
    getCoreRowModel: getCoreRowModel(),
    state: {
      get columnVisibility() {
        return SETTINGS.live.dps.players.state;
      },
    },
  });

  let maxDamage = $derived(dpsPlayersWindow.playerRows.reduce((max, p) => (p.totalDmg > max ? p.totalDmg : max), 0));

  let SETTINGS_YOUR_NAME = $derived(SETTINGS.general.state.showYourName);
  let SETTINGS_OTHERS_NAME = $derived(SETTINGS.general.state.showOthersName);

  $inspect("[REACTIVE] dpsPlayersWindow changed:", dpsPlayersWindow);
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
        {@const className = row.original.name.includes("You") ? (SETTINGS_YOUR_NAME !== "Hide Your Name" ? row.original.className : "") : SETTINGS_OTHERS_NAME !== "Hide Others' Name" ? row.original.className : ""}
        <tr class="h-7 px-2 py-1 text-center" onclick={() => {
            const enc = $selectedEncounter?.mode === 'history' ? $selectedEncounter.id : null;
            const base = `/live/dps/skills?playerUid=${row.original.uid}`;
            const url = enc ? `${base}&enc=${encodeURIComponent(enc)}` : base;
            goto(url);
          }}>
          {#each row.getVisibleCells() as cell (cell.id)}
            <td class="text-right"><FlexRender content={cell.column.columnDef.cell ?? "UNKNOWN CELL"} context={cell.getContext()} /></td>
          {/each}
          <td class="-z-1 absolute left-0 h-7" style="background-color: {getClassColor(className)}; width: {SETTINGS.general.state.relativeToTopDPSPlayer ? (maxDamage > 0 ? (row.original.totalDmg / maxDamage) * 100 : 0) : row.original.dmgPct}%;"></td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>
