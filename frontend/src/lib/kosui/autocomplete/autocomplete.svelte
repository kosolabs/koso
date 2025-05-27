<script module lang="ts">
  import type { Snippet } from "svelte";
  import type { ClassName } from "../utils";
  import {
    AutocompleteContext,
    setAutocompleteContext,
  } from "./autocomplete-context.svelte";

  export type AutocompleteProps = {
    children: Snippet;
  } & ClassName;
</script>

<script lang="ts">
  let { children }: AutocompleteProps = $props();

  setAutocompleteContext(new AutocompleteContext());
</script>

{@render children()}
