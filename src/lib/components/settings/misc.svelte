<script lang="ts">
	import * as Tabs from '$lib/components/ui/tabs/index.js';
	import * as Card from '$lib/components/ui/card/index.js';
	import { SETTINGS } from '$lib/settings-store';
	import SettingsButton from './settings-button.svelte';
	import SettingsSwitch from './settings-switch.svelte';
	import { commands } from '$lib/bindings';
	import { openPath, revealItemInDir } from '@tauri-apps/plugin-opener';
	import * as path from '@tauri-apps/api/path';

	const SETTINGS_CATEGORY = 'misc';
</script>

<Tabs.Content value={SETTINGS_CATEGORY} class="space-y-4">
	<Card.Root>
		<Card.Header>
			<Card.Title>Debug & Testing</Card.Title>
		</Card.Header>
		<Card.Content class="space-y-2">
			<SettingsButton
				onclick={() => commands.hardReset()}
				buttonLabel="Restart"
				label="Restart Packet Capture"
				description="Restart WinDivert packet capture. Use this if data is not updating or if you encounter issues."
			/>
			<SettingsSwitch
				bind:checked={SETTINGS.misc.state.testingMode}
				label="Testing Mode"
				description="Enable UI Testing. Only works with DPS/Heal Player/Skills. Once you turn it off, make sure to reload the window."
			/>
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title>File Locations</Card.Title>
		</Card.Header>
		<Card.Content class="space-y-2">
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
		</Card.Content>
	</Card.Root>
</Tabs.Content>
