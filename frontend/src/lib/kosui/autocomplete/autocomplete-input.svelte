<script module lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";
  import type { Autocomplete } from ".";
  import { type Variants } from "../base";
  import { Input } from "../input";
  import type { ClassName, ElementRef } from "../utils";

  export type AutocompleteInputProps = {
    autocomplete: Autocomplete;
  } & ElementRef &
    ClassName &
    Variants &
    Omit<HTMLInputAttributes, "autocomplete">;
</script>

<script lang="ts">
  let {
    value = $bindable(""),
    autocomplete,
    ...restProps
  }: AutocompleteInputProps = $props();

  $effect(() => autocomplete.setInputValue(value));
</script>

<Input
  bind:value
  autocomplete="off"
  ref={autocomplete.setAnchorEl}
  onkeydown={autocomplete.handleKeyDown}
  {...restProps}
/>
