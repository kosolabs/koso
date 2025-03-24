<script module lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";
  import { type Variants } from "../base";
  import { Input } from "../input";
  import type { ClassName, ElementRef } from "../utils";
  import { getAutocompleteContext } from "./autocomplete-context.svelte";

  export type AutocompleteInputProps = {} & ElementRef &
    ClassName &
    Variants &
    Omit<HTMLInputAttributes, "autocomplete">;
</script>

<script lang="ts">
  import { mergeComponentProps } from "../merge-props";

  let {
    el = $bindable(),
    value = $bindable(""),
    ...restProps
  }: AutocompleteInputProps = $props();

  const ctx = getAutocompleteContext();
  ctx.bindInput(
    () => value,
    (newval) => (value = newval),
  );
  ctx.bindAnchorEl(
    () => el,
    (newval) => (el = newval),
  );

  function handleKeyDown(event: KeyboardEvent) {
    ctx.handleKeyDown(event);
  }
</script>

<Input
  bind:el={ctx.anchorEl}
  bind:value={ctx.input}
  autocomplete="off"
  {...mergeComponentProps(Input, { onkeydown: handleKeyDown }, restProps)}
/>
