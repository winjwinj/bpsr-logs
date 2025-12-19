<script lang="ts">
	import '../app.css';

	let { children } = $props();
</script>

<svelte:window oncontextmenu={(e) => e.preventDefault()} />

{@render children()}
