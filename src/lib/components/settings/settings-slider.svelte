<script lang="ts">
	import { Slider } from '$lib/components/ui/slider/index.js';

	let {
		label = '',
		description = '',
		value = $bindable(60),
		...restProps
	}: {
		label: string;
		description?: string | undefined;
		value: number;
	} = $props();
</script>

<label class="flex flex-col gap-2 py-0.5">
	<div>
		<div class="text-sm leading-tight font-medium">{label}</div>
		{#if description}
			<div class="mt-0.5 text-xs leading-snug text-muted-foreground">{description}</div>
		{/if}
	</div>
	<div class="flex flex-row items-center gap-4">
		<Slider type="single" bind:value min={0} max={120} step={5} {...restProps} class="flex-1" />
		<div class="w-12 text-right text-sm font-medium tabular-nums">
			{value}s
		</div>
	</div>
</label>
