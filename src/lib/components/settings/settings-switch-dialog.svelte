<script lang="ts">
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import * as Card from '$lib/components/ui/card/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import SettingsSwitch, { type SettingsSwitchProps } from './settings-switch.svelte';

	let { checked = $bindable(false), ...restProps }: {} & SettingsSwitchProps = $props();

	let confirmOpen: boolean = $state(false);
	let attemptedValue: boolean | null = $state(null);

	function handleSwitchChange(newState: boolean) {
		// if the user is trying to turn it off, show confirmation
		if (newState === false) {
			// keep the switch visually ON until the user confirms
			checked = true;
			attemptedValue = false;
			confirmOpen = true;
		} else {
			// allow turning on without confirmation
			checked = true;
			attemptedValue = null;
		}
	}

	function confirmDisable() {
		if (attemptedValue === false) {
			checked = false;
		}
		attemptedValue = null;
		confirmOpen = false;
	}

	function cancelDisable() {
		// simply close the dialog and keep the switch on
		attemptedValue = null;
		confirmOpen = false;
		checked = true;
	}
</script>

<label class="flex flex-row items-center">
	<SettingsSwitch bind:checked onCheckedChange={handleSwitchChange} {...restProps} />
	<!-- confirmation dialog shown when turning off -->
	<Dialog.Root bind:open={confirmOpen}>
		<Dialog.Content showCloseButton={false} class="sm:max-w-md">
			<Dialog.Header>
				<Dialog.Title>Disable integration?</Dialog.Title>
				<Dialog.Description
					>We rely on this data to help other players. Are you sure you want to continue?</Dialog.Description
				>
			</Dialog.Header>
			<Card.Root class="my-4 border-destructive/50 bg-destructive/5">
				<Card.Content class="py-3">
					<p class="text-sm text-muted-foreground">
						Disabling this integration will stop sharing data with the community. You can re-enable
						it at any time.
					</p>
				</Card.Content>
			</Card.Root>
			<Dialog.Footer class="gap-2">
				<Button variant="outline" onclick={() => cancelDisable()}>Keep Enabled</Button>
				<Button variant="destructive" onclick={() => confirmDisable()}>Disable</Button>
			</Dialog.Footer>
		</Dialog.Content>
	</Dialog.Root>
</label>
