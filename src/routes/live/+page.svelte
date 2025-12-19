<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';

	onMount(() => {
		// Default going to dps tab TODO: make it remember which tab it was in last? not sure if this is a good idea
		goto(resolve('/live/dps/'));
	});
</script>

<p>Redirecting to DPS...</p>
