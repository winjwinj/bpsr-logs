<script lang="ts">
	import * as Tabs from '$lib/components/ui/tabs/index.js';
	import General from '$lib/components/settings/general.svelte';
	import Live from '$lib/components/settings/live.svelte';
	import Misc from '$lib/components/settings/misc.svelte';
	import Shortcuts from '$lib/components/settings/shortcuts.svelte';
	import { SETTINGS } from '$lib/settings-store';

	const settingsTabs = [
		{ id: 'general', label: 'General' },
		{ id: 'live', label: 'Display' },
		{ id: 'shortcuts', label: 'Shortcuts' },
		{ id: 'misc', label: 'Misc' }
	];

	let activeTab = $state('general');

	$effect(() => {
		const opacity = SETTINGS.accessibility.state.transparencyOpacity / 100;
		document.documentElement.style.setProperty('--card-opacity', opacity.toString());
	});
</script>

<div
	class="h-full w-full overflow-y-auto px-4 py-4"
	style={`background-color: oklch(from var(--background) l c h / ${SETTINGS.accessibility.state.transparencyOpacity / 100});`}
>
	<Tabs.Root bind:value={activeTab} class="w-full">
		<Tabs.List class="mb-4">
			{#each settingsTabs as settingsTab (settingsTab.id)}
				<Tabs.Trigger value={settingsTab.id}>{settingsTab.label}</Tabs.Trigger>
			{/each}
		</Tabs.List>
		<div class="settings-cards space-y-4">
			<General />
			<Live />
			<Shortcuts />
			<Misc />
		</div>
	</Tabs.Root>
</div>

<style>
	:global(.settings-cards [data-slot='card']) {
		background-color: oklch(from var(--card) l c h / var(--card-opacity, 0.6));
	}
</style>
