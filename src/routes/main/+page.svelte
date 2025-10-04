<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';

  onMount(() => {
    goto(resolve("/main/settings"));
  });
</script>

<p>Redirecting to settings...</p>
