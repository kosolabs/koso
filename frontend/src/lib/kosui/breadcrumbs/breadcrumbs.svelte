<script module lang="ts">
  import { page } from "$app/state";
  import type { Snippet } from "svelte";
  import { tv, type ClassValue } from "tailwind-variants";
  import { Link } from "../link";
  import { toTitleCase } from "../utils";

  const breadcrumbsVariants = tv({ base: "flex gap-2" });

  type Crumb = {
    path: string;
    title: string;
  };

  type BreadcrumbsProps = {
    separator?: Snippet | string;
    class?: ClassValue;
  };
</script>

<script lang="ts">
  const { separator = "›", class: className }: BreadcrumbsProps = $props();

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
</script>

<div class={breadcrumbsVariants({ className })}>
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
    <Link href={path} underline="never" color="inherit">{title}</Link>
  {/each}
</div>
