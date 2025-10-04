<script lang="ts">
  import * as Tabs from "$lib/components/ui/tabs/index.js";
  import SettingsSwitch from "./settings-switch.svelte";
  import { SETTINGS_CONFIG_LIST, settings } from "$lib/store.svelte";

  const SETTINGS_CATEGORY = "live"

  console.log(`settings.state: ${settings.state}`)
  console.log(`settings.state[SETTINGS_CATEGORY]: ${settings.state[SETTINGS_CATEGORY]}`)
  console.log(`settings.state[SETTINGS_CATEGORY]["foolive"]: ${settings.state[SETTINGS_CATEGORY]["foolive"]}`)
</script>
  
<Tabs.Content value={SETTINGS_CATEGORY}>
  TEMPORARY TODO: REMOVE <br>
  {#each SETTINGS_CONFIG_LIST[SETTINGS_CATEGORY] as liveDpsConfig (liveDpsConfig)}
    {`${liveDpsConfig.id} = ${settings.state[SETTINGS_CATEGORY][liveDpsConfig.id]}`} <br>
  {/each}

  TEMPORARY TODO: REMOVE

  {#each SETTINGS_CONFIG_LIST[SETTINGS_CATEGORY] as liveDpsConfig (liveDpsConfig)}
    <SettingsSwitch settingsCategory={SETTINGS_CATEGORY} id={liveDpsConfig.id} label={liveDpsConfig.label} description={liveDpsConfig.description} />
  {/each}
  <!-- <h2 class="my-4 text-lg font-medium">Party Breakdown</h2> -->
  <!-- {#each SETTINGS as setting (setting)} -->
    <!-- <SettingsSwitch id={setting.id} label={setting.label} description={setting.description} /> -->
  <!-- {/each} -->
  <!-- <SettingsSwitch id="setting 1" label="setting 1 label" description="setting 1 description setting 1 description setting 1" /> -->

  <!-- <h2 class="my-4 text-lg font-medium">Skill Breakdown</h2> -->
  <!-- <SettingsSwitch id="clickthrough" label="Clickthrough label" description="Clickthrough description" /> -->
  <!-- <SettingsSwitch id="setting 1" label="setting 1 label" description="setting 1 description setting 1 description setting 1" /> -->
</Tabs.Content>
