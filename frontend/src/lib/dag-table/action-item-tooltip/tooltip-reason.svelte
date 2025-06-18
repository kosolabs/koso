<script lang="ts">
  import { Header } from "$lib/kosui/common";
  import type { Icon } from "@lucide/svelte";
  import type { Snippet } from "svelte";

  export type ReasonProps = {
    title: string;
    icon: typeof Icon;
    children: Snippet;
  };
  let { title, icon: ReasonIcon, children }: ReasonProps = $props();
</script>

<div class="flex gap-2">
  <div class="flex w-8 flex-none items-start justify-center">
    <ReasonIcon class="text-m3-primary" />
  </div>
  <div class="flex flex-col gap-1">
    <Header class="p-0">{title}</Header>
    <div>
      {@render children()}
    </div>
  </div>
</div>
