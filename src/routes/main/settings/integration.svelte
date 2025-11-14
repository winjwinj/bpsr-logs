<script lang="ts">
  import * as Tabs from "$lib/components/ui/tabs/index.js";
  import { SETTINGS } from "$lib/settings-store";
  import SettingsSwitchDialog from "./settings-switch-dialog.svelte";


  const SETTINGS_CATEGORY = "integration";

  let initialBptimerValue = SETTINGS.integration.state.bptimer;
  let showRestartNote = $state(false);

  $effect(() => {
    showRestartNote = SETTINGS.integration.state.bptimer !== initialBptimerValue;
  });
</script>

<Tabs.Content value={SETTINGS_CATEGORY}>
  <SettingsSwitchDialog bind:checked={SETTINGS.integration.state.bptimer} label="BP Timer" description="World Boss and Magical Creature HP data for bptimer.com" />
  {#if showRestartNote}
    <p class="mt-2 text-sm text-yellow-600 dark:text-yellow-500">Restart the app for this change to take effect.</p>
  {/if}
</Tabs.Content>
