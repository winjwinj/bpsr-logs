<script lang="ts">
  import { onMount } from "svelte";
  import { commands, type SkillsWindow } from "$lib/bindings";
  import { getClassColor } from "$lib/utils.svelte";
  import { page } from "$app/state";
  import { createSvelteTable, FlexRender } from "$lib/svelte-table";
  import { dpsPlayersColumnDefs, dpsSkillsColumnDefs } from "$lib/table-info";
  import { getCoreRowModel } from "@tanstack/table-core";
  import { SETTINGS } from "$lib/settings-store";

  const playerUid: string = page.url.searchParams.get("playerUid") ?? "-1";

  onMount(() => {
    fetchData();
    const interval = setInterval(fetchData, 200);
    return () => clearInterval(interval);
  });

  let dpsSkillBreakdownWindow: SkillsWindow = $state({ currPlayer: [], skillRows: [] });

  async function fetchData() {
    try {
      const result = SETTINGS.misc.state.testingMode ? await commands.getTestSkillWindow(playerUid) : await commands.getDpsSkillWindow(playerUid);
      if (result.status !== "ok") {
        console.warn("Failed to get skill window: ", result.error);
        return;
      } else {
        dpsSkillBreakdownWindow = result.data;
        // console.log("dpsSkillBreakdown: ", +Date.now(), $state.snapshot(dpsSkillBreakdownWindow));
      }
    } catch (e) {
      console.error("Error fetching data: ", e);
    }
  }

  const currPlayerTable = createSvelteTable({
    get data() {
      return dpsSkillBreakdownWindow.currPlayer;
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
      return dpsSkillBreakdownWindow.skillRows;
    },
    columns: dpsSkillsColumnDefs,
    getCoreRowModel: getCoreRowModel(),
    state: {
      get columnVisibility() {
        return SETTINGS.live.dps.skillBreakdown.state;
      },
    },
  });

  let maxSkillValue = $derived(dpsSkillBreakdownWindow.skillRows.reduce((max, p) => (p.totalDmg > max ? p.totalDmg : max), 0));

  let SETTINGS_YOUR_NAME = $derived(SETTINGS.general.state.showYourName);
  let SETTINGS_OTHERS_NAME = $derived(SETTINGS.general.state.showOthersName);
</script>

<svelte:window oncontextmenu={() => window.history.back()} />

<!-- TODO: looks ugly when split, need to figure out logic to combine together https://imgur.com/COalJFe -->
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
      {#each currPlayerTable.getRowModel().rows as row (row.id)}
        {@const currPlayer = dpsSkillBreakdownWindow.currPlayer[0]}
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
      {#each dpsSkillBreakdownTable.getRowModel().rows as row, i (row.id)}
        {@const currPlayer = dpsSkillBreakdownWindow.currPlayer[0]}
        {#if currPlayer}
          {@const className = row.original.name.includes("You") ? (SETTINGS_YOUR_NAME !== "Hide Your Name" ? currPlayer.className : "") : SETTINGS_OTHERS_NAME !== "Hide Others' Name" ? currPlayer.className : ""}
          <tr class="h-7 px-2 py-1 text-center">
            {#each row.getVisibleCells() as cell (cell.id)}
              <td><FlexRender content={cell.column.columnDef.cell ?? "UNKNOWN CELL"} context={cell.getContext()} /></td>
            {/each}
            <td class="-z-1 absolute left-0 h-7" style="background-color: {`color-mix(in srgb, ${getClassColor(className)} 80%, white ${i % 2 === 0 ? '50%' : '20%'})`}; width: {SETTINGS.general.state.relativeToTopDPSSkill ? (maxSkillValue > 0 ? (row.original.totalDmg / maxSkillValue) * 100 : 0) : row.original.dmgPct}%;"></td>
          </tr>
        {/if}
      {/each}
    </tbody>
  </table>
</div>
