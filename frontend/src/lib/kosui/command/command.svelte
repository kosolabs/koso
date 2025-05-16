<script module lang="ts">
  import type { Snippet } from "svelte";
  import { twMerge } from "tailwind-merge";
  import type { ClassName } from "../utils";
  import { CommandContext, setCommandContext } from "./command-context.svelte";

  export type CommandProps = {
    children: Snippet<[]>;
  } & ClassName;
</script>

<script lang="ts">
  let { children, class: className }: CommandProps = $props();
  setCommandContext(new CommandContext());
</script>

<div class={twMerge("flex h-full flex-col p-1", className)}>
  {@render children()}
</div>
