<script lang="ts">
  import { commands } from "$lib/bindings";
  import { SETTINGS } from "$lib/settings-store";
  import { cn } from "$lib/utils";
  import { onMount } from "svelte";
  import Footer from "./footer.svelte";
  import Header from "./header.svelte";

  let { children } = $props();
  let screenshotDiv: HTMLDivElement | undefined = $state();

  // TODO: workaround, need to wait for svelte tanstack devs to respond
  onMount(() => {
    const interval = setInterval(refreshWindow, 5 * 60 * 1000); // refresh every 5m

    return () => clearInterval(interval);
  });
  function refreshWindow() {
    window.location.reload();
  }

  $effect(() => {
    if (SETTINGS.accessibility.state.blur) {
      commands.enableBlur();
    } else {
      commands.disableBlur();
    }
  });
</script>

<!-- flex flex-col min-h-screen → makes the page stretch full height and stack header, body, and footer. -->
<!-- flex-1 on <main> → makes the body expand to fill leftover space, pushing the footer down. -->
<div class="flex h-screen flex-col text-sm text-white" bind:this={screenshotDiv}>
  <Header {screenshotDiv} />
  <main class={cn("flex-1 overflow-y-auto", !SETTINGS.accessibility.state.transparency && "bg-neutral-900/25")}>
    {@render children()}
  </main>
  <Footer />
</div>

<style>
  :global {
    /* Hide scrollbars globally but keep scrolling functional */
    * {
      -ms-overflow-style: none; /* IE and Edge */
      scrollbar-width: none; /* Firefox */
    }
    *::-webkit-scrollbar {
      display: none; /* Chrome, Safari, Edge */
    }
  }
</style>
