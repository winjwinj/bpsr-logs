<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { getVersion } from '@tauri-apps/api/app';
	import { openUrl } from '@tauri-apps/plugin-opener';
	import { cn } from '$lib/utils';
	import { SETTINGS } from '$lib/settings-store';

	const isDpsActive = $derived(
		page.url.pathname === '/' || page.url.pathname.startsWith('/skills')
	);
	const isHealActive = $derived(page.url.pathname.startsWith('/heal'));

	function navigateToDps() {
		goto('/');
	}

	function navigateToHeal() {
		goto('/heal');
	}
</script>

<footer
	class="sticky bottom-0 z-10 flex h-7 items-center justify-between border-t px-1.5"
	style={`background-color: oklch(from var(--card) l c h / ${SETTINGS.accessibility.state.transparencyOpacity / 100});`}
>
	<span class="flex h-full items-center gap-1">
		<button
			class={cn(
				'rounded-md px-2 py-0.5 text-xs font-medium transition-colors',
				isDpsActive
					? 'bg-primary text-primary-foreground'
					: 'bg-transparent hover:bg-accent hover:text-accent-foreground'
			)}
			onclick={navigateToDps}
		>
			DPS
		</button>
		<button
			class={cn(
				'rounded-md px-2 py-0.5 text-xs font-medium transition-colors',
				isHealActive
					? 'bg-primary text-primary-foreground'
					: 'bg-transparent hover:bg-accent hover:text-accent-foreground'
			)}
			onclick={navigateToHeal}
		>
			HEAL
		</button>
	</span>
	<button
		type="button"
		class="px-1.5 text-xs tracking-tighter text-muted-foreground underline-offset-2 hover:text-foreground hover:underline"
		onclick={() => openUrl('https://discord.gg/3UTC4pfCyC')}
		aria-label="Open BPSR Logs Discord"
	>
		BPSR Logs v{#await getVersion()}X.Y.Z{:then version}{version}{/await}
	</button>
</footer>
