<script lang="ts">
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';

	import AppSidebar from './sidebar.svelte';
	import Header from './header.svelte';
	import { setupShortcuts } from './settings/shortcuts';
	import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
	import { goto } from '$app/navigation';

	let { children } = $props();

	$effect.pre(() => {
		(async () => {
			await setupShortcuts();
		})();
	});

	const appWebview = getCurrentWebviewWindow();
	appWebview.listen<string>('navigate', (event) => {
		const route = event.payload;
		goto(route);
	});
</script>

<div class="min-h-screen bg-background text-foreground">
	<Sidebar.Provider>
		<AppSidebar />
		<Sidebar.Inset>
			<Header />
			<main>
				<div class="mx-auto px-8 py-4">
					{@render children()}
				</div>
			</main>
		</Sidebar.Inset>
	</Sidebar.Provider>
</div>
