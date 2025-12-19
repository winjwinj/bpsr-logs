<script lang="ts">
	import { copyToClipboard, getSkillIcon } from '$lib/utils.svelte';

	let {
		skillUid = -1,
		skillName = 'Unknown Skill'
	}: {
		skillUid: number;
		skillName: string;
	} = $props();
</script>

<div class="ml-2 flex">
	<img
		class="size-5 object-contain"
		src={getSkillIcon(skillUid)}
		alt={`${skillName} icon`}
		onerror={(e) => {
			const img = e.currentTarget as HTMLImageElement;
			if (img.src !== '/images/blank.png') {
				img.src = '/images/blank.png';
			}
		}}
	/>
	<button
		type="button"
		class="ml-1 cursor-pointer truncate border-none bg-transparent p-0 text-left"
		onclick={(error) => copyToClipboard(error, skillName)}
	>
		{skillName} ({skillUid})
	</button>
</div>
