<script lang="ts">
	import { cn } from '$lib/utils.js';
	import type { HTMLAttributes } from 'svelte/elements';

	let { class: className, children, ...restProps }: HTMLAttributes<HTMLDivElement> = $props();
</script>

<div
	data-slot="item-content"
	class={cn('flex gap-1 [&+[data-slot=item-content]]:flex-none', className)}
	{...restProps}
>
	{@render children?.()}
</div>
