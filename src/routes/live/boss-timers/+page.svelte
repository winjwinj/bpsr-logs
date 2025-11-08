<script lang="ts">
  import { onMount } from "svelte";
  import { commands } from "$lib/bindings";
  import type { Result } from "$lib/bindings";
  import { SETTINGS } from "$lib/settings-store";

  type MobHpData = {
    remote_id: string;
    server_id: number;
    hp_percent: number;
  };

  type CrowdsourcedMonster = {
    name: string;
    id: number;
    remote_id: string | null;
  };

  type CrowdsourcedMonsterOption = {
    name: string;
    id: number;
    remote_id: string;
  };

  const commandsExtended = commands as typeof commands & {
    getCrowdsourcedMonster: () => Promise<CrowdsourcedMonster | null>;
    getCrowdsourcedMonsterOptions: () => Promise<CrowdsourcedMonsterOption[]>;
    getCrowdsourcedMobHp: () => Promise<Result<MobHpData[], string>>;
    setBptimerStreamActive: (active: boolean) => Promise<Result<null, string>>;
    setCrowdsourcedMonsterRemote: (remoteId: string) => Promise<Result<null, string>>;
    getLocalPlayerLine: () => Promise<Result<number | null, string>>;
    markCurrentCrowdsourcedLineDead: () => Promise<Result<null, string>>;
  };

  let monsterOptions: CrowdsourcedMonsterOption[] = $state([]);
  let currentMonster: CrowdsourcedMonster | null = $state(null);
  let selectedRemoteId: string | null = $state(null);
  let mobHpData: MobHpData[] = $state([]);
  let currentLineId: number | null = $state(null);
  let streamActive = false;

  type CacheEntry = { data: MobHpData[]; timestamp: number };
  const mobHpCache = new Map<string, CacheEntry>();
  const CACHE_TTL_MS = 20_000;

  type HpChangeRecord = { hp: number; timestamp: number };
  const mobHpLastChange = new Map<string, HpChangeRecord>();
  const STALE_HP_THRESHOLD = 30;
  const STALE_HP_DURATION_MS = 30_000;
  let activeRemoteId: string | null = null;

  function mobKey(entry: MobHpData) {
    return `${entry.remote_id}:${entry.server_id}`;
  }

  function updateMobLastChange(entries: MobHpData[]) {
    const now = Date.now();
    const seen = new Set<string>();

    for (const entry of entries) {
      const key = mobKey(entry);
      seen.add(key);
      const record = mobHpLastChange.get(key);
      if (!record || record.hp !== entry.hp_percent) {
        mobHpLastChange.set(key, { hp: entry.hp_percent, timestamp: now });
      }
    }

    for (const key of Array.from(mobHpLastChange.keys())) {
      if (!seen.has(key)) {
        mobHpLastChange.delete(key);
      }
    }
  }

  function filterStaleEntries(entries: MobHpData[]) {
    const now = Date.now();
    return entries.filter((entry) => {
      if (entry.hp_percent > STALE_HP_THRESHOLD) {
        return true;
      }
      const record = mobHpLastChange.get(mobKey(entry));
      if (!record) {
        return true;
      }
      return now - record.timestamp < STALE_HP_DURATION_MS;
    });
  }

  function setStreamActive(active: boolean) {
    if (streamActive === active) {
      return;
    }

    streamActive = active;

    void commandsExtended
      .setBptimerStreamActive(active)
      .then((result) => {
        if (result.status === "error") {
          console.error("boss-timers/+page:setStreamActive", {
            error: result.error,
            active,
          });
        }
      })
      .catch((error) => {
        console.error("boss-timers/+page:setStreamActive", {
          error,
          active,
        });
      });
  }

  function clampPercent(value: number) {
    return Math.min(100, Math.max(0, value));
  }

  function barClass(percent: number) {
    if (percent === 0) return "bg-neutral-700";
    if (percent <= 30) return "bg-red-600/80";
    if (percent <= 60) return "bg-yellow-500/80";
    if (percent <= 99) return "bg-green-500/80";

    return "bg-green-500/20";
  }

  async function loadMonsterOptions() {
    try {
      monsterOptions = await commandsExtended.getCrowdsourcedMonsterOptions();
    } catch (error) {
      console.error("boss-timers/+page:loadMonsterOptions", { error });
      monsterOptions = [];
    }
  }

  onMount(() => {
    setStreamActive(SETTINGS.integration.state.bptimerUI);
    void loadMonsterOptions();
    void fetchData();
    const interval = setInterval(fetchData, 500);

    return () => {
      clearInterval(interval);
      setStreamActive(false);
    };
  });

  async function handleMonsterSelect(remoteId: string) {
    if (!remoteId || remoteId === currentMonster?.remote_id) {
      return;
    }

    try {
      const result = await commandsExtended.setCrowdsourcedMonsterRemote(remoteId);
      if (result.status === "error") {
        console.error("boss-timers/+page:setCrowdsourcedMonsterRemote", {
          error: result.error,
          remoteId,
        });
        return;
      }

      selectedRemoteId = remoteId;
      activeRemoteId = remoteId;
      mobHpLastChange.clear();
      mobHpCache.delete(remoteId);
      await fetchData();
    } catch (error) {
      console.error("boss-timers/+page:setCrowdsourcedMonsterRemote", {
        error,
        remoteId,
      });
    }
  }

  async function fetchData() {
    try {
      currentMonster = await commandsExtended.getCrowdsourcedMonster();
    } catch (error) {
      console.error("boss-timers/+page:getCrowdsourcedMonster", { error });
      currentMonster = null;
    }

    const remoteId = currentMonster?.remote_id ?? null;
    if (remoteId !== activeRemoteId) {
      mobHpLastChange.clear();
      activeRemoteId = remoteId;
    }
    selectedRemoteId = remoteId;

    try {
      const lineResult = await commandsExtended.getLocalPlayerLine();
      console.log("lineResult", lineResult);
      currentLineId = lineResult.status === "ok" ? lineResult.data ?? null : null;
    } catch (error) {
      console.error("boss-timers/+page:getLocalPlayerLine", { error });
      currentLineId = null;
    }

    if (!currentMonster) {
      mobHpData = [];
      activeRemoteId = null;
      mobHpLastChange.clear();
      return;
    }

    try {
      const result = await commandsExtended.getCrowdsourcedMobHp();
      const currentRemoteId = currentMonster.remote_id;

      if (result.status === "ok") {
        const data = result.data;

        if (currentRemoteId) {
          if (data.length > 0) {
            updateMobLastChange(data);
            const filteredData = filterStaleEntries(data);
            mobHpCache.set(currentRemoteId, {
              data,
              timestamp: Date.now(),
            });
            mobHpData = filteredData;
          } else {
            const cached = mobHpCache.get(currentRemoteId);
            if (cached && Date.now() - cached.timestamp <= CACHE_TTL_MS) {
              updateMobLastChange(cached.data);
              mobHpData = filterStaleEntries(cached.data);
            } else {
              mobHpCache.delete(currentRemoteId);
              mobHpData = [];
            }
          }
        } else {
          if (data.length > 0) {
            updateMobLastChange(data);
            mobHpData = filterStaleEntries(data);
          } else {
            mobHpData = [];
          }
        }
      } else {
        mobHpData = [];
      }
    } catch (error) {
      console.error("boss-timers/+page:getCrowdsourcedMobHp", { error });
      mobHpData = [];
    }
  }

  $effect(() => {
    const isEnabled = SETTINGS.integration.state.bptimerUI;
    setStreamActive(isEnabled);
    if (isEnabled) {
      fetchData();
    } else {
      currentMonster = null;
      selectedRemoteId = null;
      mobHpData = [];
    }
  });
