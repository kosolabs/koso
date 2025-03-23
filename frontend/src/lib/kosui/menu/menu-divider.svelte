<script module lang="ts">
  import type { HTMLAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import type { ClassName } from "../utils";

  export type MenuDividerProps = {} & ClassName & HTMLAttributes<HTMLHRElement>;
</script>

<script lang="ts">
  let { class: className, ...restProps }: MenuDividerProps = $props();
</script>

<hr class={twMerge("-mx-1 my-1", className)} {...restProps} />
