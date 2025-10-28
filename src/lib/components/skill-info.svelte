<script lang="ts">
  import { copyToClipboard, getSkillIcon } from "$lib/utils.svelte";

  let {
    skillUid = -1,
    skillName = "Unknown Skill",
  }: {
    skillUid: number;
    skillName: string;
  } = $props();
</script>

<div class="ml-2 flex">
  <img class="size-5 object-contain" src={getSkillIcon(skillUid)} />
  <span class="ml-1 cursor-pointer truncate" onclick={(error) => copyToClipboard(error, skillName)}>
    {skillName} ({skillUid})
  </span>
</div>
