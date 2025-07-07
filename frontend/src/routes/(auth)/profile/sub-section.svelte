<script lang="ts">
  import type { Icon } from "@lucide/svelte";
  import type { Snippet } from "svelte";

  type Props = {
    title: string;
    icon?: typeof Icon;
    children: Snippet;
  };
  const { title, icon: IconComponent, children }: Props = $props();
</script>

<div class="flex flex-col gap-2 rounded border p-2">
  <h2 class="flex items-center gap-2 text-xl font-thin">
    {#if IconComponent}
      <IconComponent size="24" />
    {/if}
    {title}
  </h2>
  <div>
    {@render children()}
  </div>
</div>
