import { createColumnHelper } from "@tanstack/table-core";

import { renderComponent } from "$lib/svelte-table";

import type { PlayerRow, SkillRow } from "$lib/bindings";
import PlayerInfo from "$lib/components/player-info.svelte";
import AbbreviatedNumber from "$lib/components/abbreviated-number.svelte";
import PercentFormat from "$lib/components/percent-format.svelte";
import SkillInfo from "$lib/components/skill-info.svelte";

declare module '@tanstack/table-core' {
  interface TableMeta<TData> {
    localPlayerUid: number;
  }

  interface ColumnMeta<TData, TValue> {
    class: string;         // CSS class (for Tailwind) for column
    label: string;         // Label text for SETTINGS
    description?: string;  // Descriptive text for SETTINGS
    // TODO: move the label and description to the settings
  }
}

// TODO: add tooltips - maybe change all these components to snippets instead?
// `meta` is being used for css class styling (per column)
const dpsPlayersColumnHelper = createColumnHelper<PlayerRow>();
export const dpsPlayersColumnDefs = [
  dpsPlayersColumnHelper.display({
    id: 'playerInfo',
    cell: ({ row, table }) =>
      renderComponent(PlayerInfo, {
        className: row.original.className,
        classSpecName: row.original.classSpecName,
        abilityScore: row.original.abilityScore,
        name: row.original.name,
        uid: row.original.uid,
        localPlayerUid: table.options.meta?.localPlayerUid ?? -1,
      }),
    meta: {
      class: "w-full",
      label: "Player Info",
      description: "Show ability score and name with UID tooltip (click to copy #UID)"
    }
  }),

  dpsPlayersColumnHelper.accessor('totalValue', {
    header: 'DMG',
    cell: ({ cell }) => renderComponent(AbbreviatedNumber, { num: cell.getValue() }),
    meta: {
      class: "w-12",
      label: "DMG",
      description: "Show player's total damage dealt"
    }
  }),

  dpsPlayersColumnHelper.accessor('valuePerSec', {
    header: 'DPS',
    cell: ({ cell }) => renderComponent(AbbreviatedNumber, { num: cell.getValue() }),
    meta: {
      class: "w-12",
      label: "DPS",
      description: "Show player's damage per second"
    }
  }),

  dpsPlayersColumnHelper.accessor('valuePct', {
    header: () => renderComponent(PercentFormat, { val: "D" }),
    cell: ({ cell }) =>
      renderComponent(PercentFormat, { val: cell.getValue(), fractionDigits: 0 }),
    meta: {
      class: "w-12",
      label: "D%",
      description: "Show player's damage % contribution"
    }
  }),

  dpsPlayersColumnHelper.accessor('critRate', {
    header: () => renderComponent(PercentFormat, { val: "CR" }),
    cell: ({ cell }) => renderComponent(PercentFormat, { val: cell.getValue() }),
    meta: {
      class: "w-12",
      label: "CR",
      description: "Show player's critical rate"
    }
  }),

  dpsPlayersColumnHelper.accessor('critValueRate', {
    header: () => renderComponent(PercentFormat, { val: "CDMG" }),
    cell: ({ cell }) => renderComponent(PercentFormat, { val: cell.getValue() }),
    meta: {
      class: "w-13",
      label: "CDMG",
      description: "Show player's % damage that crit"
    }
  }),

  dpsPlayersColumnHelper.accessor('luckyRate', {
    header: () => renderComponent(PercentFormat, { val: "LR" }),
    cell: ({ cell }) => renderComponent(PercentFormat, { val: cell.getValue() }),
    meta: {
      class: "w-12",
      label: "LR%",
      description: "Show player's lucky rate"
    }
  }),

  dpsPlayersColumnHelper.accessor('luckyValueRate', {
    header: () => renderComponent(PercentFormat, { val: "LDMG" }),
    cell: ({ cell }) => renderComponent(PercentFormat, { val: cell.getValue() }),
    meta: {
      class: "w-13",
      label: "LDMG%",
      description: "Show player's % damage that was lucky"
    }
  }),

  dpsPlayersColumnHelper.accessor('hits', {
    header: 'Hits',
    meta: {
      class: "w-13",
      label: "Hits",
      description: "Show player's total number of hits"
    }
  }),

  dpsPlayersColumnHelper.accessor('hitsPerMinute', {
    header: 'HPM',
    cell: ({ cell }) => cell.getValue().toFixed(1),
    meta: {
      class: "w-12",
      label: "HPM",
      description: "Show player's number of hits per minute"
    }
  }),
];

