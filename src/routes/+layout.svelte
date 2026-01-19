<script lang="ts">
	import '../app.css';
	import { SETTINGS } from '$lib/settings-store';
	import { onMount } from 'svelte';
	import Footer from '$lib/components/footer.svelte';
	import Header from '$lib/components/header.svelte';
	import { setupShortcuts } from '$lib/shortcuts';
	import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';

	let { children } = $props();
	let screenshotDiv: HTMLDivElement | undefined = $state();

	function applyTheme() {
		const theme = SETTINGS.accessibility.state.theme;
		if (theme === 'light') {
			document.documentElement.classList.add('light');
		} else {
			document.documentElement.classList.remove('light');
		}
	}

	// Apply theme on mount
	onMount(() => {
		applyTheme();
	});

	// Apply theme when it changes
	$effect(() => {
		applyTheme();
	});

	$effect.pre(() => {
		(async () => {
			await setupShortcuts();
		})();
	});

	// TODO: workaround, need to wait for svelte tanstack devs to respond
	onMount(() => {
		const interval = setInterval(refreshWindow, 5 * 60 * 1000); // refresh every 5m
		return () => clearInterval(interval);
	});
	function refreshWindow() {
		window.location.reload();
	}

	const appWebview = getCurrentWebviewWindow();
	appWebview.listen<string>('navigate', (event) => {
		const route = event.payload;
		goto(route);
	});
	appWebview.listen('toggle-settings', () => {
		if (page.url.pathname === '/settings') {
			goto('/');
		} else {
			goto('/settings');
		}
	});
</script>

<svelte:window oncontextmenu={(e) => e.preventDefault()} />

<div
	class="flex h-screen flex-col text-sm text-foreground"
	style={`background-color: oklch(from var(--background) l c h / ${SETTINGS.accessibility.state.transparencyOpacity / 100});`}
	bind:this={screenshotDiv}
>
	<Header {screenshotDiv} />
	<main class="flex-1 overflow-y-auto">
		{@render children()}
	</main>
	<Footer />
</div>

<style>
	:global {
		/* Hide scrollbars globally but keep scrolling functional */
		* {
			-ms-overflow-style: none; /* IE and Edge */
			scrollbar-width: none; /* Firefox */
		}
		*::-webkit-scrollbar {
			display: none; /* Chrome, Safari, Edge */
		}
		/* Make body and html transparent for live window to allow window transparency */
		html,
		body {
			background: transparent !important;
		}
	}
</style>
