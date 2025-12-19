<script lang="ts">
	import { cn } from '$lib/utils.js';
	import { Command as CommandPrimitive, useId } from 'bits-ui';

	let {
		ref = $bindable(null),
		class: className,
		children,
		heading,
		value,
		...restProps
	}: CommandPrimitive.GroupProps & {
		heading?: string;
	} = $props();
</script>

<CommandPrimitive.Group
	bind:ref
	data-slot="command-group"
	class={cn('overflow-hidden p-1 text-foreground', className)}
	value={value ?? heading ?? `----${useId()}`}
	{...restProps}
>
	{#if heading}
		<CommandPrimitive.GroupHeading class="px-2 py-1.5 text-xs font-medium text-muted-foreground">
			{heading}
		</CommandPrimitive.GroupHeading>
	{/if}
	{#if children}
		<CommandPrimitive.GroupItems {children} />
	{/if}
</CommandPrimitive.Group>
