<script lang="ts">
	import { Button, type ButtonProps } from '$lib/components/ui/button/index.js';

	let {
		buttonLabel = '',
		label = '',
		description = '',
		...restProps
	}: {
		buttonLabel: string;
		label: string;
		description?: string | undefined;
	} & ButtonProps = $props();
</script>

<label class="flex flex-row items-start gap-3 py-0.5">
	<Button {...restProps} class="shrink-0">{buttonLabel}</Button>
	<div class="min-w-0 flex-1">
		<div class="text-sm leading-tight font-medium">{label}</div>
		{#if description}
			<div class="mt-0.5 text-xs leading-snug text-muted-foreground">{description}</div>
		{/if}
	</div>
</label>
