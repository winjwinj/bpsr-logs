import tippy from 'tippy.js';
import 'tippy.js/dist/tippy.css';
import type { Attachment } from 'svelte/attachments';
import html2canvas from 'html2canvas-pro';
import { writeText, writeImage } from '@tauri-apps/plugin-clipboard-manager';
import { image } from '@tauri-apps/api';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';

// Theme-aware class colors using oklch
// Colors match class themes: healers (green), tanks (brown/yellow), DPS (red/ice/cyan/purple)
export const classColors: Record<string, { dark: string; light: string }> = {
	// Healers - greenish tones
	'Beat Performer': { dark: 'oklch(0.55 0.18 145)', light: 'oklch(0.45 0.18 145)' },
	'Verdant Oracle': { dark: 'oklch(0.60 0.20 130)', light: 'oklch(0.50 0.20 130)' },
	// Tanks - brownish and yellowish
	'Heavy Guardian': { dark: 'oklch(0.50 0.10 50)', light: 'oklch(0.40 0.10 50)' },
	'Shield Knight': { dark: 'oklch(0.65 0.15 85)', light: 'oklch(0.55 0.15 85)' },
	// DPS - red, ice, cyan, purple
	Marksman: { dark: 'oklch(0.55 0.20 25)', light: 'oklch(0.45 0.20 25)' },
	'Frost Mage': { dark: 'oklch(0.70 0.12 220)', light: 'oklch(0.60 0.12 220)' },
	'Wind Knight': { dark: 'oklch(0.60 0.18 200)', light: 'oklch(0.50 0.18 200)' },
	Stormblade: { dark: 'oklch(0.55 0.22 280)', light: 'oklch(0.45 0.22 280)' }
};

export function getClassColor(className: string): string {
	const isLight = document.documentElement.classList.contains('light');
	const color =
		classColors[className]?.[isLight ? 'light' : 'dark'] ??
		(isLight ? 'oklch(0.50 0.15 320)' : 'oklch(0.60 0.15 320)');
	return `oklch(from ${color} l c h / 0.5)`;
}

export function getClassIcon(className: string): string {
	if (
		className === 'Hidden Class' ||
		className === 'Unknown Class' ||
		className === 'Undefined Class' ||
		!className
	) {
		return '/images/blank.png';
	}
	return `/images/classes/${className}.png`;
}

import SkillIconJson from '$lib/data/json/SkillIcon.json';
export const SkillIconMap: Record<string, string> = SkillIconJson;
export function getSkillIcon(skillUid: number): string {
	const key = skillUid.toString();
	if (key in SkillIconMap) {
		return `/images/skills/${SkillIconMap[key]}.webp`;
	} else {
		return '/images/blank.png';
	}
}

// https://svelte.dev/docs/svelte/@attach#Attachment-factories
export function tooltip(getContent: () => string): Attachment {
	return (element: Element) => {
		const tooltip = tippy(element, {
			content: ''
		});
		$effect(() => {
			tooltip.setContent(getContent());
		});
		return tooltip.destroy;
	};
}

export async function copyToClipboard(
	error: MouseEvent & { currentTarget: EventTarget & HTMLElement },
	content: string
) {
	// TODO: add a way to simulate a "click" animation
	error.stopPropagation();
	await writeText(content);
}

export async function takeScreenshot(target?: HTMLElement): Promise<void> {
	if (!target) return;
	// Give the browser a paint frame (helps if caller just changed DOM)
	await new Promise(requestAnimationFrame);

	const canvas = await html2canvas(target, { backgroundColor: '#27272A' });

	const blob: Blob | null = await new Promise((resolve) => canvas.toBlob(resolve));
	if (!blob) return;

	try {
		await writeImage(await image.Image.fromBytes(await blob.arrayBuffer()));
	} catch (error) {
		console.error('Failed to take a screenshot', error);
	}
}

let isClickthrough = false;

export async function setClickthrough(bool: boolean) {
	const liveWindow = await WebviewWindow.getByLabel('live');
	await liveWindow?.setIgnoreCursorEvents(bool);
	isClickthrough = bool;
}

export async function toggleClickthrough() {
	const liveWindow = await WebviewWindow.getByLabel('live');
	await liveWindow?.setIgnoreCursorEvents(!isClickthrough);
	isClickthrough = !isClickthrough;
}
