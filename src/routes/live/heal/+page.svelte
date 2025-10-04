<script lang="ts">
  import { onMount } from "svelte";
  import { commands, type DPSWindow } from "$lib/bindings";
  import { copyToClipboard, getClassColor, getClassIcon, tooltip } from "$lib/utils.svelte";
  import { goto } from "$app/navigation";
  import AbbreviatedNumber from "$lib/components/abbreviated-number.svelte";

  onMount(() => {
    fetchData();
    const interval = setInterval(fetchData, 200);

    return () => clearInterval(interval);
  });

  let dpsWindow: DPSWindow | undefined = $state();

  async function fetchData() {
    try {
      const result = await commands.getHealWindow();
      if (result.status !== "ok") {
        console.warn("Failed to get heal window: ", result.error);
        return;
      } else {
        dpsWindow = result.data;
        console.log("healWindow: ", +Date.now(), $state.snapshot(dpsWindow));
      }
    } catch (e) {
      console.error("Error fetching data:", e);
    }
  }
</script>

{#if dpsWindow !== undefined}
  <div class="relative flex flex-col">
    <table class="w-screen table-fixed">
      <thead class="z-1 sticky top-0 h-6">
        <tr class="bg-neutral-900">
          <th class="w-3/8 pl-2 text-left"><span>UID</span></th>
          <th class="w-5 px-4"><!-- Class Image --></th>
          <th class="w-5/8"><!-- Ability Score + Name --></th>
          <th class="w-12"><span {@attach tooltip(() => "Healing Done")}>Heal</span></th>
          <th class="w-12"><span {@attach tooltip(() => "Healing per Second")}>HPS</span></th>
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
        {#each dpsWindow.dpsRows as dpsRow (dpsRow)}
          <tr class="h-7 px-2 py-1 text-center" onclick={() => goto(`/live/heal/dpsSkillBreakdown?playerUid=${dpsRow.uid}`)}>
            <td class="truncate pl-2 text-left">{dpsRow.uid}</td>
            <td {@attach tooltip(() => `${dpsRow.class}-${dpsRow.classSpec}`)}><img class="ml-2 size-5 object-contain" src={getClassIcon(dpsRow.class)} alt={`${dpsRow.class} class icon`} /></td>
            <td><span class="flex"><span class="cursor-pointer truncate" onclick={(error) => copyToClipboard(error, `#${dpsRow.uid}`)} {@attach tooltip(() => `UID: #${dpsRow.uid}`)}>{`${dpsRow.abilityScore && dpsRow.abilityScore !== 0 ? dpsRow.abilityScore : "??"} ${dpsRow.name?.trim() ? dpsRow.name : "Unknown Name"}`}</span></span></td>
            <td><AbbreviatedNumber val={Number(dpsRow.totalDmg)} /></td>
            <td><AbbreviatedNumber val={dpsRow.dps} /></td>
            <td>{dpsRow.dmgPct.toFixed(0)}<span class="text-tiny text-gray-300">%</span></td>
            <td>{dpsRow.critRate.toFixed(1)}<span class="text-tiny text-gray-300">%</span></td>
            <td>{dpsRow.critDmgRate.toFixed(1)}<span class="text-tiny text-gray-300">%</span></td>
            <td>{dpsRow.luckyRate.toFixed(1)}<span class="text-tiny text-gray-300">%</span></td>
            <td>{dpsRow.luckyDmgRate.toFixed(1)}<span class="text-tiny text-gray-300">%</span></td>
            <td>{dpsRow.hits}</td>
            <td>{dpsRow.hitsPerMinute.toFixed(1)}</td>
            <td class="-z-1 absolute left-0 h-7" style="background-color: {getClassColor(dpsRow.class)}; width: {dpsRow.dmgPct}vw;"></td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/if}
