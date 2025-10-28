<script lang="ts">
  import { SETTINGS } from "$lib/settings-store";
  import { copyToClipboard, getClassIcon, tooltip } from "$lib/utils.svelte";
  import AbbreviatedNumber from "./abbreviated-number.svelte";

  let {
    className = "Unknown Class",
    classSpecName = "Unknown Spec",
    abilityScore = -1,
    name = "Unknown Name",
    uid = -1,
    localPlayerUid = -1,
  }: {
    className: string;
    classSpecName: string;
    abilityScore: number;
    name: string;
    uid: number;
    localPlayerUid: number;
  } = $props();

  let SETTINGS_YOUR_NAME = $derived(SETTINGS.general.state.showYourName);
  let SETTINGS_OTHERS_NAME = $derived(SETTINGS.general.state.showOthersName);

  // Derived helpers
  const isLocalPlayer = $derived(uid !== -1 && uid === localPlayerUid);
  const classDisplay = $derived(`${className}${classSpecName ? "-" : ""}${classSpecName}`);

  const nameDisplay = $derived(() => {
    if (isLocalPlayer) {
      if (SETTINGS_YOUR_NAME === "Show Your Class") {
        return `${classDisplay} (You)`;
      } else if (SETTINGS_YOUR_NAME === "Hide Your Name") {
        return "Hidden Name (You)";
      }
      return `${name} (You)`;
    } else {
      if (SETTINGS_OTHERS_NAME === "Show Others' Class") {
        return classDisplay;
      } else if (SETTINGS_OTHERS_NAME === "Hide Others' Name") {
        return "Hidden Name";
      }
      return name;
    }
  });

  const classIconDisplay = $derived(() => {
    if (isLocalPlayer) {
      if (SETTINGS_YOUR_NAME === "Hide Your Name") {
        return "Hidden Class";
      }
    } else {
      if (SETTINGS_OTHERS_NAME === "Hide Others' Name") {
        return "Hidden Class";
      }
    }
    return className;
  });
</script>

<div class="ml-2 flex">
  <img {@attach tooltip(() => classDisplay)} class="size-5 object-contain" src={getClassIcon(classIconDisplay())} />

  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <span class="ml-1 cursor-pointer truncate" onclick={(error) => copyToClipboard(error, `#${uid}`)} {@attach tooltip(() => `UID: #${uid}`)}>
    {#if abilityScore !== -1}
      {#if SETTINGS.general.state.shortenAbilityScore}
        {#if isLocalPlayer && SETTINGS.general.state.showYourAbilityScore}
          <AbbreviatedNumber num={abilityScore} />
        {:else if !isLocalPlayer && SETTINGS.general.state.showOthersAbilityScore}
          <AbbreviatedNumber num={abilityScore} />
        {/if}
      {:else}
        <span>{abilityScore}</span>
      {/if}
    {:else}
      ??
    {/if}
    {nameDisplay()}
  </span>
</div>
