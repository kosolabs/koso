<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { getAuthContext } from "$lib/auth.svelte";
  import {
    getRegistryContext,
    type ActionID,
  } from "$lib/components/ui/command-palette";
  import { Action } from "$lib/kosui/command";
  import { nav } from "$lib/nav.svelte";
  import { House } from "lucide-svelte";
  import { onMount, type Snippet } from "svelte";

  type Props = {
    children: Snippet;
  };
  const { children }: Props = $props();

  const command = getRegistryContext();
  const auth = getAuthContext();

  $effect(() => {
    if (!auth.ok()) {
      nav.pushRedirectOnUserNotAuthenticated();
      goto("/");
    }
  });

  const actions: Action<ActionID>[] = [
    new Action({
      id: "ProjectsView",
      callback: () => goto(`/projects`),
      title: "All projects",
      description: "Navigate to all projects view",
      icon: House,
      enabled: () => page.url.pathname !== `/projects`,
    }),
  ];

  onMount(() => {
    return command.register(...actions);
  });
</script>

{#if auth.ok()}
  {@render children()}
{:else}
  <div class="flex flex-col items-center justify-center rounded border p-4">
    <div class="text-l">Redirecting to login...</div>
  </div>
{/if}
