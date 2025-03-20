<script module lang="ts">
  import type { Snippet } from "svelte";
  import { twMerge } from "tailwind-merge";
  import type { ClassName } from "../utils";

  export type CommandContentProps = { children: Snippet } & ClassName;
</script>

<script lang="ts">
  let { class: className, children }: CommandContentProps = $props();
</script>

<div class={twMerge("h-full overflow-scroll", className)}>
  {@render children()}
</div>
