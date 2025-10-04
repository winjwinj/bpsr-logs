<script lang="ts">
  import { onMount } from "svelte";
  import { commands, type DPSSkillBreakdownWindow } from "$lib/bindings";
  import { getClassColor, getClassIcon, tooltip } from "$lib/utils.svelte";
  import { page } from "$app/state";
  import AbbreviatedNumber from "$lib/components/abbreviated-number.svelte";

  const playerUid: string = page.url.searchParams.get("playerUid") ?? "-1";

  onMount(() => {
    fetchData();
    const interval = setInterval(fetchData, 200);
    return () => clearInterval(interval);
  });

  let healSkillBreakdownWindow: DPSSkillBreakdownWindow | undefined = $state();

  async function fetchData() {
    try {
      const result = await commands.getHealSkillWindow(playerUid);
      if (result.status !== "ok") {
        console.warn("Failed to get skill window: ", result.error);
        return;
      } else {
        healSkillBreakdownWindow = result.data;
        console.log("healSkillBreakdown: ", +Date.now(), $state.snapshot(healSkillBreakdownWindow));
      }
    } catch (e) {
      console.error("Error fetching data: ", e);
    }
  }
</script>

{#if healSkillBreakdownWindow !== undefined}
  <div class="relative flex flex-col">
    <table
      class="w-screen table-fixed"
      oncontextmenu={() => {
        window.history.back();
      }}
    >
      <thead class="z-1 sticky top-0 h-6">
        <tr class="bg-neutral-900">
          <th class="w-5 px-4"><!-- Class Image --></th>
          <th class="w-5/8"><!-- Ability Score + Name --></th>
          <th class="w-12"><span {@attach tooltip(() => "Healing Dealt")}>DMG</span></th>
          <th class="w-12"><span {@attach tooltip(() => "Healing per Second")}>DPS</span></th>
          <th class="w-12"><span {@attach tooltip(() => "Healing %")}>H<span class="text-tiny text-gray-300">%</span></span></th>
          <th class="w-12"><span {@attach tooltip(() => "Crit Rate %")}>CR<span class="text-tiny text-gray-300">%</span></span></th>
          <th class="w-13"><span {@attach tooltip(() => "% Healing that Crit")}>CDMG<span class="text-tiny text-gray-300">%</span></span></th>
          <th class="w-12"><span {@attach tooltip(() => "Lucky Rate %")}>LR<span class="text-tiny text-gray-300">%</span></span></th>
          <th class="w-13"><span {@attach tooltip(() => "% Healing that was Lucky")}>LDMG<span class="text-tiny text-gray-300">%</span></span></th>
          <th class="w-13"><span {@attach tooltip(() => "Number of hits")}>Hits</span></th>
          <th class="w-13"><span {@attach tooltip(() => "Hits per minute")}>HPM</span></th>
        </tr>
      </thead>
      <tbody>
        <tr class="h-7 px-2 py-1 text-center">
          <td {@attach tooltip(() => `${healSkillBreakdownWindow.currPlayer.class}-${healSkillBreakdownWindow.currPlayer.classSpec}`)}><img class="ml-2 size-5 object-contain" src={getClassIcon(healSkillBreakdownWindow.currPlayer.class)} alt={`${healSkillBreakdownWindow.currPlayer.class} class icon`} /></td>
          <td><span class="flex"><span class="cursor-pointer truncate" {@attach tooltip(() => `UID: #${healSkillBreakdownWindow.currPlayer.uid}`)}>{`${healSkillBreakdownWindow.currPlayer.abilityScore && healSkillBreakdownWindow.currPlayer.abilityScore !== 0 ? healSkillBreakdownWindow.currPlayer.abilityScore : "??"} ${healSkillBreakdownWindow.currPlayer.name?.trim() ? healSkillBreakdownWindow.currPlayer.name : "Unknown Name"}`}</span></span></td>
          <td><AbbreviatedNumber val={Number(healSkillBreakdownWindow.currPlayer.totalDmg)} /></td>
          <td><AbbreviatedNumber val={healSkillBreakdownWindow.currPlayer.dps} /></td>
          <td>{healSkillBreakdownWindow.currPlayer.dmgPct.toFixed(0)}<span class="text-tiny text-gray-300">%</span></td>
          <td>{healSkillBreakdownWindow.currPlayer.critRate.toFixed(1)}<span class="text-tiny text-gray-300">%</span></td>
          <td>{healSkillBreakdownWindow.currPlayer.critDmgRate.toFixed(1)}<span class="text-tiny text-gray-300">%</span></td>
          <td>{healSkillBreakdownWindow.currPlayer.luckyRate.toFixed(1)}<span class="text-tiny text-gray-300">%</span></td>
          <td>{healSkillBreakdownWindow.currPlayer.luckyDmgRate.toFixed(1)}<span class="text-tiny text-gray-300">%</span></td>
          <td>{healSkillBreakdownWindow.currPlayer.hits}</td>
          <td>{healSkillBreakdownWindow.currPlayer.hitsPerMinute.toFixed(1)}</td>
          <td class="-z-1 absolute left-0 h-7" style="background-color: {getClassColor(healSkillBreakdownWindow.currPlayer.class)}; width: 100vw;"></td>
        </tr>
        <!-- TODO: get some sort of key for this each loop -->
        <!-- eslint-disable svelte/require-each-key -->
        {#each healSkillBreakdownWindow.skillRows as skillRow (skillRow)}
          <tr class="h-7 px-2 py-1 text-center">
            <td colspan="2"><span class="ml-2 flex"><span class="truncate">{skillRow.name}</span></span></td>
            <td><AbbreviatedNumber val={Number(skillRow.totalDmg)} /></td>
            <td><AbbreviatedNumber val={skillRow.dps} /></td>
            <td>{skillRow.dmgPct.toFixed(0)}<span class="text-tiny text-gray-300">%</span></td>
            <td>{skillRow.critRate.toFixed(1)}<span class="text-tiny text-gray-300">%</span></td>
            <td>{skillRow.critDmgRate.toFixed(1)}<span class="text-tiny text-gray-300">%</span></td>
            <td>{skillRow.luckyRate.toFixed(1)}<span class="text-tiny text-gray-300">%</span></td>
            <td>{skillRow.luckyDmgRate.toFixed(1)}<span class="text-tiny text-gray-300">%</span></td>
            <td>{skillRow.hits}</td>
            <td>{skillRow.hitsPerMinute.toFixed(1)}</td>
            <td class="-z-1 absolute left-0 h-7" style="background-color: {`color-mix(in srgb, ${getClassColor(healSkillBreakdownWindow.currPlayer.class)} 80%, white ${i % 2 === 0 ? '30%' : '10%'})`}; width: {skillRow.dmgPct}vw;"></td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/if}
