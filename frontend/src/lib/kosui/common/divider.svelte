<script module lang="ts">
  import type { HTMLAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import type { ClassName } from "../utils";

  export type DividerProps = {} & ClassName & HTMLAttributes<HTMLHRElement>;
</script>

<script lang="ts">
  let { class: className, ...restProps }: DividerProps = $props();
</script>

<hr class={twMerge("-mx-1 my-1", className)} {...restProps} />
