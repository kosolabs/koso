<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import type { Snippet } from "svelte";
  import { cn, toTitleCase } from "../utils";

  type Props = {
    separator?: Snippet | string;
    class?: string;
  };
  const { separator = "›", class: className }: Props = $props();

  type Crumb = {
    path: string;
    title: string;
  };

  let crumbs = $derived.by(() => {
    const parts = page.url.pathname.split("/");
    const crumbs: Crumb[] = [];
    for (let i = 1; i < parts.length; i++) {
      crumbs.push({
        path: parts.slice(0, i + 1).join("/"),
        title: toTitleCase(parts[i]),
      });
    }
    return crumbs;
  });
  $inspect(crumbs);
</script>

<div class={cn("flex gap-2", className)}>
  {#each crumbs as { path, title }, index}
    {#if index !== 0}
      <div>
        {#if typeof separator === "function"}
          {@render separator()}
        {:else}
          {separator}
        {/if}
      </div>
    {/if}
    <button onclick={() => goto(path)}>{title}</button>
  {/each}
</div>
