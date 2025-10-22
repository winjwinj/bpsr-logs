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

  import { onMount } from "svelte";
  import { commands, type HeaderInfo } from "$lib/bindings";
  import { pushEncounter } from "$lib/encounter-history-store";
  import { encounters, selectEncounter, clearHistory, selectedEncounterId } from "$lib/encounter-history-store";
  import { takeScreenshot, tooltip } from "$lib/utils.svelte";
  import AbbreviatedNumber from "$lib/components/abbreviated-number.svelte";
  import { emitTo } from "@tauri-apps/api/event";
  import { SETTINGS } from "$lib/settings-store";

  onMount(() => {
    fetchData();
    const interval = setInterval(fetchData, 200);
    return () => clearInterval(interval);
  });

  let hasReset = false;

  async function fetchData() {
    try {
      const result = await commands.getHeaderInfo();
      if (result.status !== "ok") {
        console.warn("Failed to get header: ", result.error);
        return;
      } else {
        headerInfo = result.data;
        // Only check for elapsed reset after we have a valid header timestamp
  // only trigger reset if we have a valid last-packet timestamp
  const lastMs = headerInfo.timeLastCombatPacketMs || 0;
        if (SETTINGS.general.state.resetElapsed && lastMs > 0 && !hasReset && Date.now() - lastMs > SETTINGS.general.state.resetElapsed * 1000) {
          hasReset = true;
          console.log(`Resetting as ${SETTINGS.general.state.resetElapsed}s has passed.`);
          // Snapshot before automatic reset
          try {
              const snapshot = await buildSnapshot(headerInfo);
              // push snapshot into history but don't change selection during automatic reset
              pushEncounter(snapshot);
          } catch (e) {
            console.warn('Failed to snapshot encounter before automatic hardReset', e);
          }
          commands.hardReset(); // TODO: this is temporary, switch to resetEncounter once bug is fixed.
        }
        // console.log("header: ", +Date.now(), $state.snapshot(headerInfo));
        if (hasReset) {
          hasReset = false;
          window.location.reload();
          console.log("Fresh packet");
        }
      }
    } catch (e) {
      console.error("Error fetching data: ", e);
    }
  }

  // Build a snapshot object including header, dps/heal windows and per-player skill windows when possible
  async function buildSnapshot(existingHeader?: any) {
    const snapshot: any = {};
    try {
      const headerRes = existingHeader ? { status: 'ok', data: existingHeader } : await commands.getHeaderInfo();
      const dpsRes = await commands.getDpsPlayerWindow();
      const healRes = await commands.getHealPlayerWindow();
      if (headerRes.status === 'ok') snapshot.headerInfo = headerRes.data;
      if (dpsRes.status === 'ok') snapshot.dpsPlayersWindow = dpsRes.data;
      if (healRes.status === 'ok') snapshot.healPlayersWindow = healRes.data;

      // Attempt to fetch per-player skill windows for players in dpsPlayersWindow
      if (dpsRes.status === 'ok' && Array.isArray(dpsRes.data?.playerRows)) {
        const uids = dpsRes.data.playerRows.map((r: any) => String(r.uid)).filter(Boolean);
        const skillPromises = uids.map((uid: string) =>
          // commands.getDpsSkillWindow accepts playerUid string
          commands.getDpsSkillWindow(uid).then((res: any) => ({ uid, res })).catch((e: any) => ({ uid, res: { status: 'error', error: e } }))
        );
        const skillResults = await Promise.all(skillPromises);
        const map: Record<string, any> = {};
        for (const item of skillResults) {
          if (item.res && item.res.status === 'ok') map[item.uid] = item.res.data;
        }
        if (Object.keys(map).length > 0) snapshot.dpsSkillWindows = map;
      }
      // Also attempt to fetch per-player skill windows for healPlayersWindow
      if (healRes.status === 'ok' && Array.isArray(healRes.data?.playerRows)) {
        const uids = healRes.data.playerRows.map((r: any) => String(r.uid)).filter(Boolean);
        const healSkillPromises = uids.map((uid: string) =>
          // commands.getHealSkillWindow accepts playerUid string
          commands.getHealSkillWindow(uid).then((res: any) => ({ uid, res })).catch((e: any) => ({ uid, res: { status: 'error', error: e } }))
        );
        const healSkillResults = await Promise.all(healSkillPromises);
        const healMap: Record<string, any> = {};
        for (const item of healSkillResults) {
          if (item.res && item.res.status === 'ok') healMap[item.uid] = item.res.data;
        }
        if (Object.keys(healMap).length > 0) snapshot.healSkillWindows = healMap;
      }
    } catch (e) {
      console.warn('buildSnapshot error', e);
    }
    return snapshot;
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
    timeLastCombatPacketMs: 0,
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
      await emitTo("main", "navigate", "/main/settings");
    }
  }

  // local binding for the select so its UI updates when the store changes
  let selectedId = $state($selectedEncounterId);
  $effect(() => {
    // keep local select value in sync with the store
    selectedId = $selectedEncounterId;
  });