</script>

<div class="flex h-full w-full flex-col justify-start gap-2 p-4">
    {#if currentMonster}
      <div class="flex w-full flex-col gap-2">
        {#if mobHpData.length > 0}
          <div class="grid w-full gap-2 grid-cols-10">
            {#each mobHpData
              .filter((mob) => mob.hp_percent > 0 || currentLineId === mob.server_id)
              .sort((a, b) => a.hp_percent - b.hp_percent)
              .slice(0, 20) as mob}
              <div class={`relative overflow-hidden rounded-md border ${currentLineId === mob.server_id ? "border-primary/80 ring-2 ring-primary/30" : "border-neutral-700"} bg-neutral-900/60 p-2 text-center text-xs`}>
                <div
                  class={`absolute inset-y-0 left-0 ${barClass(mob.hp_percent)} transition-all duration-200`}
                  style={`width: ${clampPercent(mob.hp_percent)}%;`}
                ></div>
                <div class="relative z-10 flex flex-col items-center gap-0.5">
                  <span class="font-medium text-neutral-200">{mob.server_id}</span>
                  {#if currentLineId === mob.server_id}
                    <span class="rounded bg-primary/20 px-1 text-[0.65rem] uppercase tracking-wide text-primary"></span>
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        {:else}
          <p class="text-xs text-neutral-600">No HP data available</p>
        {/if}
      </div>
    {:else}
      <p class="text-sm text-neutral-500">No timed monster found, select a monster to track:</p>
    {/if}
    <div class="flex w-full flex-col gap-1 md:w-1/2 lg:w-1/3">
      <select
        id="monster-select"
        class="w-full rounded-md border border-neutral-700 bg-neutral-900/60 px-2 py-1 text-sm text-neutral-200 outline-none transition-colors focus:border-neutral-500"
        disabled={monsterOptions.length === 0}
        value={selectedRemoteId ?? ""}
        onchange={(event) => handleMonsterSelect((event.currentTarget as HTMLSelectElement).value)}
      >
        <option value="" disabled selected={!selectedRemoteId}>
          {monsterOptions.length > 0 ? "Select monster" : "Loading monsters..."}
        </option>
        {#each monsterOptions as option}
          <option value={option.remote_id}>
            {option.name}
          </option>
        {/each}
      </select>
    </div>
    <p class="text-xs text-neutral-600">Use shortcut to mark current line dead</p>

</div>

