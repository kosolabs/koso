<script lang="ts">
  import { page } from "$app/state";
  import Navbar from "$lib/navbar.svelte";
  import { type Snippet } from "svelte";

  type Props = {
    children: Snippet;
  };

  const { children }: Props = $props();

  let title = $derived.by(() => {
    const slugs = page.url.pathname.split("/");
    const words = slugs[slugs.length - 1].split("-");
    return words.map((w) => w.charAt(0).toUpperCase() + w.slice(1)).join(" ");
  });
</script>

<Navbar />

<div class="flex flex-col gap-2 p-2">
  <h1 class="text-3xl font-thin">{title}</h1>
  <hr />
  {@render children()}
</div>