</script>

<!-- justify-between to create left/right sides -->
<header data-tauri-drag-region class="sticky top-0 flex h-7 w-full items-center justify-between bg-neutral-900/80 px-1">
  <!-- Left side -->
  <span>
    <button
      onclick={async () => {
        try {
          const snapshot = await buildSnapshot();
          const newId = pushEncounter(snapshot);
          if (newId && SETTINGS.general.state.keepCurrentAfterEnd && $selectedEncounterId === 'current') {
            selectEncounter(newId);
          }
        } catch (e) {
          console.warn('Failed to snapshot encounter before hardReset', e);
        }
        commands.hardReset();
        window.location.reload();
      }}
      {@attach tooltip(() => "Temp Fix: Hard Reset")}><RefreshCwIcon /></button
    >
    <span {@attach tooltip(() => "Time Elapsed")}>{formatElapsed(headerInfo.elapsedMs)}</span>
    <span><span {@attach tooltip(() => "Total Damage Dealt")}>T.DMG</span> <span {@attach tooltip(() => headerInfo.totalDmg.toLocaleString())}><AbbreviatedNumber num={Number(headerInfo.totalDmg)} /></span></span>
    <span><span {@attach tooltip(() => "Total Damage per Second")}>T.DPS</span> <span {@attach tooltip(() => headerInfo.totalDps.toLocaleString())}><AbbreviatedNumber num={headerInfo.totalDps} /></span></span>
  </span>
  <!-- Right side -->
  <span class="flex gap-1">
    <!-- Encounter selector -->
    <div class="flex items-center gap-1">
      <select class="text-xs" style="color: white;" bind:value={selectedId} onchange={() => {
        const v = selectedId;
        if (v === '__clear__') {
          clearHistory();
          selectEncounter('current');
        } else {
          selectEncounter(v);
        }
      }}>
        <option value="current" style="color: black;">Current Encounter</option>
        {#if $encounters.length === 0}
          <option disabled style="color: black;">No history</option>
        {/if}
        {#each $encounters as enc}
          <option value={enc.id} style="color: black;">{new Date(enc.timestamp).toLocaleTimeString()}</option>
        {/each}
        <option value="__clear__" style="color: black;">Clear history</option>
      </select>
    </div>
    <!-- TODO: add responsive clicks, toaster -->
    <button
      onclick={async () => takeScreenshot(screenshotDiv)}
      {@attach tooltip(() => "Screenshot to Clipboard")}
    >
      <CameraIcon />
    </button>
    <button
      onclick={async () => {
        // Snapshot current encounter into history before resetting
        try {
          const snapshot = await buildSnapshot();
          const newId = pushEncounter(snapshot);
          // If setting is enabled and the user was viewing current, switch to the new snapshot
          if (newId && SETTINGS.general.state.keepCurrentAfterEnd && $selectedEncounterId === 'current') {
            selectEncounter(newId);
          }
        } catch (e) {
          console.warn('Failed to snapshot encounter before reset', e);
        }
        // perform reset
        await commands.resetEncounter();
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
