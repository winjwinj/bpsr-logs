import type { Component, ComponentProps, Snippet } from 'svelte';

/**
 * A helper class to make it easy to identify Svelte components in
 * `columnDef.cell` and `columnDef.header` properties.
 *
 * > NOTE: This class should only be used internally by the adapter. If you're
 * reading this and you don't know what this is for, you probably don't need it.
 *
 * @example
 * ```svelte
 * {@const result = content(context as any)}
 * {#if result instanceof RenderComponentConfig}
 *   {@const { component: Component, props } = result}
 *   <Component {...props} />
 * {/if}
 * ```
 * */
export class RenderComponentConfig<TComponent extends Component> {
	constructor(
		public component: TComponent,
		public props: ComponentProps<TComponent> | Record<string, never> = {}
	) {}
}

/**
 * A helper class to make it easy to identify Svelte Snippets in `columnDef.cell` and `columnDef.header` properties.
 *
 * > NOTE: This class should only be used internally by the adapter. If you're
 * reading this and you don't know what this is for, you probably don't need it.
 *
 * @example
 * ```svelte
 * {@const result = content(context as any)}
 * {#if result instanceof RenderSnippetConfig}
 *   {@const { snippet, params } = result}
 *   {@render snippet(params)}
 * {/if}
 * ```
 * */
export class RenderSnippetConfig<TProps> {
	constructor(
		public snippet: Snippet<[TProps]>,
		public params: TProps
	) {}
}

/**
 * A helper function to help create cells from Svelte components through ColumnDef's `cell` and `header` properties.
 *
 * @param component A Svelte component
 * @param props The props to pass to `component`
 * @returns A `RenderComponentConfig` object that helps svelte-table know how to render the header/cell component.
 * @example
 * ```ts
 * // +page.svelte
 * const defaultColumns = [
 *   columnHelper.accessor('name', {
 *     header: header => renderComponent(SortHeader, { label: 'Name', header }),
 *   }),
 *   columnHelper.accessor('state', {
 *     header: header => renderComponent(SortHeader, { label: 'State', header }),
 *   }),
 * ]
 * ```
 * @see {@link https://tanstack.com/table/latest/docs/guide/column-defs}
 */
export const renderComponent = <
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	TComponent extends Component<any>,
	TProps extends ComponentProps<TComponent>
>(
	component: TComponent,
	props: TProps
) => new RenderComponentConfig(component, props);
