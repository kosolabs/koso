<script lang="ts">
  import { cn } from "$lib/utils";
  import type { HTMLImgAttributes } from "svelte/elements";

  type Props = HTMLImgAttributes;

  let { class: classes, ...props }: Props = $props();
</script>

<img class={cn("aspect-square h-full w-full", classes)} {...props} />
