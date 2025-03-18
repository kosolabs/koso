<script module lang="ts">
  import type { Snippet } from "svelte";
  import { twMerge } from "tailwind-merge";
  import type { ClassName } from "../utils";
  import { newCommandContext } from "./command-context.svelte";

  export type CommandProps = {
    children: Snippet<[]>;
  } & ClassName;
</script>

<script lang="ts">
  let { children, class: className }: CommandProps = $props();
  const ctx = newCommandContext();
</script>

<div bind:this={ctx.el} class={twMerge("p-1", className)}>
  {@render children()}
</div>
