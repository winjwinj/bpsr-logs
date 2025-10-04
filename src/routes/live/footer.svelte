<script lang="ts">
  import { page } from "$app/state";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { getVersion } from "@tauri-apps/api/app";
</script>

<footer class="sticky bottom-0 h-7 flex items-center justify-between bg-neutral-800/70 px-1.5">
  <span class="flex h-full items-center">
    <button
      class={`rounded-xs px-1.5 data-[state=active]:bg-background ${page.url.pathname.includes("dps") ? "data-[state=active]:bg-background" : ""}`}
      onclick={() => {
        goto(resolve("/live/dps"));
      }}>DPS</button
    >
    <button
      class={`rounded-xs px-1.5 data-[state=active]:bg-background ${page.url.pathname.includes("heal") ? "data-[state=active]:bg-background" : ""}`}
      onclick={() => {
        goto(resolve("/live/heal"));
      }}>HEAL</button
    >
  </span>
  <span class="px-1.5 tracking-tighter"><span>BPSR Logs v{#await getVersion()}X.Y.Z{:then version}{version}{/await}</span></span>
</footer>
