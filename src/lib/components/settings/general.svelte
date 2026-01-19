<script lang="ts">
	import * as Tabs from '$lib/components/ui/tabs/index.js';
	import * as Card from '$lib/components/ui/card/index.js';
	import { Slider } from '$lib/components/ui/slider/index.js';
	import { SETTINGS } from '$lib/settings-store';
	import SettingsSelect from './settings-select.svelte';
	import SettingsSlider from './settings-slider.svelte';
	import SettingsSwitch from './settings-switch.svelte';
	import SettingsSwitchDialog from './settings-switch-dialog.svelte';
	import SettingsButton from './settings-button.svelte';
	import { commands } from '$lib/bindings';
	import {
		enable as enableAutostart,
		disable as disableAutostart
	} from '@tauri-apps/plugin-autostart';
	import { openUrl } from '@tauri-apps/plugin-opener';

	const SETTINGS_CATEGORY = 'general';
	// eslint-disable-next-line svelte/prefer-writable-derived
	let transparencyOpacity = $state(SETTINGS.accessibility.state.transparencyOpacity);

	$effect(() => {
		transparencyOpacity = SETTINGS.accessibility.state.transparencyOpacity;
	});

	let sliderId = 'transparency-opacity-slider';

	let previousValue = $state(SETTINGS.integration.state.bptimer);

	$effect(() => {
		const currentValue = SETTINGS.integration.state.bptimer;
		if (currentValue !== previousValue) {
			previousValue = currentValue;
			commands.setBptimerEnabled(currentValue).catch((err: unknown) => {
				console.error('Failed to update bptimer enabled state:', err);
			});
		}
	});

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

<Tabs.Content value={SETTINGS_CATEGORY} class="space-y-4">
	<Card.Root>
		<Card.Header>
			<Card.Title>Appearance</Card.Title>
		</Card.Header>
		<Card.Content class="space-y-4">
			<div class="space-y-2">
				<div class="flex items-center justify-between">
					<label for={sliderId} class="text-sm font-medium">Background Opacity</label>
					<span class="text-sm text-muted-foreground">{transparencyOpacity}%</span>
				</div>
				<Slider
					id={sliderId}
					type="single"
					bind:value={transparencyOpacity}
					min={0}
					max={100}
					step={5}
					onValueChange={(value: number) => {
						transparencyOpacity = value;
						SETTINGS.accessibility.state.transparencyOpacity = value;
					}}
					class="w-full"
				/>
				<p class="text-xs text-muted-foreground">
					0% is fully transparent, 100% is fully opaque. Default is 60%.
				</p>
			</div>
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title>Integration</Card.Title>
		</Card.Header>
		<Card.Content class="space-y-2">
			<SettingsSwitchDialog
				bind:checked={SETTINGS.integration.state.bptimer}
				label="BP Timer"
				description="World Boss and Magical Creature HP data for bptimer.com"
			/>
			<SettingsButton
				onclick={extractModules}
				buttonLabel="Module Optimizer"
				label="Module Optimizer (Temporary)"
				description="Extract module data from local player and open Module Optimizer. This is a temporary feature."
			/>
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title>Display Options</Card.Title>
		</Card.Header>
		<Card.Content class="space-y-2">
			<SettingsSelect
				bind:selected={SETTINGS.general.state.showYourName}
				values={['Show Your Name', 'Show Your Class', 'Hide Your Name']}
				label="Show Your Name"
				description="Show Your Class = replace your name with your class."
			/>
			<SettingsSelect
				bind:selected={SETTINGS.general.state.showOthersName}
				values={["Show Others' Name", "Show Others' Class", "Hide Others' Name"]}
				label="Show Others' Name"
				description="Show Others' Class = replace others' name with their class."
			/>
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title>Ability Score</Card.Title>
		</Card.Header>
		<Card.Content class="space-y-2">
			<SettingsSwitch
				bind:checked={SETTINGS.general.state.showYourAbilityScore}
				label="Your Ability Score"
				description="Show your ability score."
			/>
			<SettingsSwitch
				bind:checked={SETTINGS.general.state.showOthersAbilityScore}
				label="Others' Ability Score"
				description="Show others' ability score."
			/>
			<SettingsSwitch
				bind:checked={SETTINGS.general.state.shortenAbilityScore}
				label="Shorten Ability Score"
				description="Shortens the Ability Score."
			/>
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title>Combat Settings</Card.Title>
		</Card.Header>
		<Card.Content class="space-y-2">
			<SettingsSwitch
				bind:checked={SETTINGS.general.state.bossOnly}
				label="Boss Only Damage"
				description="Only track damage dealt to bosses."
			/>
			<SettingsSlider
				bind:value={SETTINGS.general.state.resetElapsed}
				label="Reset after Elapsed Time"
				description="Amount of time to wait before the meter automatically resets the encounter. 0s = Never Resets."
			></SettingsSlider>
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title>System</Card.Title>
		</Card.Header>
		<Card.Content class="space-y-2">
			<SettingsSwitch
				bind:checked={SETTINGS.general.state.autostart}
				label="Autostart"
				description="Automatically launch application at system startup."
				onCheckedChange={async (checked) =>
					checked ? await enableAutostart() : await disableAutostart()}
			/>
		</Card.Content>
	</Card.Root>
</Tabs.Content>
