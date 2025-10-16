<script lang="ts">
  import * as Tabs from "$lib/components/ui/tabs/index.js";
  import { SETTINGS } from "$lib/settings-store";
  import SettingsSelect from "./settings-select.svelte";
  import SettingsSlider from "./settings-slider.svelte";
  import SettingsSwitch from "./settings-switch.svelte";

  const SETTINGS_CATEGORY = "general";
</script>

<Tabs.Content value={SETTINGS_CATEGORY}>
  <SettingsSelect bind:selected={SETTINGS.general.state.showYourName} values={["Show Your Name", "Show Your Class", "Hide Your Name"]} label="Show Your Name" description="Show Your Class = replace your name with your class" />
  <SettingsSelect bind:selected={SETTINGS.general.state.showOthersName} values={["Show Others' Name", "Show Others' Class", "Hide Others' Name"]} label="Show Others' Name" description="Show Others' Class = replace others' name with their class" />
  <SettingsSwitch bind:checked={SETTINGS.general.state.showYourAbilityScore} label="Your Ability Score" description="Show your ability score" />
  <SettingsSwitch bind:checked={SETTINGS.general.state.showOthersAbilityScore} label="Others' Ability Score" description="Show others' ability score" />
  <SettingsSwitch bind:checked={SETTINGS.general.state.relativeToTopDPSPlayer} label="Relative to Top DPS - Player" description="Color bars are relative to top DPS player instead of all players. Useful for 20 man or World Bosses." />
  <SettingsSwitch bind:checked={SETTINGS.general.state.relativeToTopDPSSkill} label="Relative to Top DPS - Skill" description="Color bars are relative to top DPS skill instead of all skills. Useful for 20 man or World Bosses." />
  <SettingsSwitch bind:checked={SETTINGS.general.state.relativeToTopHealPlayer} label="Relative to Top Heal - Player" description="Color bars are relative to top healing player instead of all players. Useful for 20 man or World Bosses." />
  <SettingsSwitch bind:checked={SETTINGS.general.state.relativeToTopHealSkill} label="Relative to Top - Skill" description="Color bars are relative to top healing skill instead of all skills. Useful for 20 man or World Bosses." />
  <SettingsSwitch bind:checked={SETTINGS.general.state.shortenAbilityScore} label="Shorten Ability Score" description="Shortens the Ability Score" />
  <SettingsSlider bind:value={SETTINGS.general.state.resetElapsed} label="Reset after Elapsed Time" description="Amount of time to wait before the meter automatically resets the encounter. 0s = Never Resets"></SettingsSlider>
</Tabs.Content>
