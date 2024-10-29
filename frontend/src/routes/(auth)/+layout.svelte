<script lang="ts">
  import { goto } from "$app/navigation";
  import { auth } from "$lib/auth.svelte";
  import { nav } from "$lib/nav.svelte";
  import type { Snippet } from "svelte";

  type Props = {
    children: Snippet;
  };
  const { children }: Props = $props();

  $effect(() => {
    if (!auth.ok()) {
      nav.pushRedirectOnUserNotAuthenticated();
      goto("/");
    }
  });
</script>

{#if auth.ok()}
  {@render children()}
{:else}
  <div class="flex flex-col items-center justify-center rounded border p-4">
    <div class="text-l">Redirecting to login...</div>
  </div>
{/if}
