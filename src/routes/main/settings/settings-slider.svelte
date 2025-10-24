<script lang="ts">
  import { Slider } from "$lib/components/ui/slider/index.js";

  let {
    label = "",
    description = "",
    value = $bindable(60),
    ...restProps
  }: {
    label: string;
    description?: string | undefined;
    value: number;
  } = $props();
</script>

<label class="items-center">
  <div class="mb-4">
    <div>{label}</div>
    {#if description}
      <div class="text-muted-foreground text-sm">{description}</div>
    {/if}
  </div>
  <div class="flex flex-row">
    <Slider type="single" bind:value min={0} max={120} step={5} {...restProps} class="max-w-[70%]" />
    <div class="ml-4">
      <span>{value}s</span>
    </div>
  </div>
</label>
