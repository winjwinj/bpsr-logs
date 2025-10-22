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
  import { healPlayersColumnDefs } from "$lib/table-info";
  import FlexRender from "$lib/svelte-table/flex-render.svelte";
  import { SETTINGS } from "$lib/settings-store";

  onMount(() => {
    const params = new URLSearchParams(page.url.search);
    const enc = params.get('enc');
    if (enc) selectEncounter(enc);

    fetchData();
    const interval = setInterval(fetchData, 200);

    return () => clearInterval(interval);
  });

  let healPlayersWindow: PlayersWindow = $state({ playerRows: [] });

  async function fetchData() {
    try {
      const result = SETTINGS.misc.state.testingMode ? await commands.getTestPlayerWindow() : await commands.getHealPlayerWindow();
      if (result.status !== "ok") {
        console.warn("timestamp: ", +Date.now(), " Failed to get heal window: ", +Date.now(), result.error);
        return;
      } else {
        healPlayersWindow = result.data;
        console.log("timestamp: ", +Date.now(), " healPlayersWindow: ", $state.snapshot(healPlayersWindow));
      }
    } catch (e) {
      console.error("Error fetching data: ", e);
    }
  }

  const healTable = createSvelteTable({
    get data() {
      if ($selectedEncounter?.mode === 'history' && $selectedEncounter?.data?.healPlayersWindow) {
        return $selectedEncounter.data.healPlayersWindow.playerRows;
      }
      if ($selectedEncounterId === 'current' && SETTINGS.general.state.useNewestHistoryOverlayWhenCurrent) {
        const newest = ($encounters || [])[0];
        if (newest?.data?.healPlayersWindow && (!healPlayersWindow.playerRows || healPlayersWindow.playerRows.length === 0)) {
          return newest.data.healPlayersWindow.playerRows;
        }
      }
      return healPlayersWindow.playerRows;
    },
    columns: healPlayersColumnDefs,
    getCoreRowModel: getCoreRowModel(),
    state: {
      get columnVisibility() {
        return SETTINGS.live.heal.players.state;
      },
    },
  });

  let maxHeal = $derived(healPlayersWindow.playerRows.reduce((max, p) => (p.totalDmg > max ? p.totalDmg : max), 0));

  let SETTINGS_YOUR_NAME = $derived(SETTINGS.general.state.showYourName);
  let SETTINGS_OTHERS_NAME = $derived(SETTINGS.general.state.showOthersName);
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
        {@const className = row.original.name.includes("You") ? (SETTINGS_YOUR_NAME !== "Hide Your Name" ? row.original.className : "") : SETTINGS_OTHERS_NAME !== "Hide Others' Name" ? row.original.className : ""}
        <tr class="h-7 px-2 py-1 text-center" onclick={() => {
            const enc = $selectedEncounter?.mode === 'history' ? $selectedEncounter.id : null;
            const base = `/live/heal/skills?playerUid=${row.original.uid}`;
            const url = enc ? `${base}&enc=${encodeURIComponent(enc)}` : base;
            goto(url);
          }}>
          {#each row.getVisibleCells() as cell (cell.id)}
            <td class="text-right"><FlexRender content={cell.column.columnDef.cell ?? "UNKNOWN CELL"} context={cell.getContext()} /></td>
          {/each}
          <td class="-z-1 absolute left-0 h-7" style="background-color: {getClassColor(className)}; width: {SETTINGS.general.state.relativeToTopHealPlayer ? (maxHeal > 0 ? (row.original.totalDmg / maxHeal) * 100 : 0) : row.original.dmgPct}%;"></td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>
