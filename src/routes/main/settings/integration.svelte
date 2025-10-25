<script lang="ts">
  import * as Tabs from "$lib/components/ui/tabs/index.js";
  import { SETTINGS } from "$lib/settings-store";
  import SettingsSwitchDialog from "./settings-switch-dialog.svelte";


  const SETTINGS_CATEGORY = "integration";
</script>

<Tabs.Content value={SETTINGS_CATEGORY}>
  <SettingsSwitchDialog bind:checked={SETTINGS.integration.state.bptimer} label="BP Timer" description="World Boss and Magical Creature HP data for bptimer.com" />
</Tabs.Content>
