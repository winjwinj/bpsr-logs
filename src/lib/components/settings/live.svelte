<script lang="ts">
	import * as Tabs from '$lib/components/ui/tabs/index.js';
	import * as Card from '$lib/components/ui/card/index.js';
	import SettingsSwitch from './settings-switch.svelte';
	import {
		dpsPlayersColumnDefs,
		dpsSkillsColumnDefs,
		healPlayersColumnDefs,
		healSkillsColumnDefs
	} from '$lib/table-info';
	import { SETTINGS } from '$lib/settings-store';
	import type { ColumnDef } from '@tanstack/table-core';
	import type { PlayerRow, SkillRow } from '$lib/bindings';

	const SETTINGS_CATEGORY = 'live';

	type ColumnWithAccessorKey<T> = ColumnDef<T> & { accessorKey: string };

	function hasAccessorKey<T>(col: ColumnDef<T>): col is ColumnWithAccessorKey<T> {
		return (
			'accessorKey' in col && typeof (col as { accessorKey?: unknown }).accessorKey === 'string'
		);
	}

	function getAccessorKey<T>(col: ColumnWithAccessorKey<T>): string {
		return col.accessorKey;
	}

	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	const dpsPlayersCols = (dpsPlayersColumnDefs as any[]).filter(
		hasAccessorKey
	) as ColumnWithAccessorKey<PlayerRow>[];
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	const dpsSkillsCols = (dpsSkillsColumnDefs as any[]).filter(
		hasAccessorKey
	) as ColumnWithAccessorKey<SkillRow>[];
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	const healPlayersCols = (healPlayersColumnDefs as any[]).filter(
		hasAccessorKey
	) as ColumnWithAccessorKey<PlayerRow>[];
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	const healSkillsCols = (healSkillsColumnDefs as any[]).filter(
		hasAccessorKey
	) as ColumnWithAccessorKey<SkillRow>[];
</script>

<Tabs.Content value={SETTINGS_CATEGORY} class="space-y-4">
	<Card.Root>
		<Card.Header>
			<Card.Title>DPS Meter</Card.Title>
		</Card.Header>
		<Card.Content class="space-y-3">
			<div>
				<h4 class="mb-2 text-sm font-medium">Player Columns</h4>
				<div class="grid grid-cols-1 gap-x-6 gap-y-1.5 sm:grid-cols-2 lg:grid-cols-3">
					{#each dpsPlayersCols as col (getAccessorKey(col))}
						{@const key = getAccessorKey(col) as keyof typeof SETTINGS.live.dps.players.state}
						<SettingsSwitch
							bind:checked={SETTINGS.live.dps.players.state[key]}
							label={col.meta?.label ?? 'LABEL MISSING'}
							description={col.meta?.description}
						/>
					{/each}
				</div>
			</div>
			<div>
				<h4 class="mb-2 text-sm font-medium">Skill Breakdown Columns</h4>
				<div class="grid grid-cols-1 gap-x-6 gap-y-1.5 sm:grid-cols-2 lg:grid-cols-3">
					{#each dpsSkillsCols as col (getAccessorKey(col))}
						{@const key = getAccessorKey(
							col
						) as keyof typeof SETTINGS.live.dps.skillBreakdown.state}
						<SettingsSwitch
							bind:checked={SETTINGS.live.dps.skillBreakdown.state[key]}
							label={col.meta?.label ?? 'LABEL MISSING'}
							description={col.meta?.description}
						/>
					{/each}
				</div>
			</div>
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title>Heal Meter</Card.Title>
		</Card.Header>
		<Card.Content class="space-y-3">
			<div>
				<h4 class="mb-2 text-sm font-medium">Player Columns</h4>
				<div class="grid grid-cols-1 gap-x-6 gap-y-1.5 sm:grid-cols-2 lg:grid-cols-3">
					{#each healPlayersCols as col (getAccessorKey(col))}
						{@const key = getAccessorKey(col) as keyof typeof SETTINGS.live.heal.players.state}
						<SettingsSwitch
							bind:checked={SETTINGS.live.heal.players.state[key]}
							label={col.meta?.label ?? 'LABEL MISSING'}
							description={col.meta?.description}
						/>
					{/each}
				</div>
			</div>
			<div>
				<h4 class="mb-2 text-sm font-medium">Skill Breakdown Columns</h4>
				<div class="grid grid-cols-1 gap-x-6 gap-y-1.5 sm:grid-cols-2 lg:grid-cols-3">
					{#each healSkillsCols as col (getAccessorKey(col))}
						{@const key = getAccessorKey(
							col
						) as keyof typeof SETTINGS.live.heal.skillBreakdown.state}
						<SettingsSwitch
							bind:checked={SETTINGS.live.heal.skillBreakdown.state[key]}
							label={col.meta?.label ?? 'LABEL MISSING'}
							description={col.meta?.description}
						/>
					{/each}
				</div>
			</div>
		</Card.Content>
	</Card.Root>
</Tabs.Content>
