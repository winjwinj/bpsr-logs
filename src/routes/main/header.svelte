<script lang="ts">
	import { Separator } from '$lib/components/ui/separator/index.js';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';

	import { page } from '$app/state';

	import { SIDEBAR_ROUTES } from './routes.svelte';

	let currentPage = $derived(
		(page.url.pathname as keyof typeof SIDEBAR_ROUTES) in SIDEBAR_ROUTES
			? SIDEBAR_ROUTES[page.url.pathname as keyof typeof SIDEBAR_ROUTES]
			: undefined
	);
</script>

<header
	class="flex h-(--header-height) shrink-0 items-center gap-2 border-b transition-[width,height] ease-linear group-has-data-[collapsible=icon]/sidebar-wrapper:h-(--header-height)"
>
	<div class="flex w-full items-center gap-1 px-4 lg:gap-2 lg:px-6">
		<Sidebar.Trigger class="-ml-1" />
		<Separator orientation="vertical" class="mx-2 data-[orientation=vertical]:h-4" />
		<h1>{currentPage?.label ?? 'UNKNOWN PAGE LABEL'}</h1>
	</div>
</header>
