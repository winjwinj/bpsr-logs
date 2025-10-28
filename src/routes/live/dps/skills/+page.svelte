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

  let dpsSkillBreakdownWindow: SkillsWindow | undefined = $state(undefined);

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
        {#each inspectedPlayerTable.getRowModel().rows as row (row.id)}
          <tr class="h-7 px-2 py-1 text-center">
            {#each row.getVisibleCells() as cell (cell.id)}
              <td><FlexRender content={cell.column.columnDef.cell ?? "UNKNOWN CELL"} context={cell.getContext()} /></td>
            {/each}
            <td class="-z-1 absolute left-0 h-7" style="background-color: {getClassColor(dpsSkillBreakdownWindow.inspectedPlayer.className)}; width: 100vw;"></td>
          </tr>
        {/each}
        {#each dpsSkillBreakdownTable.getRowModel().rows as row, i (row.id)}
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
