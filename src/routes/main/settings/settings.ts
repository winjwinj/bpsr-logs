import type { SETTINGS } from '$lib/settings-store';

export type BaseInputs = BaseInput[];

/** Common base for all settings */
export interface BaseInput {
	id: keyof typeof SETTINGS.shortcuts.state;
	label: string;
	description?: string;
}
