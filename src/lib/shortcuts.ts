import { commands } from '$lib/bindings';
import { SETTINGS } from '$lib/settings-store';
import { toggleClickthrough } from '$lib/utils.svelte';
import { emitTo } from '@tauri-apps/api/event';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { register, unregisterAll } from '@tauri-apps/plugin-global-shortcut';

export async function setupShortcuts() {
	await unregisterAll();
	for (const [cmdId, shortcutKey] of Object.entries(SETTINGS.shortcuts.state)) {
		registerShortcut(cmdId, shortcutKey);
	}
}

export async function registerShortcut(cmdId: string, shortcutKey: string) {
	if (shortcutKey) {
		switch (cmdId) {
			case 'toggleLiveMeter':
				await register(shortcutKey, async (event) => {
					if (event.state === 'Pressed') {
						const liveWindow = await WebviewWindow.getByLabel('live');
						const isVisible = await liveWindow?.isVisible();
						if (isVisible) {
							await liveWindow?.hide();
						} else {
							await liveWindow?.show();
						}
					}
				});
				break;

			case 'showDpsTab':
				await register(shortcutKey, async (event) => {
					if (event.state === 'Pressed') {
						await emitTo('live', 'navigate', '/');
					}
				});
				break;

			case 'showHealTab':
				await register(shortcutKey, async (event) => {
					if (event.state === 'Pressed') {
						await emitTo('live', 'navigate', '/heal');
					}
				});
				break;

			case 'toggleClickthrough':
				await register(shortcutKey, async (event) => {
					if (event.state === 'Pressed') {
						await toggleClickthrough();
					}
				});
				break;

			case 'resetEncounter':
				await register(shortcutKey, async (event) => {
					if (event.state === 'Pressed') {
						await commands.resetEncounter();
					}
				});
				break;

			case 'hardReset':
				await register(shortcutKey, async (event) => {
					if (event.state === 'Pressed') {
						commands.hardReset();
					}
				});
				break;

			default:
				console.log('Unknown command');
		}
	}
}
