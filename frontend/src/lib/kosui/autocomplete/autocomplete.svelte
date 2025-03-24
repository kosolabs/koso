<script module lang="ts">
  import type { Snippet } from "svelte";
  import type { ClassName } from "../utils";
  import { newAutocompleteContext } from "./autocomplete-context.svelte";

  export type AutocompleteProps = {
    children: Snippet;
  } & ClassName;
</script>

<script lang="ts">
  let { children }: AutocompleteProps = $props();

  newAutocompleteContext();
</script>

{@render children()}
