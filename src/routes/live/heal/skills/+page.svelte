<script lang="ts">
  import { onMount } from "svelte";
  import { commands, type SkillsWindow } from "$lib/bindings";
  import { selectedEncounter, selectedEncounterId, encounters } from "$lib/encounter-history-store";
  import { selectEncounter } from "$lib/encounter-history-store";
  import { getClassColor } from "$lib/utils.svelte";
  import { page } from "$app/state";
  import { createSvelteTable, FlexRender } from "$lib/svelte-table";
  import { healPlayersColumnDefs, healSkillsColumnDefs } from "$lib/table-info";
  import { getCoreRowModel } from "@tanstack/table-core";
  import { SETTINGS } from "$lib/settings-store";

  const playerUid: string = page.url.searchParams.get("playerUid") ?? "-1";
  const encParam: string | null = page.url.searchParams.get("enc");

  onMount(() => {
    if (encParam) selectEncounter(encParam);
    fetchData();
    const interval = setInterval(fetchData, 200);
    return () => clearInterval(interval);
  });
  
  $effect(() => {
    if ($selectedEncounter) fetchData();
  });

  let healSkillBreakdownWindow: SkillsWindow = $state({ currPlayer: [], skillRows: [] });

  async function fetchData() {
    try {
      // If a historical encounter is selected, try to populate from snapshot if available
      if ($selectedEncounter?.mode === 'history') {
        const snap = $selectedEncounter.data;
        if (snap?.healSkillWindows && snap.healSkillWindows[playerUid]) {
          const src = snap.healSkillWindows[playerUid];
          healSkillBreakdownWindow = {
            currPlayer: Array.isArray(src.currPlayer) ? [...src.currPlayer] : [],
            skillRows: Array.isArray(src.skillRows) ? [...src.skillRows] : [],
          };
          return;
        }
        if (snap?.healPlayersWindow) {
          healSkillBreakdownWindow = { currPlayer: [], skillRows: [] };
          return;
        }
      }

      if ($selectedEncounterId === 'current' && SETTINGS.general.state.useNewestHistoryOverlayWhenCurrent) {
        const newest = ($encounters || [])[0];
        if (newest?.data?.healSkillWindows && newest.data.healSkillWindows[playerUid]) {
          const src = newest.data.healSkillWindows[playerUid];
          healSkillBreakdownWindow = {
            currPlayer: Array.isArray(src.currPlayer) ? [...src.currPlayer] : [],
            skillRows: Array.isArray(src.skillRows) ? [...src.skillRows] : [],
          };
          return;
        }
      }

      const result = SETTINGS.misc.state.testingMode ? await commands.getTestSkillWindow(playerUid) : await commands.getHealSkillWindow(playerUid);
      if (result.status !== "ok") {
        console.warn("Failed to get skill window: ", result.error);
        return;
      } else {
        // clone arrays to ensure reactivity
        const src = result.data;
        healSkillBreakdownWindow = {
          currPlayer: Array.isArray(src.currPlayer) ? [...src.currPlayer] : [],
          skillRows: Array.isArray(src.skillRows) ? [...src.skillRows] : [],
        };
      }
    } catch (e) {
      console.error("Error fetching data: ", e);
    }
  }

  const currPlayerTable = createSvelteTable({
    get data() {
      return healSkillBreakdownWindow.currPlayer;
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

  let maxSkillValue = $derived(healSkillBreakdownWindow.skillRows.reduce((max, p) => (p.totalDmg > max ? p.totalDmg : max), 0));

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
      {#each currPlayerTable.getRowModel().rows as row (row.id)}
        {@const currPlayer = healSkillBreakdownWindow.currPlayer[0]}
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
      {#each healSkillBreakdownTable.getRowModel().rows as row, i (row.id)}
        {@const currPlayer = healSkillBreakdownWindow.currPlayer[0]}
        {#if currPlayer}
          {@const className = row.original.name.includes("You") ? (SETTINGS_YOUR_NAME !== "Hide Your Name" ? currPlayer.className : "") : SETTINGS_OTHERS_NAME !== "Hide Others' Name" ? currPlayer.className : ""}
          <tr class="h-7 px-2 py-1 text-center">
            {#each row.getVisibleCells() as cell (cell.id)}
              <td><FlexRender content={cell.column.columnDef.cell ?? "UNKNOWN CELL"} context={cell.getContext()} /></td>
            {/each}
            <td class="-z-1 absolute left-0 h-7" style="background-color: {`color-mix(in srgb, ${getClassColor(className)} 80%, white ${i % 2 === 0 ? '50%' : '20%'})`}; width: {SETTINGS.general.state.relativeToTopHealSkill ? (maxSkillValue > 0 ? (row.original.totalDmg / maxSkillValue) * 100 : 0) : row.original.dmgPct}%;"></td>
          </tr>
        {/if}
      {/each}
    </tbody>
  </table>
</div>
