<script lang="ts">
  import Button, { type ButtonProps } from "$lib/components/ui/button/button.svelte";

  let {
    buttonLabel = "",
    label = "",
    description = "",
    ...restProps
  }: {
    buttonLabel: string;
    label: string;
    description?: string | undefined;
  } & ButtonProps = $props();
</script>

<label class="flex flex-row items-center">
  <Button {...restProps}>{buttonLabel}</Button>
  <div class="ml-4">
    <div>{label}</div>
    {#if description}
      <div class="text-muted-foreground text-sm">{description}</div>
    {/if}
  </div>
</label>
