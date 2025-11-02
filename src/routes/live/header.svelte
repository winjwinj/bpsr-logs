<script lang="ts">
  import { getCurrentWebviewWindow, WebviewWindow } from "@tauri-apps/api/webviewWindow";

  import CameraIcon from "virtual:icons/lucide/camera";
  import TimerResetIcon from "virtual:icons/lucide/timer-reset";
  import PauseIcon from "virtual:icons/lucide/pause";
  import PlayIcon from "virtual:icons/lucide/play";
  import MinusIcon from "virtual:icons/lucide/minus";
  import PointerIcon from "virtual:icons/lucide/pointer";
  import SettingsIcon from "virtual:icons/lucide/settings";
  import RefreshCwIcon from "virtual:icons/lucide/refresh-cw";
  import ChevronDownIcon from "virtual:icons/lucide/chevron-down";

  import { onMount, tick } from "svelte";
  import { commands, type HeaderInfo, type EncounterRecord } from "$lib/bindings";
  import { takeScreenshot, tooltip } from "$lib/utils.svelte";
  import AbbreviatedNumber from "$lib/components/abbreviated-number.svelte";
  import { emitTo } from "@tauri-apps/api/event";
  import { SETTINGS } from "$lib/settings-store";
  import { selectedEncounter, liveFallbackEncounterId } from "$lib/encounter-store";

  onMount(() => {
    fetchData();
    loadEncounters();
    const interval = setInterval(fetchData, 200);
    // Only load encounters when dropdown is opened (see below)
    
    // Handle click outside dropdown
    const handleClickOutside = (e: MouseEvent) => {
      if (dropdownElement && !dropdownElement.contains(e.target as Node)) {
        showEncounterDropdown = false;
      }
    };
    
    document.addEventListener("click", handleClickOutside);
    
    return () => {
      clearInterval(interval);
      document.removeEventListener("click", handleClickOutside);
    };
  });

  let hasReset = false;
  // When true we will freeze the header/UI and avoid replacing it with empty packets
  // until a new packet with any non-zero damage/heal arrives. Frontend-only guard.
  let freezeUntilNewCombat = $state(false);
  // Only attempt header historical fallback once per idle period
  let headerFallbackLoaded = $state(false);
  let recentEncounters: EncounterRecord[] = $state([]);
  let showEncounterDropdown = $state(false);
  let dropdownElement: HTMLElement | null = $state(null);
  let buttonElement: HTMLElement | null = $state(null);

  async function loadEncounters() {
    try {
      const result = await commands.getAllEncounterHistory();
      console.log("Loaded encounters result:", result);
      
      if (result.status !== "ok") {
        console.error("Failed to load encounters:", result.error);
        return;
      }
      
      const encounters = result.data;
      console.log("[snapshot] Recent encounters (limited to 10):", encounters);
      
      recentEncounters = encounters.slice(0, 10);
    } catch (error) {
      console.error("Failed to load encounters:", error);
    }
  }

  function switchEncounter(encounter: EncounterRecord) {
    console.log("Switching to encounter:", encounter.id);
    // Clear any live fallback since user explicitly selected a historical encounter
    liveFallbackEncounterId.set(null);
    selectedEncounter.set({
      type: "historical",
      encounterId: encounter.id,
    });
    showEncounterDropdown = false;
  }

  function switchToLive() {
    console.log("Switching to live");
    // Clear any existing fallback because the user explicitly chose Live
    liveFallbackEncounterId.set(null);
    selectedEncounter.set({
      type: "live",
    });
    showEncounterDropdown = false;
  }

  async function fetchData() {
    // If the configured elapsed reset fires, don't trigger a backend reset from the UI.
    // Instead just freeze the UI until a new combat with any non-zero damage/heal arrives.
    // This is frontend-only and prevents the immediate reload/clear behavior when combat ends.
    if (SETTINGS.general.state.resetElapsed && !hasReset && Date.now() - headerInfo.timeLastCombatPacketMs > SETTINGS.general.state.resetElapsed * 1000) {
      // Only act if there's actual combat data present
      if (headerInfo.totalDmg > 0 || headerInfo.elapsedMs > 0) {
        hasReset = true;
        freezeUntilNewCombat = true;
        console.log(`Elapsed reset threshold passed; invoking backend reset and freezing UI until new combat packet.`);
        // Call backend reset to close out the encounter (this should create the historical entry server-side)
        // but do NOT reload or clear the UI here — we'll preserve the last non-empty view until new data arrives.
        try {
          await commands.resetEncounter();
          console.log('Backend resetEncounter() invoked');
          // Immediately query the history to pick up the saved encounter and set the
          // shared live fallback id so other pages can display the historical data
          // right after the backend reset (avoids a timing window where navigation
          // happens before pages can discover the new history entry).
          try {
            const historyRes = await commands.getAllEncounterHistory();
            if (historyRes.status === 'ok' && historyRes.data && historyRes.data.length > 0) {
              const lastEncounter = historyRes.data[0]!;
              liveFallbackEncounterId.set(lastEncounter.id ?? null);
              console.log('Set liveFallbackEncounterId to', lastEncounter.id);
            }
          } catch (e) {
            console.warn('Failed to load encounter history immediately after reset:', e);
          }
        } catch (err) {
          console.error('Failed to invoke resetEncounter():', err);
        }
      }
    }
    try {
      const result = await commands.getHeaderInfo();
      if (result.status !== "ok") {
        console.warn("Failed to get header: ", result.error);
        return;
      } else {
        // Don't overwrite an existing non-empty header with an empty/zeroed header from the backend.
        // This keeps the elapsed/total values visible until a new packet with any non-zero damage/heal arrives.
        const fetched = result.data;
        const fetchedHasCombat = (fetched.totalDmg ?? 0) > 0 || (fetched.elapsedMs ?? 0) > 0;
        const currentHasCombat = (headerInfo.totalDmg ?? 0) > 0 || (headerInfo.elapsedMs ?? 0) > 0;

        if (fetchedHasCombat || !currentHasCombat) {
          headerInfo = fetched;
          // If we were frozen waiting for new combat, unfreeze now that real data arrived.
          if (fetchedHasCombat) {
            freezeUntilNewCombat = false;
            if (hasReset) {
              hasReset = false;
              console.log("Fresh combat packet received after frontend-freeze");
            }
            // allow header fallback to run again later if needed
            headerFallbackLoaded = false;
          }
        } else {
          // Keep previous headerInfo until new combat data arrives.
          // If there is no live combat and we don't already have a header, fall back to the most recent saved encounter
          if (!headerFallbackLoaded) {
            headerFallbackLoaded = true;
            try {
              const historyRes = await commands.getAllEncounterHistory();
              if (historyRes.status === 'ok' && historyRes.data && historyRes.data.length > 0) {
                const lastEncounter = historyRes.data[0]!;
                // Map EncounterRecord -> HeaderInfo-like display values
                const durationMs = lastEncounter.duration_ms ?? 0;
                const totalDmg = lastEncounter.total_damage ?? 0;
                const totalDps = durationMs > 0 ? Math.round((totalDmg / (durationMs / 1000)) * 100) / 100 : 0;

                headerInfo = {
                  totalDps: totalDps,
                  totalDmg: totalDmg,
                  elapsedMs: durationMs,
                  // Keep the timeLastCombatPacketMs as-is from the previous header (don't overwrite with historical timestamp)
                  timeLastCombatPacketMs: headerInfo.timeLastCombatPacketMs,
                };
              }
            } catch (err) {
              // swallow; we simply keep the existing headerInfo
              console.error('Failed to load last historical header as fallback:', err);
            }
          }
        }
        // console.log("header: ", +Date.now(), $state.snapshot(headerInfo));
      }
    } catch (e) {
      console.error("Error fetching data: ", e);
    }
  }

  function formatElapsed(msElapsed: number) {
    const totalSeconds = Math.floor(Number(msElapsed) / 1000);
    const minutes = Math.floor((totalSeconds % 3600) / 60);
    const seconds = totalSeconds % 60;

    return `${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
  }

  let headerInfo: HeaderInfo = $state({
    totalDps: 0,
    totalDmg: 0,
    elapsedMs: 0,
    timeLastCombatPacketMs: Date.now(), // TODO: tempfix
  });
  let isEncounterPaused = $state(false);
  const {
    screenshotDiv,
  }: {
    screenshotDiv?: HTMLElement;
  } = $props();
  const appWindow = getCurrentWebviewWindow();

  async function openSettings() {
    const mainWindow = await WebviewWindow.getByLabel("main");
    if (mainWindow !== null) {
      await mainWindow?.unminimize();
      await mainWindow?.show();
      await mainWindow?.setFocus();
      await emitTo("main", "navigate", "/main/settings"); // main/+layout.svelte
    }
  }
</script>

<!-- justify-between to create left/right sides -->
<header data-tauri-drag-region class="sticky top-0 z-40 flex h-7 w-full items-center justify-between bg-neutral-900/80 px-1">
  <!-- Left side -->
  <span class="flex items-center gap-2">
    <button
      onclick={() => {
        commands.hardReset();
        window.location.reload();
      }}
      {@attach tooltip(() => "Temp Fix: Hard Reset")}><RefreshCwIcon /></button
    >
    
    <!-- Encounter Selector -->
    <div class="relative z-50" bind:this={dropdownElement}>
      <button
        onclick={async () => {
          if (!showEncounterDropdown) {
            // Load encounters when opening dropdown
            await loadEncounters();
          }
          showEncounterDropdown = !showEncounterDropdown;
        }}
        bind:this={buttonElement}
        class="flex items-center gap-1 px-2 py-0.5 rounded hover:bg-neutral-800 transition-colors text-xs relative"
        {@attach tooltip(() => $selectedEncounter.type === "live" ? "Currently viewing: Live" : `Currently viewing: Encounter #${$selectedEncounter.encounterId}`)}
      >
        {#if $selectedEncounter.type === "live"}
          <span>📊 Live</span>
        {:else}
          <span>📜 #{$selectedEncounter.encounterId}</span>
        {/if}
        <ChevronDownIcon class="w-3 h-3" />
      </button>

      {#if showEncounterDropdown}
        <!-- Dropdown menu with solid background to prevent bleed-through -->
        <div style="position: absolute; top: 100%; left: 0; margin-top: 0.25rem; background-color: rgb(5, 5, 5); border: 1px solid rgb(55, 65, 81); border-radius: 0.375rem; box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.9); z-index: 60; min-width: 12rem; max-height: 24rem; overflow-y: auto; isolation: isolate;" bind:this={dropdownElement}>
          <!-- Reset Encounters button - AT TOP -->
          <div style="padding: 0.375rem; border-bottom: 1px solid rgb(55, 65, 81);">
            <button
              onclick={async () => {
                if (confirm("Are you sure you want to reset all encounters? This will clear all stored player data and encounter history.")) {
                  try {
                    await commands.clearAllData();
                    selectedEncounter.set({ type: "live" });
                    showEncounterDropdown = false;
                    await loadEncounters();
                  } catch (error) {
                    console.error("Failed to reset encounters:", error);
                  }
                }
              }}
              style="width: 100%; padding: 0.5rem; text-align: left; color: rgb(239, 68, 68); font-size: 0.75rem; display: flex; justify-content: center; align-items: center; cursor: pointer; border: none; background: none; font-weight: 500;"
              onmouseover={(e) => (e.currentTarget.style.backgroundColor = "rgba(239, 68, 68, 0.1)")}
              onmouseout={(e) => (e.currentTarget.style.backgroundColor = "transparent")}
              onfocus={(e) => (e.currentTarget.style.backgroundColor = "rgba(239, 68, 68, 0.1)")}
              onblur={(e) => (e.currentTarget.style.backgroundColor = "transparent")}
            >
              🔄 Reset Encounters
            </button>
          </div>

          <!-- Live option - ALWAYS VISIBLE -->
          <div style="padding: 0.375rem; border-bottom: 1px solid rgb(55, 65, 81); background-color: rgb(5, 5, 5);">
            <button
              onclick={() => {
                console.log("Live button clicked");
                switchToLive();
              }}
              onkeydown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                  e.preventDefault();
                  switchToLive();
                }
              }}
              style="width: 100%; padding: 0.5rem; text-align: left; color: white; font-size: 0.75rem; display: flex; justify-content: space-between; align-items: center; cursor: pointer; border: none; background: none;"
              onmouseover={(e) => (e.currentTarget.style.backgroundColor = "rgb(31, 41, 55)")}
              onmouseout={(e) => (e.currentTarget.style.backgroundColor = "transparent")}
              onfocus={(e) => (e.currentTarget.style.backgroundColor = "rgb(31, 41, 55)")}
              onblur={(e) => (e.currentTarget.style.backgroundColor = "transparent")}
            >
              <span>📊 Live</span>
              {#if $selectedEncounter.type === "live"}
                <span style="color: rgb(34, 197, 94);">✓</span>
              {/if}
            </button>
          </div>

          <!-- Recent encounters -->
          {#if recentEncounters.length > 0}
            {#each recentEncounters as encounter (encounter.id)}
              <div style="padding: 0.375rem; border-bottom: 1px solid rgb(55, 65, 81);">
                <button
                  onclick={() => {
                    console.log("Encounter clicked:", encounter.id);
                    switchEncounter(encounter);
                  }}
                  onkeydown={(e) => {
                    if (e.key === 'Enter' || e.key === ' ') {
                      e.preventDefault();
                      switchEncounter(encounter);
                    }
                  }}
                  style="width: 100%; padding: 0.375rem; text-align: left; color: white; font-size: 0.75rem; display: flex; justify-content: space-between; align-items: center; cursor: pointer; border: none; background: none;"
                  onmouseover={(e) => (e.currentTarget.style.backgroundColor = "rgb(31, 41, 55)")}
                  onmouseout={(e) => (e.currentTarget.style.backgroundColor = "transparent")}
                  onfocus={(e) => (e.currentTarget.style.backgroundColor = "rgb(31, 41, 55)")}
                  onblur={(e) => (e.currentTarget.style.backgroundColor = "transparent")}
                >
                  <span style="flex: 1;">
                    <span style="font-family: monospace;">#{encounter.id}</span> - {encounter.total_damage.toLocaleString()} dmg
                  </span>
                  {#if $selectedEncounter.type === "historical" && $selectedEncounter.encounterId === encounter.id}
                    <span style="color: rgb(34, 197, 94); margin-left: 0.5rem;">✓</span>
                  {/if}
                </button>
              </div>
            {/each}
          {/if}
        </div>
      {/if}
    </div>

    <span {@attach tooltip(() => "Time Elapsed")}>{formatElapsed(headerInfo.elapsedMs)}</span>
    <span><span {@attach tooltip(() => "Total Damage Dealt")}>T.DMG</span> <span {@attach tooltip(() => headerInfo.totalDmg.toLocaleString())}><AbbreviatedNumber num={Number(headerInfo.totalDmg)} /></span></span>
    <span><span {@attach tooltip(() => "Total Damage per Second")}>T.DPS</span> <span {@attach tooltip(() => headerInfo.totalDps.toLocaleString())}><AbbreviatedNumber num={headerInfo.totalDps} /></span></span>
  </span>
  <!-- Right side -->
  <span class="flex gap-1">
    <!-- TODO: add responsive clicks, toaster -->
    <button
      onclick={async () => {
        const prev = SETTINGS.general.state.showOthersName;
        if (SETTINGS.general.state.showOthersName === "Show Others' Name") {
          SETTINGS.general.state.showOthersName = "Show Others' Class";
        }

        // Wait for reactive flush & paint
        await tick();

        // Take screenshot AFTER change is visible
        await takeScreenshot(screenshotDiv);

        // Revert & let UI update
        SETTINGS.general.state.showOthersName = prev;
        await tick();
      }}
      {@attach tooltip(() => "Screenshot to Clipboard")}
    >
      <CameraIcon />
    </button>
    <button
      onclick={async () => {
        commands.resetEncounter();
        window.location.reload(); // TODO: temp fix
      }}
      {@attach tooltip(() => "Reset Encounter")}><TimerResetIcon /></button
    >
    <button
      onclick={() => {
        commands.togglePauseEncounter();
        isEncounterPaused = !isEncounterPaused;
      }}
    >
      {#if isEncounterPaused}
        <PlayIcon {@attach tooltip(() => "Resume Encounter")} />
      {:else}
        <PauseIcon {@attach tooltip(() => "Pause Encounter")} />
      {/if}
    </button>
    <button onclick={() => appWindow.setIgnoreCursorEvents(true)} {@attach tooltip(() => "Clickthrough")}><PointerIcon /></button>
    <button onclick={() => openSettings()} {@attach tooltip(() => "Settings")}><SettingsIcon /></button>
    <button onclick={() => appWindow.hide()} {@attach tooltip(() => "Minimize")}><MinusIcon /></button>
  </span>
</header>
