<script lang="ts">
	import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

	import CameraIcon from 'virtual:icons/lucide/camera';
	import TimerResetIcon from 'virtual:icons/lucide/timer-reset';
	import PauseIcon from 'virtual:icons/lucide/pause';
	import PlayIcon from 'virtual:icons/lucide/play';
	import MinusIcon from 'virtual:icons/lucide/minus';
	import XIcon from 'virtual:icons/lucide/x';
	import PointerIcon from 'virtual:icons/lucide/pointer';
	import SettingsIcon from 'virtual:icons/lucide/settings';
	import HomeIcon from 'virtual:icons/lucide/home';
	import SunIcon from 'virtual:icons/lucide/sun';
	import MoonIcon from 'virtual:icons/lucide/moon';

	import { onMount, tick } from 'svelte';
	import { commands, type HeaderInfo } from '$lib/bindings';
	import { takeScreenshot, tooltip } from '$lib/utils.svelte';
	import AbbreviatedNumber from '$lib/components/abbreviated-number.svelte';
	import { SETTINGS } from '$lib/settings-store';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';

	onMount(() => {
		fetchData();
		const interval = setInterval(fetchData, 200);
		return () => clearInterval(interval);
	});

	async function fetchData() {
		try {
			headerInfo = await commands.getHeaderInfo();
			if (
				SETTINGS.general.state.resetElapsed &&
				headerInfo.timeLastCombatPacketMs > 0 &&
				Date.now() - headerInfo.timeLastCombatPacketMs > SETTINGS.general.state.resetElapsed * 1000
			) {
				console.log(`Resetting as ${SETTINGS.general.state.resetElapsed}s has passed.`);
				await commands.resetEncounter();
				headerInfo = await commands.getHeaderInfo();
			}
		} catch (e) {
			console.error('Error fetching data: ', e);
		}
	}

	function formatElapsed(msElapsed: number) {
		const totalSeconds = Math.floor(Number(msElapsed) / 1000);
		const minutes = Math.floor((totalSeconds % 3600) / 60);
		const seconds = totalSeconds % 60;

		return `${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`;
	}

	let headerInfo: HeaderInfo = $state({
		totalDps: 0,
		totalDmg: 0,
		elapsedMs: 0,
		timeLastCombatPacketMs: 0
	});
	let isEncounterPaused = $state(false);
	let {
		screenshotDiv
	}: {
		screenshotDiv?: HTMLElement;
	} = $props();
	const appWindow = getCurrentWebviewWindow();

	const isSettingsActive = $derived(page.url.pathname === '/settings');
	const isLightMode = $derived(SETTINGS.accessibility.state.theme === 'light');

	function toggleTheme() {
		SETTINGS.accessibility.state.theme = isLightMode ? 'dark' : 'light';
	}

	function toggleSettings() {
		if (isSettingsActive) {
			goto('/');
		} else {
			goto('/settings');
		}
	}
</script>

<!-- justify-between to create left/right sides -->
<header
	data-tauri-drag-region
	class="sticky top-0 z-10 flex h-7 w-full items-center justify-between border-b px-1"
	style={`background-color: oklch(from var(--card) l c h / ${SETTINGS.accessibility.state.transparencyOpacity / 100});`}
>
	<!-- Left side -->
	<span class="flex items-center gap-2 text-xs font-medium">
		<span class="text-muted-foreground" {@attach tooltip(() => 'Time Elapsed')}>
			{formatElapsed(headerInfo.elapsedMs)}
		</span>
		<span class="flex items-center gap-1">
			<span class="text-muted-foreground" {@attach tooltip(() => 'Total Damage Dealt')}>
				T.DMG
			</span>
			<span class="font-semibold" {@attach tooltip(() => headerInfo.totalDmg.toLocaleString())}>
				<AbbreviatedNumber num={Number(headerInfo.totalDmg)} />
			</span>
		</span>
		<span class="flex items-center gap-1">
			<span class="text-muted-foreground" {@attach tooltip(() => 'Total Damage per Second')}>
				T.DPS
			</span>
			<span class="font-semibold" {@attach tooltip(() => headerInfo.totalDps.toLocaleString())}>
				<AbbreviatedNumber num={headerInfo.totalDps} />
			</span>
		</span>
	</span>
	<!-- Right side -->
	<span class="flex gap-0.5">
		<!-- TODO: add responsive clicks, toaster -->
		<button
			class="flex items-center justify-center rounded-md p-1 transition-colors hover:bg-accent hover:text-accent-foreground"
			onclick={async () => {
				await tick();
				await takeScreenshot(screenshotDiv);
			}}
			{@attach tooltip(() => 'Screenshot to Clipboard')}
		>
			<CameraIcon class="size-4" />
		</button>
		<button
			class="flex items-center justify-center rounded-md p-1 transition-colors hover:bg-accent hover:text-accent-foreground"
			onclick={async () => {
				await commands.resetEncounter();
				headerInfo = await commands.getHeaderInfo();
			}}
			{@attach tooltip(() => 'Reset Encounter')}
		>
			<TimerResetIcon class="size-4" />
		</button>
		<button
			class="flex items-center justify-center rounded-md p-1 transition-colors hover:bg-accent hover:text-accent-foreground"
			onclick={() => {
				commands.togglePauseEncounter();
				isEncounterPaused = !isEncounterPaused;
			}}
		>
			{#if isEncounterPaused}
				<PlayIcon class="size-4" {@attach tooltip(() => 'Resume Encounter')} />
			{:else}
				<PauseIcon class="size-4" {@attach tooltip(() => 'Pause Encounter')} />
			{/if}
		</button>
		<button
			class="flex items-center justify-center rounded-md p-1 transition-colors hover:bg-accent hover:text-accent-foreground"
			onclick={() => appWindow.setIgnoreCursorEvents(true)}
			{@attach tooltip(() => 'Clickthrough')}
		>
			<PointerIcon class="size-4" />
		</button>
		<button
			class="flex items-center justify-center rounded-md p-1 transition-colors hover:bg-accent hover:text-accent-foreground"
			onclick={toggleTheme}
			{@attach tooltip(() => (isLightMode ? 'Theme: Light' : 'Theme: Dark'))}
		>
			{#if isLightMode}
				<SunIcon class="size-4" />
			{:else}
				<MoonIcon class="size-4" />
			{/if}
		</button>
		<button
			class="flex items-center justify-center rounded-md p-1 transition-colors hover:bg-accent hover:text-accent-foreground"
			onclick={toggleSettings}
			{@attach tooltip(() => (isSettingsActive ? 'Home' : 'Settings'))}
		>
			{#if isSettingsActive}
				<HomeIcon class="size-4" />
			{:else}
				<SettingsIcon class="size-4" />
			{/if}
		</button>
		<button
			class="flex items-center justify-center rounded-md p-1 transition-colors hover:bg-accent hover:text-accent-foreground"
			onclick={() => appWindow.hide()}
			{@attach tooltip(() => 'Minimize')}
		>
			<MinusIcon class="size-4" />
		</button>
		<button
			class="flex items-center justify-center rounded-md p-1 transition-colors hover:bg-accent hover:text-accent-foreground"
			onclick={() => commands.quitApp()}
			{@attach tooltip(() => 'Quit')}
		>
			<XIcon class="size-4" />
		</button>
	</span>
</header>