const dpsSkillsColumnHelper = createColumnHelper<SkillRow>();
export const dpsSkillsColumnDefs = [
  dpsSkillsColumnHelper.display({
    id: 'skillName',
    cell: ({ row }) =>
      renderComponent(SkillInfo, {
        skillUid: row.original.uid,
        skillName: row.original.name
      }),
    meta: {
      class: "w-full",
      label: "Skill Name",
      description: "Show skill name"
    }
  }),

  dpsSkillsColumnHelper.accessor('totalValue', {
    header: 'DMG',
    cell: ({ cell }) => renderComponent(AbbreviatedNumber, { num: cell.getValue() }),
    meta: {
      class: "w-12",
      label: "DMG",
      description: "Show skill's total damage dealt"
    }
  }),

  dpsSkillsColumnHelper.accessor('valuePerSec', {
    header: 'DPS',
    cell: ({ cell }) => renderComponent(AbbreviatedNumber, { num: cell.getValue() }),
    meta: {
      class: "w-12",
      label: "DPS",
      description: "Show skill's damage per second"
    }
  }),

  dpsSkillsColumnHelper.accessor('valuePct', {
    header: () => renderComponent(PercentFormat, { val: "D" }),
    cell: ({ cell }) =>
      renderComponent(PercentFormat, { val: cell.getValue(), fractionDigits: 0 }),
    meta: {
      class: "w-12",
      label: "D%",
      description: "Show skill's damage % contribution"
    }
  }),

  dpsSkillsColumnHelper.accessor('critRate', {
    header: () => renderComponent(PercentFormat, { val: "CR" }),
    cell: ({ cell }) => renderComponent(PercentFormat, { val: cell.getValue() }),
    meta: {
      class: "w-12",
      label: "CR",
      description: "Show skill's critical rate"
    }
  }),

  dpsSkillsColumnHelper.accessor('critValueRate', {
    header: () => renderComponent(PercentFormat, { val: "CDMG" }),
    cell: ({ cell }) => renderComponent(PercentFormat, { val: cell.getValue() }),
    meta: {
      class: "w-13",
      label: "CDMG",
      description: "Show skill's % damage that crit"
    }
  }),

  dpsSkillsColumnHelper.accessor('luckyRate', {
    header: () => renderComponent(PercentFormat, { val: "LR" }),
    cell: ({ cell }) => renderComponent(PercentFormat, { val: cell.getValue() }),
    meta: {
      class: "w-12",
      label: "LR%",
      description: "Show skill's lucky rate"
    }
  }),

  dpsSkillsColumnHelper.accessor('luckyValueRate', {
    header: () => renderComponent(PercentFormat, { val: "LDMG" }),
    cell: ({ cell }) => renderComponent(PercentFormat, { val: cell.getValue() }),
    meta: {
      class: "w-13",
      label: "LDMG%",
      description: "Show skill's % damage that was lucky"
    }
  }),

  dpsSkillsColumnHelper.accessor('hits', {
    header: 'Hits',
    meta: {
      class: "w-13",
      label: "Hits",
      description: "Show skill's total number of hits"
    }
  }),

  dpsSkillsColumnHelper.accessor('hitsPerMinute', {
    header: 'HPM',
    cell: ({ cell }) => cell.getValue().toFixed(1),
    meta: {
      class: "w-12",
      label: "HPM",
      description: "Show skill's number of hits per minute"
    }
  }),
];

const healPlayersColumnHelper = createColumnHelper<PlayerRow>();
export const healPlayersColumnDefs = [
  healPlayersColumnHelper.display({
    id: 'playerInfo',
    cell: ({ row, table }) =>
      renderComponent(PlayerInfo, {
        className: row.original.className,
        classSpecName: row.original.classSpecName,
        abilityScore: row.original.abilityScore,
        name: row.original.name,
        uid: row.original.uid,
        localPlayerUid: table.options.meta?.localPlayerUid ?? -1,
      }),
    meta: {
      class: "w-full",
      label: "Player Info",
      description: "Show ability score and name with UID tooltip (click to copy #UID)"
    }
  }),

  healPlayersColumnHelper.accessor('totalValue', {
    header: 'Heal',
    cell: ({ cell }) => renderComponent(AbbreviatedNumber, { num: cell.getValue() }),
    meta: {
      class: "w-12",
      label: "Heal",
      description: "Show player's total heal given"
    }
  }),

  healPlayersColumnHelper.accessor('valuePerSec', {
    header: 'HPS',
    cell: ({ cell }) => renderComponent(AbbreviatedNumber, { num: cell.getValue() }),
    meta: {
      class: "w-12",
      label: "HPS",
      description: "Show player's heal per second"
    }
  }),

  healPlayersColumnHelper.accessor('valuePct', {
    header: () => renderComponent(PercentFormat, { val: "H" }),
    cell: ({ cell }) =>
      renderComponent(PercentFormat, { val: cell.getValue(), fractionDigits: 0 }),
    meta: {
      class: "w-12",
      label: "H%",
      description: "Show player's heal % contribution"
    }
  }),

  healPlayersColumnHelper.accessor('critRate', {
    header: () => renderComponent(PercentFormat, { val: "CR" }),
    cell: ({ cell }) => renderComponent(PercentFormat, { val: cell.getValue() }),
    meta: {
      class: "w-12",
      label: "CR",
      description: "Show player's heal critical rate"
    }
  }),

  healPlayersColumnHelper.accessor('critValueRate', {
    header: () => renderComponent(PercentFormat, { val: "CDMG" }),
    cell: ({ cell }) => renderComponent(PercentFormat, { val: cell.getValue() }),
    meta: {
      class: "w-13",
      label: "CDMG",
      description: "Show player's % heal that crit"
    }
  }),

  healPlayersColumnHelper.accessor('luckyRate', {
    header: () => renderComponent(PercentFormat, { val: "LR" }),
    cell: ({ cell }) => renderComponent(PercentFormat, { val: cell.getValue() }),
    meta: {
      class: "w-12",
      label: "LR%",
      description: "Show player's heal lucky rate"
    }
  }),

  healPlayersColumnHelper.accessor('luckyValueRate', {
    header: () => renderComponent(PercentFormat, { val: "LDMG" }),
    cell: ({ cell }) => renderComponent(PercentFormat, { val: cell.getValue() }),
    meta: {
      class: "w-13",
      label: "LDMG%",
      description: "Show player's % heal that was lucky"
    }
  }),

  healPlayersColumnHelper.accessor('hits', {
    header: 'Hits',
    meta: {
      class: "w-13",
      label: "Hits",
      description: "Show player's total number of hits"
    }
  }),

  healPlayersColumnHelper.accessor('hitsPerMinute', {
    header: 'HPM',
    cell: ({ cell }) => cell.getValue().toFixed(1),
    meta: {
      class: "w-12",
      label: "HPM",
      description: "Show player's number of hits per minute"
    }
  }),
];

