<script lang="ts">
	import { onMount } from 'svelte';
	import { commands, type SkillsWindow } from '$lib/bindings';
	import { getClassColor } from '$lib/utils.svelte';
	import { page } from '$app/state';
	import { createSvelteTable, FlexRender } from '$lib/svelte-table';
	import {
		dpsPlayersColumnDefs,
		dpsSkillsColumnDefs,
		healPlayersColumnDefs,
		healSkillsColumnDefs
	} from '$lib/table-info';
	import { getCoreRowModel } from '@tanstack/table-core';
	import { SETTINGS } from '$lib/settings-store';

	// Validate playerUid and type from query params
	const playerUidParam = page.url.searchParams.get('playerUid');
	const playerUid: string =
		playerUidParam && /^-?\d+$/.test(playerUidParam) ? playerUidParam : '-1';

	const typeParam = page.url.searchParams.get('type');
	const statType: 'dps' | 'heal' = typeParam === 'heal' ? 'heal' : 'dps';

	const playersColumnDefs = statType === 'dps' ? dpsPlayersColumnDefs : healPlayersColumnDefs;
	const skillsColumnDefs = statType === 'dps' ? dpsSkillsColumnDefs : healSkillsColumnDefs;
	const settingsPath =
		statType === 'dps'
			? SETTINGS.live.dps.skillBreakdown.state
			: SETTINGS.live.heal.skillBreakdown.state;

	onMount(() => {
		fetchData();
		const interval = setInterval(fetchData, 200);
		return () => clearInterval(interval);
	});

	let skillBreakdownWindow: SkillsWindow | undefined = $state(undefined);

	async function fetchData() {
		try {
			const result = SETTINGS.misc.state.testingMode
				? await commands.getTestSkillWindow(playerUid)
				: statType === 'dps'
					? await commands.getDpsSkillWindow(playerUid)
					: await commands.getHealSkillWindow(playerUid);
			if (result.status !== 'ok') {
				console.warn('Failed to get skill window: ', result.error);
				return;
			} else {
				skillBreakdownWindow = result.data;
			}
		} catch (e) {
			console.error('Error fetching data: ', e);
		}
	}

	const inspectedPlayerTable = createSvelteTable({
		get data() {
			if (skillBreakdownWindow !== undefined) {
				return [skillBreakdownWindow.inspectedPlayer];
			} else {
				return [];
			}
		},
		columns: playersColumnDefs,
		getCoreRowModel: getCoreRowModel(),
		state: {
			get columnVisibility() {
				return settingsPath;
			}
		}
	});

	const skillBreakdownTable = createSvelteTable({
		get data() {
			if (skillBreakdownWindow !== undefined) {
				return skillBreakdownWindow.skillRows;
			} else {
				return [];
			}
		},
		columns: skillsColumnDefs,
		getCoreRowModel: getCoreRowModel(),
		state: {
			get columnVisibility() {
				return settingsPath;
			}
		}
	});
</script>

<svelte:window oncontextmenu={() => window.history.back()} />

{#if skillBreakdownWindow !== undefined}
	<div class="relative">
		<table class="w-screen table-fixed">
			<thead class="sticky top-0 z-10 h-6">
				<tr
					class="border-b"
					style={`background-color: oklch(from var(--card) l c h / ${SETTINGS.accessibility.state.transparencyOpacity / 100});`}
				>
					{#each skillBreakdownTable.getHeaderGroups() as headerGroup (headerGroup.id)}
						{#each headerGroup.headers as header (header.id)}
							<th class={header.column.columnDef.meta?.class}
								><FlexRender
									content={header.column.columnDef.header ?? 'UNKNOWN HEADER'}
									context={header.getContext()}
								/></th
							>
						{/each}
					{/each}
				</tr>
			</thead>
			<tbody>
				{#each inspectedPlayerTable.getRowModel().rows as row (row.id)}
					<tr class="relative h-7 overflow-hidden bg-accent/20 px-2 py-1 text-center">
						{#each row.getVisibleCells() as cell (cell.id)}
							<td class="relative z-10"
								><FlexRender
									content={cell.column.columnDef.cell ?? 'UNKNOWN CELL'}
									context={cell.getContext()}
								/></td
							>
						{/each}
						<td
							class="pointer-events-none absolute top-0 left-0 h-7"
							style="background-color: {getClassColor(
								skillBreakdownWindow.inspectedPlayer.className
							)}; width: 100vw; opacity: {Math.max(
								0.3,
								SETTINGS.accessibility.state.transparencyOpacity / 100
							)}; z-index: 0;"
						></td>
					</tr>
				{/each}
				{#each skillBreakdownTable.getRowModel().rows as row (row.id)}
					<tr
						class="relative h-7 overflow-hidden px-2 py-1 text-center transition-colors hover:bg-accent/30"
					>
						{#each row.getVisibleCells() as cell (cell.id)}
							<td class="relative z-10"
								><FlexRender
									content={cell.column.columnDef.cell ?? 'UNKNOWN CELL'}
									context={cell.getContext()}
								/></td
							>
						{/each}
						<td
							class="pointer-events-none absolute top-0 left-0 h-7"
							style="background-color: {getClassColor(
								skillBreakdownWindow.inspectedPlayer.className
							)}; width: {(row.original.totalValue / skillBreakdownWindow.topValue) *
								100}%; opacity: {Math.max(
								0.3,
								SETTINGS.accessibility.state.transparencyOpacity / 100
							)}; z-index: 0;"
						></td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
{/if}
