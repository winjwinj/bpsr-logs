<script lang="ts">
  import * as Tabs from "$lib/components/ui/tabs/index.js";
  import Accessibility from "./accessibility.svelte";
  import General from "./general.svelte";
  import Live from "./live.svelte";

  const settingsTabs = [
    { id: "general", label: "General" },
    { id: "accessibility", label: "Accessibility"},
    { id: "live", label: "Live" },
  ];
</script>

<Tabs.Root value="general">
  <Tabs.List>
    {#each settingsTabs as settingsTab (settingsTab)}
      <Tabs.Trigger value={settingsTab.id} >{settingsTab.label}</Tabs.Trigger>
    {/each}
  </Tabs.List>
  <General/>
  <Accessibility/>
  <Live/>
</Tabs.Root>
