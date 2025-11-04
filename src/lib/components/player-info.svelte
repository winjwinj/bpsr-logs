<script lang="ts">
  import { SETTINGS } from "$lib/settings-store";
  import { copyToClipboard, getClassIcon, tooltip } from "$lib/utils.svelte";
  import { onMount, onDestroy } from 'svelte';
  import { getPlayerMetadata } from "$lib/player-metadata";
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

  // If the incoming name or ability score is a placeholder but we have a UID,
  // try to fetch historical metadata from the DB so bars don't initially show
  // "Unknown Name" or "??" for ability score. If it's still missing after the
  // first attempt, try again once after 5 seconds.
  let _retryTimer: ReturnType<typeof setTimeout> | null = null;

  async function tryLookupOnce(currentUid: number) {
    try {
      const data = await getPlayerMetadata(currentUid);
      if (data) {
        // Always update the live encounter cache with any discovered metadata
        // The backend will validate and apply the updates
        const { updateLivePlayerMetadata } = await import('$lib/player-metadata');
        await updateLivePlayerMetadata(
          currentUid,
          data.name,
          data.class,
          data.class_spec,
          data.ability_score
        );
        
        // Also update local component state for immediate UI feedback
        if (data.name && data.name !== "" && data.name !== "Unknown" && data.name !== "Unknown Name") {
          name = data.name;
        }
        if (data.class && (!className || className === "Unknown Class")) {
          className = data.class;
        }
        if (data.class_spec && (!classSpecName || classSpecName === "Unknown Spec")) {
          classSpecName = data.class_spec;
        }
        if (data.ability_score !== undefined && data.ability_score !== null && (abilityScore === -1 || abilityScore === null)) {
          abilityScore = data.ability_score as number;
        }
      }
    } catch (err) {
      // Non-fatal; leave placeholder if lookup fails
      console.warn('Failed to lookup player metadata for uid', currentUid, err);
    }
  }

  onMount(() => {
    const uidVal = uid;
    const needsLookupNow = uidVal !== -1 && (name === "" || name === "Unknown" || name === "Unknown Name" || abilityScore === -1 || abilityScore === null);
    if (!needsLookupNow) return;

    // First immediate attempt
    void tryLookupOnce(uidVal).then(() => {
      // After the first attempt, if still missing, schedule a single retry in 5s
      const stillMissing = (name === "" || name === "Unknown" || name === "Unknown Name") || (abilityScore === -1 || abilityScore === null);
      if (stillMissing) {
        _retryTimer = setTimeout(() => {
          void tryLookupOnce(uidVal);
          _retryTimer = null;
        }, 5000);
      }
    });
  });

  onDestroy(() => {
    if (_retryTimer) {
      clearTimeout(_retryTimer);
      _retryTimer = null;
    }
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
