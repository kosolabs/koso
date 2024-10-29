<script lang="ts">
  import { goto } from "$app/navigation";
  import { auth } from "$lib/auth.svelte";
  import { pushRedirectOnUserNotAuthenticated } from "$lib/nav";
  import type { Snippet } from "svelte";

  type Props = {
    children: Snippet;
  };
  const { children }: Props = $props();

  $effect(() => {
    if (!auth.ok()) {
      pushRedirectOnUserNotAuthenticated();
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
