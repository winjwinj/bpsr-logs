<script lang="ts">
	import { Switch } from '$lib/components/ui/switch/index.js';
	import type { WithoutChildrenOrChild } from '$lib/utils';
	import { Switch as SwitchPrimitive } from 'bits-ui';

	export type SettingsSwitchProps = {
		label: string;
		description?: string | undefined;
		checked: boolean;
	} & WithoutChildrenOrChild<SwitchPrimitive.RootProps>;

	let {
		label = '',
		description = '',
		checked = $bindable(false),
		...restProps
	}: SettingsSwitchProps = $props();
</script>

<div class="flex flex-row items-start gap-3 py-0.5">
	<Switch bind:checked {...restProps} class="mt-0.5 shrink-0" />
	<div class="min-w-0 flex-1">
		<div class="text-sm leading-tight font-medium">{label}</div>
		{#if description}
			<div class="mt-0.5 text-xs leading-snug text-muted-foreground">{description}</div>
		{/if}
	</div>
</div>
