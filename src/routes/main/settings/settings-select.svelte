<script lang="ts">
	import CheckIcon from '@lucide/svelte/icons/check';
	import ChevronsUpDownIcon from '@lucide/svelte/icons/chevrons-up-down';
	import * as Command from '$lib/components/ui/command/index.js';
	import * as Popover from '$lib/components/ui/popover/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { cn } from '$lib/utils.js';

	let {
		label = '',
		description = '',
		selected = $bindable(''),
		values = []
	}: {
		label: string;
		description?: string | undefined;
		selected: string;
		values: string[];
	} = $props();

	let open = $state(false);
</script>

<label class="flex flex-row items-center">
	<!-- https://shadcn-svelte.com/docs/components/combobox -->
	<Popover.Root bind:open>
		<Popover.Trigger>
			<Button variant="outline" class="w-[200px] justify-between" role="combobox">
				{selected}
				<ChevronsUpDownIcon />
			</Button>
		</Popover.Trigger>
		<Popover.Content class="w-[200px] p-0">
			<Command.Root>
				<Command.List>
					<Command.Group>
						{#each values as value (value)}
							<Command.Item
								{value}
								onSelect={() => {
									selected = value;
									open = false;
								}}
							>
								<CheckIcon class={cn(selected !== value && 'text-transparent')} />
								{value}
							</Command.Item>
						{/each}
					</Command.Group>
				</Command.List>
			</Command.Root>
		</Popover.Content>
	</Popover.Root>
	<div class="ml-4">
		<div>{label}</div>
		{#if description}
			<div class="text-sm text-muted-foreground">{description}</div>
		{/if}
	</div>
</label>
