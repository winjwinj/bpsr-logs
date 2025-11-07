<script lang="ts">
  import { page } from "$app/state";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { getVersion } from "@tauri-apps/api/app";
  import { SETTINGS } from "$lib/settings-store";

  const showTimersTab = $derived(SETTINGS.integration.state.bptimerUI);
</script>

<footer class="sticky bottom-0 flex h-7 items-center justify-between bg-neutral-800/70 px-1.5">
  <span class="flex h-full items-center">
    <button
      class={`rounded-xs px-1.5 ${page.url.pathname.includes("dps") ? "bg-primary" : ""}`}
      onclick={() => {
        goto(resolve("/live/dps"));
      }}>DPS</button
    >
    <button
      class={`rounded-xs px-1.5 ${page.url.pathname.includes("heal") ? "bg-primary" : ""}`}
      onclick={() => {
        goto(resolve("/live/heal"));
      }}>HEAL</button
    >
    {#if showTimersTab}
      <button
        class={`rounded-xs px-1.5 ${page.url.pathname.includes("boss-timers") ? "bg-primary" : ""}`}
        onclick={() => {
          goto(resolve("/live/boss-timers"));
        }}>TIMERS</button
      >
    {/if}
  </span>
  <span class="px-1.5 tracking-tighter"><span>BPSR Logs v{#await getVersion()}X.Y.Z{:then version}{version}{/await}</span></span>
</footer>
