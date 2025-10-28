<script lang="ts">
  import { Switch } from "$lib/components/ui/switch/index.js";
  import type { WithoutChildrenOrChild } from "$lib/utils";
  import { Switch as SwitchPrimitive } from "bits-ui";

  export type SettingsSwitchProps = {
    label: string;
    description?: string | undefined;
    checked: boolean;
  } & WithoutChildrenOrChild<SwitchPrimitive.RootProps>;

  let {
    label = "",
    description = "",
    checked = $bindable(false),
    ...restProps
  }: SettingsSwitchProps = $props();
</script>

<div class="flex flex-row items-center">
  <Switch bind:checked {...restProps} />
  <div class="ml-4">
    <div>{label}</div>
    {#if description}
      <div class="text-muted-foreground text-sm">{description}</div>
    {/if}
  </div>
</div>
