<script lang="ts">
  import { onMount } from "svelte";
  import { commands, type PlayersWindow } from "$lib/bindings";
  import { getClassColor } from "$lib/utils.svelte";
  import { goto } from "$app/navigation";
  import { getCoreRowModel } from "@tanstack/table-core";
  import { createSvelteTable } from "$lib/svelte-table";
  import { healPlayersColumnDefs } from "$lib/table-info";
  import FlexRender from "$lib/svelte-table/flex-render.svelte";
  import { settings } from "$lib/settings-store";

  onMount(() => {
    fetchData();
    const interval = setInterval(fetchData, 200);

    return () => clearInterval(interval);
  });

  let healPlayersWindow: PlayersWindow = $state({ playerRows: [] });

  async function fetchData() {
    try {
      const result = await commands.getHealPlayerWindow();
      if (result.status !== "ok") {
        console.warn("Failed to get heal window: ", result.error);
        return;
      } else {
        healPlayersWindow = result.data;
        // console.log("healWindow: ", +Date.now(), $state.snapshot(healPlayersWindow));
      }
    } catch (e) {
      console.error("Error fetching data: ", e);
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
        return settings.state["live"]["heal"]["players"];
      },
    },
  });

  let maxHeal = $derived(healPlayersWindow.playerRows.reduce((max, p) => (p.totalDmg > max ? p.totalDmg : max), 0n));

  let SETTINGS_YOUR_NAME = $derived(settings.state["general"]["showYourName"]);
  let SETTINGS_OTHERS_NAME = $derived(settings.state["general"]["showOthersName"]);
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
        <tr class="h-7 px-2 py-1 text-center" onclick={() => goto(`/live/heal/skills?playerUid=${row.original.uid}`)}>
          {#each row.getVisibleCells() as cell (cell.id)}
            <td><FlexRender content={cell.column.columnDef.cell ?? "UNKNOWN CELL"} context={cell.getContext()} /></td>
          {/each}
          <td class="-z-1 absolute left-0 h-7" style="background-color: {getClassColor(className)}; width: {maxHeal > 0n ? (Number(row.original.totalDmg) / Number(maxHeal)) * 100 : 0}%;"></td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>
