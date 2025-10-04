<script lang="ts">
  import Footer from "./footer.svelte";
  import Header from "./header.svelte";

  let { children } = $props();
  let screenshotDiv: HTMLDivElement | undefined = $state();
</script>

<!-- flex flex-col min-h-screen → makes the page stretch full height and stack header, body, and footer. -->
<!-- flex-1 on <main> → makes the body expand to fill leftover space, pushing the footer down. -->
<div class="flex flex-col h-screen text-white text-sm" bind:this={screenshotDiv}>
  <Header {screenshotDiv} />
  <main class="flex-1 overflow-y-auto">
    {@render children()}
  </main>
  <Footer/>
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
