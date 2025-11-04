<script lang="ts">
  import { commands, type EncounterRecord } from "$lib/bindings";
  import { selectedEncounter } from "$lib/encounter-store";
  import { onMount } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import TrashIcon from "virtual:icons/lucide/trash-2";

  let encounters: EncounterRecord[] = $state([]);
  let loading = $state(true);
  let searchQuery = $state("");

  onMount(async () => {
    await loadEncounters();
  });

  async function loadEncounters() {
    loading = true;
    try {
      const result = await commands.getAllEncounterHistory();
      if (result.status !== "ok") {
        console.error("Failed to load encounters:", result.error);
        return;
      }
      encounters = result.data;
    } catch (error) {
      console.error("Failed to load encounters:", error);
    } finally {
      loading = false;
    }
  }

  async function deleteEncounter(encounterId: number) {
    if (confirm("Are you sure you want to delete this encounter?")) {
      try {
        const result = await commands.deleteEncounterHistory(encounterId);
        if (result.status !== "ok") {
          console.error("Failed to delete encounter:", result.error);
          return;
        }
        await loadEncounters();
      } catch (error) {
        console.error("Failed to delete encounter:", error);
      }
    }
  }

  function selectEncounter(encounter: EncounterRecord) {
    selectedEncounter.set({
      type: "historical",
      encounterId: encounter.id,
    });
  }

  function getFilteredEncounters() {
    return encounters.filter((e) => {
      return !searchQuery || e.id.toString().includes(searchQuery.toLowerCase());
    });
  }

  function formatDuration(ms: number): string {
    const seconds = Math.floor(ms / 1000);
    const minutes = Math.floor(seconds / 60);
    const secs = seconds % 60;
    if (minutes > 0) {
      return `${minutes}m ${secs}s`;
    }
    return `${secs}s`;
  }

  function formatDate(dateStr: string): string {
    try {
      const date = new Date(dateStr);
      const now = new Date();
      const diffMs = now.getTime() - date.getTime();
      const diffSecs = Math.floor(diffMs / 1000);
      const diffMins = Math.floor(diffSecs / 60);
      const diffHours = Math.floor(diffMins / 60);
      const diffDays = Math.floor(diffHours / 24);

      if (diffSecs < 60) return "just now";
      if (diffMins < 60) return `${diffMins} minute${diffMins > 1 ? "s" : ""} ago`;
      if (diffHours < 24) return `${diffHours} hour${diffHours > 1 ? "s" : ""} ago`;
      if (diffDays < 7) return `${diffDays} day${diffDays > 1 ? "s" : ""} ago`;
      
      return date.toLocaleDateString();
    } catch {
      return dateStr;
    }
  }
</script>

<div class="space-y-6">
  <div>
    <h1 class="text-3xl font-bold">Encounter History</h1>
    <p class="text-muted-foreground mt-2">Review and analyze past encounters</p>
  </div>

  {#if loading}
    <div class="flex items-center justify-center py-12">
      <div class="text-muted-foreground">Loading encounters...</div>
    </div>
  {:else if encounters.length === 0}
    <div class="flex items-center justify-center py-12 bg-muted/50 rounded-lg">
      <div class="text-muted-foreground">No encounters recorded yet. Start a combat and reset to save encounters!</div>
    </div>
  {:else}
    <div class="space-y-4">
      <Input
        placeholder="Search by encounter ID..."
        bind:value={searchQuery}
        class="max-w-sm"
      />

      <div class="overflow-x-auto">
        <table class="w-full text-sm">
          <thead>
            <tr class="border-b bg-muted/50">
              <th class="px-4 py-2 text-left font-semibold">ID</th>
              <th class="px-4 py-2 text-left font-semibold">Start Time</th>
              <th class="px-4 py-2 text-right font-semibold">Duration</th>
              <th class="px-4 py-2 text-right font-semibold">Total Damage</th>
              <th class="px-4 py-2 text-right font-semibold">Total Healing</th>
              <th class="px-4 py-2 text-center font-semibold">Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each getFilteredEncounters() as encounter (encounter.id)}
              <tr class="border-b hover:bg-muted/50 transition-colors cursor-pointer" onclick={() => selectEncounter(encounter)}>
                <td class="px-4 py-2 font-mono">{encounter.id}</td>
                <td class="px-4 py-2">{formatDate(encounter.start_time)}</td>
                <td class="px-4 py-2 text-right">{formatDuration(encounter.duration_ms)}</td>
                <td class="px-4 py-2 text-right font-mono">{encounter.total_damage.toLocaleString()}</td>
                <td class="px-4 py-2 text-right font-mono">{encounter.total_healing.toLocaleString()}</td>
                <td class="px-4 py-2 text-center">
                  <Button
                    variant="ghost"
                    size="sm"
                    onclick={(e) => {
                      e.stopPropagation();
                      deleteEncounter(encounter.id);
                    }}
                    class="hover:text-destructive"
                  >
                    <TrashIcon class="w-4 h-4" />
                  </Button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>
  {/if}
</div>
