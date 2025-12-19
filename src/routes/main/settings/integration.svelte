<script lang="ts">
	import * as Tabs from '$lib/components/ui/tabs/index.js';
	import { SETTINGS } from '$lib/settings-store';
	import SettingsSwitchDialog from './settings-switch-dialog.svelte';
	import { commands } from '$lib/bindings';

	const SETTINGS_CATEGORY = 'integration';

	let previousValue = $state(SETTINGS.integration.state.bptimer);

	$effect(() => {
		const currentValue = SETTINGS.integration.state.bptimer;
		if (currentValue !== previousValue) {
			previousValue = currentValue;
			commands.setBptimerEnabled(currentValue).catch((err: unknown) => {
				console.error('Failed to update bptimer enabled state:', err);
			});
		}
	});
</script>

<Tabs.Content value={SETTINGS_CATEGORY}>
	<SettingsSwitchDialog
		bind:checked={SETTINGS.integration.state.bptimer}
		label="BP Timer"
		description="World Boss and Magical Creature HP data for bptimer.com"
	/>
</Tabs.Content>
