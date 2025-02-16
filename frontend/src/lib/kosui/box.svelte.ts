/**
 * A generic container that wraps a reactive Svelte state. The value in the
 * container can be updated by components in a Snippet:
 *
 * ```svelte
 * <Tooltip>
 *   {#snippet trigger(ref: Box<HTMLElement>, props)}
 *     <!-- Bind the button's ref to the value in the ref box -->
 *     <Button bind:ref={ref.value} {...props}>Render Delegated</Button>
 *   {/snippet}
 *   {#snippet children()}
 *     Hi, I'm a Tooltip!
 *   {/snippet}
 * </Tooltip>
 * ```
 *
 * In this example, the Tooltip component owns a reactive state that is a
 * reference to the HTMLButtonElement. This is done so that the Tooltip can
 * track the location of the button, and update the position of the tooltip.
 * Normally, you can't update state inside of a Snippet. So, we put the state
 * into a Box, and then we can bind the Button's ref to the value inside the
 * Box.
 *
 * @template T The type of the value stored in the box
 */
export class Box<T> {
  #value: T | undefined = $state();

  get value(): T | undefined {
    return this.#value;
  }

  set value(value: T | undefined) {
    this.#value = value;
  }

  apply(value: T | undefined) {
    this.#value = value;
  }
}
