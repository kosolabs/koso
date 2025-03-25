<script lang="ts">
  import { Navbar } from "$lib/components/ui/navbar";
  import { Breadcrumbs } from "$lib/kosui/breadcrumbs";
  import { type Snippet } from "svelte";

  type Props = {
    children: Snippet;
  };

  const { children }: Props = $props();
</script>

<Navbar />

<div class="flex flex-col gap-2 p-2">
  <h1 class="text-3xl font-thin">
    <Breadcrumbs />
  </h1>
  <hr />
  {@render children()}
</div>
