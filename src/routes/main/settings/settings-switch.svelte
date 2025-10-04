<script lang="ts">
  import { Switch } from "$lib/components/ui/switch/index.js";
  import { settings } from "$lib/store.svelte";

  let {
    settingsCategory,
    id,
    label,
    description,
  }: {
    settingsCategory: string,
    id: string;
    label: string;
    description: string;
  } = $props();
</script>

<label class="flex flex-row items-center">
  <Switch bind:checked={settings.state[settingsCategory][id]} />
  <div class="ml-4">
    <div>{label}</div>
    <div class="text-muted-foreground text-sm">{description}</div>
  </div>
</label>
