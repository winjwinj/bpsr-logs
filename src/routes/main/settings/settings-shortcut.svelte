<script lang="ts">
	import { Switch } from '$lib/components/ui/switch/index.js';

	let {
		checked = $bindable(false),
		label = ''
	}: {
		checked: boolean | undefined;
		label: string;
	} = $props();
</script>

<label class="flex flex-row items-center">
	<Switch bind:checked />
	<div class="ml-4">
		<div>{label}</div>
	</div>
</label>
