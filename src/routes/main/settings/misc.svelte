<script lang="ts">
	import { commands } from '$lib/bindings';
	import * as Tabs from '$lib/components/ui/tabs/index.js';
	import { SETTINGS } from '$lib/settings-store';
	import SettingsButton from './settings-button.svelte';
	import SettingsSwitch from './settings-switch.svelte';
	import { openPath, revealItemInDir, openUrl } from '@tauri-apps/plugin-opener';
	import * as path from '@tauri-apps/api/path';

	const SETTINGS_CATEGORY = 'misc';

	async function extractModules() {
		try {
			const result = await commands.extractModulesFromLocalPlayer();
			if (result.status === 'ok') {
				await openUrl(result.data);
			} else {
				alert(`Failed to extract modules: ${result.error}`);
			}
		} catch (error) {
			alert(`Failed to extract modules: ${error}`);
		}
	}
</script>

<Tabs.Content value={SETTINGS_CATEGORY}>
	<SettingsButton
		onclick={commands.hardReset}
		buttonLabel="Restart"
		label="Restart Packet Capture"
		description="Restart WinDivert packet capture. Use this if data is not updating or if you encounter issues."
	/>
	<SettingsSwitch
		bind:checked={SETTINGS.misc.state.testingMode}
		label="Testing Mode"
		description="Enable UI Testing. Only works with DPS/Heal Player/Skills. Once you turn it off, make sure to reload the window."
	/>
	<!-- https://v2.tauri.app/plugin/file-system/#usage -->
	<!-- https://v2.tauri.app/plugin/file-system/#scopes -->
	<SettingsButton
		onclick={async () => await openPath(await path.appLogDir())}
		buttonLabel="Logs"
		label="Go to Logs Folder"
		description="Go to logs folder that contains all the logs for bpsr-logs. Use this file to report any bugs."
	/>
	<SettingsButton
		onclick={async () =>
			await revealItemInDir(
				await path.join(await path.appDataDir(), 'tauri-plugin-svelte\\general.json')
			)}
		buttonLabel="Settings"
		label="Go to Settings Folder"
		description="Go to settings folder that contains all the setting files for bpsr-logs."
	/>
	<SettingsButton
		onclick={async () =>
			await revealItemInDir(await path.join(await path.appDataDir(), '.window-state.json'))}
		buttonLabel="Window Memory"
		label="Go to Window Memory Folder"
		description="Go to window memory folder that contains the window memory file for bpsr-logs. This file contains the memory of your window positions, etc."
	/>
	<SettingsButton
		onclick={async () =>
			await revealItemInDir(await path.join(await path.resourceDir(), 'bpsr-logs.exe'))}
		buttonLabel="App Install"
		label="Go to App Install location"
		description="Go to folder that contains the bpsr-logs installation."
	/>
	<!-- TEMP: Module extraction button -->
	<SettingsButton
		onclick={extractModules}
		buttonLabel="Module Optimizer"
		label="Module Optimizer (Temporary)"
		description="Extract module data from local player and open Module Optimizer. This is a temporary feature."
	/>
</Tabs.Content>
