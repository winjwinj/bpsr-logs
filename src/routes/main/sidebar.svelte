<script lang="ts">
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';

	import { getVersion } from '@tauri-apps/api/app';
	import { SIDEBAR_ROUTES } from './routes.svelte';
</script>

<Sidebar.Root>
	<Sidebar.Header>BPSR Logs</Sidebar.Header>
	<Sidebar.Separator />
	<Sidebar.Content>
		<Sidebar.Group>
			<Sidebar.GroupContent>
				<Sidebar.Menu>
					{#each Object.entries(SIDEBAR_ROUTES) as [href, route] (route.label)}
						<Sidebar.MenuItem>
							<Sidebar.MenuButton>
								{#snippet child({ props }: { props: Record<string, unknown> })}
									<a {href} {...props}>
										<route.icon />
										<span>{route.label}</span>
									</a>
								{/snippet}
							</Sidebar.MenuButton>
						</Sidebar.MenuItem>
					{/each}
				</Sidebar.Menu>
			</Sidebar.GroupContent>
		</Sidebar.Group>
	</Sidebar.Content>
	<Sidebar.Footer
		><span>v{#await getVersion()}X.Y.Z{:then version}{version}{/await}</span></Sidebar.Footer
	>
</Sidebar.Root>
