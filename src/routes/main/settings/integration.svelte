<script lang="ts">
  import * as Tabs from "$lib/components/ui/tabs/index.js";
  import { SETTINGS } from "$lib/settings-store";
  import SettingsSwitchDialog from "./settings-switch-dialog.svelte";
  import SettingsSwitch from "./settings-switch.svelte";


  const SETTINGS_CATEGORY = "integration";
</script>

<Tabs.Content value={SETTINGS_CATEGORY}>
  <SettingsSwitchDialog bind:checked={SETTINGS.integration.state.bptimer} label="BP Timer" description="World Boss and Magical Creature HP data for bptimer.com" />
  <SettingsSwitch bind:checked={SETTINGS.integration.state.bptimerUI} label="BP Timer UI (WIP)" description="Shows live HP data for the selected monster in the boss timers tab" />
</Tabs.Content>
