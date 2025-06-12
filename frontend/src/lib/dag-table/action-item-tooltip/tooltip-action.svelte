<script lang="ts">
  import { Box } from "$lib/kosui/box";
  import { Header } from "$lib/kosui/common";
  import type { Snippet } from "svelte";

  export type ReasonProps = {
    title: string;
    score: number;
    children: Snippet;
  };
  let { title, score, children }: ReasonProps = $props();
</script>

<div class="flex gap-2">
  <div class="flex w-8 flex-none items-start justify-center">
    <Box
      variant="filled"
      shape="rounded"
      color="secondary"
      class="cursor-default px-1 text-xs"
    >
      +{score}
    </Box>
  </div>
  <div class="flex flex-col gap-1">
    <Header color="secondary" class="p-0">{title}</Header>
    <div>
      {@render children()}
    </div>
  </div>
</div>
