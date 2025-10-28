<script lang="ts">
  import { commands } from "$lib/bindings";
  import * as Tabs from "$lib/components/ui/tabs/index.js";
  import { SETTINGS } from "$lib/settings-store";
  import SettingsSwitch from "./settings-switch.svelte";

  const SETTINGS_CATEGORY = "accessibility";
</script>

<Tabs.Content value={SETTINGS_CATEGORY}>
  <SettingsSwitch bind:checked={SETTINGS.accessibility.state.blur} label="Blur Meter Background" description="Adds background blur effect to live meter (Windows 10 only). Bad performance when resizing/dragging the window on Windows 11 build 22621+ unless you to disable Transparency effects in Windows accessibility SETTINGS." onCheckedChange={(checked) => checked ? commands.enableBlur() : commands.disableBlur()} />
  <SettingsSwitch bind:checked={SETTINGS.accessibility.state.transparency} label="Transparent Meter" description="Toggle transparent background for live meter." />
</Tabs.Content>
