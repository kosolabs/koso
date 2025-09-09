<script lang="ts">
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { getAuthContext } from "$lib/auth.svelte";
  import { getRegistryContext } from "$lib/components/ui/command-palette";
  import {
    ActionIds,
    Categories,
  } from "$lib/components/ui/command-palette/command-palette.svelte";
  import { nav } from "$lib/nav.svelte";
  import { NavigationAction } from "$lib/navigation-action";
  import { LogOut, Rows3, UserCog } from "@lucide/svelte";
  import { Action } from "kosui";
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
      goto(resolve("/"));
    }
  });

  const actions: Action[] = [
    new NavigationAction({
      id: ActionIds.ProjectsView,
      href: "/projects",
      category: Categories.Navigation,
      name: "List Projects",
      description: "Navigate to All Projects view",
      icon: Rows3,
    }),
    new NavigationAction({
      id: ActionIds.ProfileView,
      href: "/profile",
      category: Categories.Account,
      name: "User Profile",
      description: "Navigate to your user profile settings",
      icon: UserCog,
    }),
    new Action({
      id: ActionIds.Logout,
      callback: () => auth.logout(),
      category: Categories.Account,
      name: "Logout",
      description: "Log out of your account",
      icon: LogOut,
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
