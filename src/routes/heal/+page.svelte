<script lang="ts">
	import { onMount } from 'svelte';
	import { commands, type PlayersWindow } from '$lib/bindings';
	import { getClassColor } from '$lib/utils.svelte';
	import { goto } from '$app/navigation';
	import { getCoreRowModel } from '@tanstack/table-core';
	import { createSvelteTable } from '$lib/svelte-table';
	import { healPlayersColumnDefs } from '$lib/table-info';
	import FlexRender from '$lib/svelte-table/flex-render.svelte';
	import { SETTINGS } from '$lib/settings-store';

	onMount(() => {
		fetchData();
		const interval = setInterval(fetchData, 200);

		return () => clearInterval(interval);
	});

	let healPlayersWindow: PlayersWindow = $state({
		playerRows: [],
		localPlayerUid: -1,
		topValue: 0
	});

	async function fetchData() {
		healPlayersWindow = SETTINGS.misc.state.testingMode
			? await commands.getTestPlayerWindow()
			: await commands.getHealPlayerWindow();
	}

	const healTable = createSvelteTable({
		get data() {
			return healPlayersWindow.playerRows;
		},
		columns: healPlayersColumnDefs,
		getCoreRowModel: getCoreRowModel(),
		state: {
			get columnVisibility() {
				return SETTINGS.live.heal.players.state;
			}
		},
		meta: {
			get localPlayerUid() {
				return healPlayersWindow.localPlayerUid;
			}
		}
	});

	let SETTINGS_YOUR_NAME = $derived(SETTINGS.general.state.showYourName);
	let SETTINGS_OTHERS_NAME = $derived(SETTINGS.general.state.showOthersName);
</script>

<div class="relative">
	<table class="w-screen table-fixed">
		<thead class="sticky top-0 z-10 h-6">
			<tr
				class="border-b"
				style={`background-color: oklch(from var(--card) l c h / ${SETTINGS.accessibility.state.transparencyOpacity / 100});`}
			>
				{#each healTable.getHeaderGroups() as headerGroup (headerGroup.id)}
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
			{#each healTable.getRowModel().rows as row (row.id)}
				{@const isYou =
					row.original.uid !== -1 && row.original.uid == healPlayersWindow.localPlayerUid}
				{@const className = isYou
					? SETTINGS_YOUR_NAME !== 'Hide Your Name'
						? row.original.className
						: 'Hidden Class'
					: SETTINGS_OTHERS_NAME !== "Hide Others' Name"
						? row.original.className
						: 'Hidden Class'}
				<tr
					class="relative h-7 cursor-pointer overflow-hidden px-2 py-1 text-center transition-colors hover:bg-accent/50"
					onclick={() => goto(`/skills?playerUid=${row.original.uid}&type=heal`)}
				>
					{#each row.getVisibleCells() as cell (cell.id)}
						<td class="relative z-10 text-right"
							><FlexRender
								content={cell.column.columnDef.cell ?? 'UNKNOWN CELL'}
								context={cell.getContext()}
							/></td
						>
					{/each}
					<td
						class="pointer-events-none absolute top-0 left-0 h-7"
						style="background-color: {getClassColor(className)}; width: {(row.original.totalValue /
							healPlayersWindow.topValue) *
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