const healSkillsColumnHelper = createColumnHelper<SkillRow>();
export const healSkillsColumnDefs = [
  healSkillsColumnHelper.display({
    id: 'skillName',
    cell: ({ row }) =>
      renderComponent(SkillInfo, {
        skillUid: row.original.uid,
        skillName: row.original.name
      }),
    meta: {
      class: "w-full",
      label: "Skill Name",
      description: "Show skill name"
    }
  }),

  healSkillsColumnHelper.accessor('totalValue', {
    header: 'Heal',
    cell: ({ cell }) => renderComponent(AbbreviatedNumber, { num: cell.getValue() }),
    meta: {
      class: "w-12",
      label: "Heal",
      description: "Show skill's total heal given"
    }
  }),

  healSkillsColumnHelper.accessor('valuePerSec', {
    header: 'HPS',
    cell: ({ cell }) => renderComponent(AbbreviatedNumber, { num: cell.getValue() }),
    meta: {
      class: "w-12",
      label: "HPS",
      description: "Show skill's heal per second"
    }
  }),

  healSkillsColumnHelper.accessor('valuePct', {
    header: () => renderComponent(PercentFormat, { val: "H" }),
    cell: ({ cell }) =>
      renderComponent(PercentFormat, { val: cell.getValue(), fractionDigits: 0 }),
    meta: {
      class: "w-12",
      label: "H%",
      description: "Show skill's heal % contribution"
    }
  }),

  healSkillsColumnHelper.accessor('critRate', {
    header: () => renderComponent(PercentFormat, { val: "CR" }),
    cell: ({ cell }) => renderComponent(PercentFormat, { val: cell.getValue() }),
    meta: {
      class: "w-12",
      label: "CR",
      description: "Show skill's critical rate"
    }
  }),

  healSkillsColumnHelper.accessor('critValueRate', {
    header: () => renderComponent(PercentFormat, { val: "CDMG" }),
    cell: ({ cell }) => renderComponent(PercentFormat, { val: cell.getValue() }),
    meta: {
      class: "w-13",
      label: "CDMG",
      description: "Show skill's % heal that crit"
    }
  }),

  healSkillsColumnHelper.accessor('luckyRate', {
    header: () => renderComponent(PercentFormat, { val: "LR" }),
    cell: ({ cell }) => renderComponent(PercentFormat, { val: cell.getValue() }),
    meta: {
      class: "w-12",
      label: "LR%",
      description: "Show skill's heal lucky rate"
    }
  }),

  healSkillsColumnHelper.accessor('luckyValueRate', {
    header: () => renderComponent(PercentFormat, { val: "LDMG" }),
    cell: ({ cell }) => renderComponent(PercentFormat, { val: cell.getValue() }),
    meta: {
      class: "w-13",
      label: "LDMG%",
      description: "Show skill's % heal that was lucky"
    }
  }),

  healSkillsColumnHelper.accessor('hits', {
    header: 'Hits',
    meta: {
      class: "w-13",
      label: "Hits",
      description: "Show skill's total number of hits"
    }
  }),

  healSkillsColumnHelper.accessor('hitsPerMinute', {
    header: 'HPM',
    cell: ({ cell }) => cell.getValue().toFixed(1),
    meta: {
      class: "w-12",
      label: "HPM",
      description: "Show skill's number of hits per minute"
    }
  }),
];
