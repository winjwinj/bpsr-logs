<script lang="ts">
  import * as Tabs from "$lib/components/ui/tabs/index.js";
  import { SETTINGS } from "$lib/settings-store";
  import SettingsSelect from "./settings-select.svelte";
  import SettingsSlider from "./settings-slider.svelte";
  import SettingsSwitch from "./settings-switch.svelte";
  import { enable as enableAutostart, disable as disableAutostart} from '@tauri-apps/plugin-autostart';

  const SETTINGS_CATEGORY = "general";
</script>

<Tabs.Content value={SETTINGS_CATEGORY}>
  <SettingsSelect bind:selected={SETTINGS.general.state.showYourName} values={["Show Your Name", "Show Your Class", "Hide Your Name"]} label="Show Your Name" description="Show Your Class = replace your name with your class." />
  <SettingsSelect bind:selected={SETTINGS.general.state.showOthersName} values={["Show Others' Name", "Show Others' Class", "Hide Others' Name"]} label="Show Others' Name" description="Show Others' Class = replace others' name with their class." />
  <SettingsSwitch bind:checked={SETTINGS.general.state.showYourAbilityScore} label="Your Ability Score" description="Show your ability score." />
  <SettingsSwitch bind:checked={SETTINGS.general.state.showOthersAbilityScore} label="Others' Ability Score" description="Show others' ability score." />
  <SettingsSwitch bind:checked={SETTINGS.general.state.shortenAbilityScore} label="Shorten Ability Score" description="Shortens the Ability Score." />
  <SettingsSwitch bind:checked={SETTINGS.general.state.bossOnly} label="Boss Only Damage" description="Only track damage dealt to bosses." />
  <SettingsSlider bind:value={SETTINGS.general.state.resetElapsed} label="Reset after Elapsed Time" description="Amount of time to wait before the meter automatically resets the encounter. 0s = Never Resets."></SettingsSlider>
  <SettingsSwitch bind:checked={SETTINGS.general.state.autostart} label="Autostart" description="Automatically launch application at system startup." onCheckedChange={async (checked) => checked ? await enableAutostart() : await disableAutostart()} />
</Tabs.Content>
