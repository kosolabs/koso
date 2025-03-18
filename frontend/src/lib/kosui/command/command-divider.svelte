<script module lang="ts">
  import { twMerge } from "tailwind-merge";
  import type { ClassName } from "../utils";

  export type CommandDividerProps = {} & ClassName;
</script>

<script lang="ts">
  let { class: className }: CommandDividerProps = $props();
</script>

<hr class={twMerge("-mx-1 my-1", className)} />
