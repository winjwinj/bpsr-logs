<script lang="ts">
  import { onMount } from "svelte";
  import { commands, type DPSSkillBreakdownWindow } from "$lib/bindings";
  import { copyToClipboard, getClassColor, getClassIcon, tooltip } from "$lib/utils.svelte";
  import { page } from "$app/state";
  import AbbreviatedNumber from "$lib/components/abbreviated-number.svelte";

  const playerUid: string = page.url.searchParams.get("playerUid") ?? "-1";

  onMount(() => {
    fetchData();
    const interval = setInterval(fetchData, 200);
    return () => clearInterval(interval);
  });

  let dpsSkillBreakdownWindow: DPSSkillBreakdownWindow | undefined = $state();

  async function fetchData() {
    try {
      const result = await commands.getSkillWindow(playerUid);
      if (result.status !== "ok") {
        console.warn("Failed to get skill window: ", result.error);
        return;
      } else {
        dpsSkillBreakdownWindow = result.data;
        console.log("dpsSkillBreakdown: ", +Date.now(), $state.snapshot(dpsSkillBreakdownWindow));
      }
    } catch (e) {
      console.error("Error fetching data:", e);
    }
  }
</script>

{#if dpsSkillBreakdownWindow !== undefined}
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
          <th class="w-12"><span {@attach tooltip(() => "Damage Dealt")}>DMG</span></th>
          <th class="w-12"><span {@attach tooltip(() => "Damage per Second")}>DPS</span></th>
          <th class="w-12"><span {@attach tooltip(() => "Damage %")}>D<span class="text-tiny text-gray-300">%</span></span></th>
          <th class="w-12"><span {@attach tooltip(() => "Crit Rate %")}>CR<span class="text-tiny text-gray-300">%</span></span></th>
          <th class="w-13"><span {@attach tooltip(() => "% Damage that Crit")}>CDMG<span class="text-tiny text-gray-300">%</span></span></th>
          <th class="w-12"><span {@attach tooltip(() => "Lucky Rate %")}>LR<span class="text-tiny text-gray-300">%</span></span></th>
          <th class="w-13"><span {@attach tooltip(() => "% Damage that was Lucky")}>LDMG<span class="text-tiny text-gray-300">%</span></span></th>
          <th class="w-13"><span {@attach tooltip(() => "Number of hits")}>Hits</span></th>
          <th class="w-13"><span {@attach tooltip(() => "Hits per minute")}>HPM</span></th>
        </tr>
      </thead>
      <tbody>
        <tr class="h-7 px-2 py-1 text-center">
          <td {@attach tooltip(() => `${dpsSkillBreakdownWindow.currPlayer.class}-${dpsSkillBreakdownWindow.currPlayer.classSpec}`)}><img class="ml-2 size-5 object-contain" src={getClassIcon(dpsSkillBreakdownWindow.currPlayer.class)} alt={`${dpsSkillBreakdownWindow.currPlayer.class} class icon`} /></td>
          <td><span class="flex"><span class="cursor-pointer truncate" onclick={(error) => copyToClipboard(error, `#${dpsSkillBreakdownWindow.currPlayer.uid}`)} {@attach tooltip(() => `UID: #${dpsSkillBreakdownWindow.currPlayer.uid}`)}>{`${dpsSkillBreakdownWindow.currPlayer.abilityScore && dpsSkillBreakdownWindow.currPlayer.abilityScore !== 0 ? dpsSkillBreakdownWindow.currPlayer.abilityScore : "??"} ${dpsSkillBreakdownWindow.currPlayer.name?.trim() ? dpsSkillBreakdownWindow.currPlayer.name : "Unknown Name"}`}</span></span></td>
          <td><AbbreviatedNumber val={Number(dpsSkillBreakdownWindow.currPlayer.totalDmg)} /></td>
          <td><AbbreviatedNumber val={dpsSkillBreakdownWindow.currPlayer.dps} /></td>
          <td>{dpsSkillBreakdownWindow.currPlayer.dmgPct.toFixed(0)}<span class="text-tiny text-gray-300">%</span></td>
          <td>{dpsSkillBreakdownWindow.currPlayer.critRate.toFixed(1)}<span class="text-tiny text-gray-300">%</span></td>
          <td>{dpsSkillBreakdownWindow.currPlayer.critDmgRate.toFixed(1)}<span class="text-tiny text-gray-300">%</span></td>
          <td>{dpsSkillBreakdownWindow.currPlayer.luckyRate.toFixed(1)}<span class="text-tiny text-gray-300">%</span></td>
          <td>{dpsSkillBreakdownWindow.currPlayer.luckyDmgRate.toFixed(1)}<span class="text-tiny text-gray-300">%</span></td>
          <td>{dpsSkillBreakdownWindow.currPlayer.hits}</td>
          <td>{dpsSkillBreakdownWindow.currPlayer.hitsPerMinute.toFixed(1)}</td>
          <td class="-z-1 absolute left-0 h-7" style="background-color: {getClassColor(dpsSkillBreakdownWindow.currPlayer.class)}; width: 100vw;"></td>
        </tr>
        <!-- TODO: get some sort of key for this each loop -->
        <!-- eslint-disable svelte/require-each-key -->
        {#each dpsSkillBreakdownWindow.skillRows as skillRow (skillRow)}
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
            <td class="-z-1 absolute left-0 h-7" style="background-color: {`color-mix(in srgb, ${getClassColor(dpsSkillBreakdownWindow.currPlayer.class)} 80%, white ${i % 2 === 0 ? '30%' : '10%'})`}; width: {skillRow.dmgPct}vw;"></td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/if}
