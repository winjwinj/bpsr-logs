<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onDestroy, onMount } from "svelte";
  import type { DPSRow, DPSWindow, SkillRow } from "$lib/bindings";
  import SelectedPlayerSkillsTable from "$lib/components/SelectedPlayerSkillsTable.svelte";
  import "../../../app.css";
  import { formatElapsed, pct } from "$lib/utils";

  let interval: number;
  onMount(() => {
    window.addEventListener("contextmenu", (e) => {
      e.preventDefault();
    });
    interval = setInterval(fetchData, 200); // then every 200ms
  });
  onDestroy(() => {
    clearInterval(interval);
  });

  let dpsRows: DPSRow[] = $state([]);
  let dpsWindowPayload: DPSWindow = $state({ dpsRows: [], totalDmg: 0n, elapsedMs: 0n });

  // store the clicked player
  let selectedPlayerUID: bigint | null = $state(null);

  function selectPlayer(playerAndSkill: bigint) {
    selectedPlayerUID = playerAndSkill;
  }

  function clearSelection(event: MouseEvent) {
    event.preventDefault(); // stop default context menu
    selectedPlayerUID = null;
  }

  async function fetchData() {
    try {
      dpsWindowPayload = await invoke<DPSWindow>("get_damage_row");
      
      console.log(+Date.now(), $state.snapshot(dpsWindowPayload));
      dpsRows = dpsWindowPayload.dpsRows;
    } catch (e) {
      console.error("Error fetching data:", e);
    }
  }

  // Derive the selected player row and its skills (mirroring prior implicit variable)
  const selectedPlayerAndSkills = $derived.by(() => {
    if (selectedPlayerUID == null) return null;
    const player = dpsRows.find((p) => p.uid === selectedPlayerUID);
    if (!player) return null;
    return [player, player.skills] as [DPSRow, SkillRow[]];
  });
</script>

<main class="container">
  <h1 class="text-white">DPS Meter - Time Elapsed: {formatElapsed(dpsWindowPayload.elapsedMs)}</h1>

  {#if selectedPlayerUID == null}
    <!-- First table -->
    <table class="table-auto border-collapse border border-gray-300 w-full">
      <thead>
        <tr class="bg-gray-200">
          <th class="border border-gray-300 px-2 py-1">#</th>
          <th class="border border-gray-300 px-2 py-1">UID</th>
          <th class="border border-gray-300 px-2 py-1">Class</th>
          <th class="border border-gray-300 px-2 py-1">ilvl</th>
          <th class="border border-gray-300 px-2 py-1">Name</th>
          <th class="border border-gray-300 px-2 py-1">DMG</th>
          <th class="border border-gray-300 px-2 py-1">DPS</th>
          <th class="border border-gray-300 px-2 py-1">D%</th>
          <th class="border border-gray-300 px-2 py-1">CR%</th>
          <th class="border border-gray-300 px-2 py-1">CDmg%</th>
          <th class="border border-gray-300 px-2 py-1">LR%</th>
          <th class="border border-gray-300 px-2 py-1">LDmg%</th>
        </tr>
      </thead>
      <tbody>
        {#each dpsRows as player, index}
          <tr
            class="odd:bg-white even:bg-gray-50 hover:bg-gray-100 cursor-pointer"
            onclick={() => selectPlayer(player.uid)}
          >
            <!-- TODO: skills become static after clicking on player, something wrong about reactivity, maybe just have a rust command for each of these component views -->
            <td class="border border-gray-300 px-2 py-1">{index + 1}</td>
            <td class="border border-gray-300 px-2 py-1">{player.uid}</td>
            <td class="border border-gray-300 px-2 py-1">{player.class || "??"}</td>
            <td class="border border-gray-300 px-2 py-1">{player.abilityScore || "??"}</td>
            <td class="border border-gray-300 px-2 py-1">{player.name || "??"}</td>
            <td class="border border-gray-300 px-2 py-1">{player.totalDmg}</td>
            <td class="border border-gray-300 px-2 py-1">{player.dps.toFixed()}</td>
            <td class="border border-gray-300 px-2 py-1">{pct(player.totalDmg, dpsWindowPayload.totalDmg)}</td>
            <td class="border border-gray-300 px-2 py-1">{player.critRate.toFixed(2)}</td>
            <td class="border border-gray-300 px-2 py-1">{player.critDmgRate.toFixed(2)}</td>
            <td class="border border-gray-300 px-2 py-1">{player.luckyRate.toFixed(2)}</td>
            <td class="border border-gray-300 px-2 py-1">{player.luckyDmgRate.toFixed(2)}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {:else}
    <SelectedPlayerSkillsTable {selectedPlayerAndSkills} {dpsWindowPayload} {clearSelection} />
  {/if}
</main>
