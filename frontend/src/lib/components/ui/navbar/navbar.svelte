<script lang="ts">
  import { getAuthContext } from "$lib/auth.svelte";
  import {
    ActionIds,
    getRegistryContext,
  } from "$lib/components/ui/command-palette";
  import { KosoLogo } from "$lib/components/ui/koso-logo";
  import { UserAvatar } from "$lib/components/ui/user-select";
  import { Avatar } from "$lib/kosui/avatar";
  import { Badge } from "$lib/kosui/badge";
  import { baseClasses } from "$lib/kosui/base";
  import {
    Menu,
    MenuActions,
    MenuContent,
    MenuDivider,
    MenuTrigger,
  } from "$lib/kosui/menu";
  import { MenuIcon, UserRound } from "@lucide/svelte";
  import type { Snippet } from "svelte";
  import { twMerge } from "tailwind-merge";
  import { StorybookNavigationActionIds } from "../../../../routes/storybook/+layout.svelte";
  import CommandButton from "./command-button.svelte";
  import NavigateButton from "./navigate-button.svelte";

  type Props = {
    left?: Snippet;
    breadcrumbs?: string[];
  };
  const { left, breadcrumbs }: Props = $props();

  const auth = getAuthContext();
  const command = getRegistryContext();

  let navActions = $derived(
    [
      // Project
      ActionIds.ConnectToGitHub,
      ActionIds.ExportProject,
      ActionIds.ShareProject,

      // Navigation
      ActionIds.Home,
      ActionIds.ProjectsView,
      ActionIds.PlanView,
      ActionIds.InboxView,
      ActionIds.NextDashView,
      ActionIds.Storybook,
      StorybookNavigationActionIds.Alerts,
      StorybookNavigationActionIds.Autocomplete,
      StorybookNavigationActionIds.Avatar,
      StorybookNavigationActionIds.Badge,
      StorybookNavigationActionIds.Buttons,
      StorybookNavigationActionIds.Chips,
      StorybookNavigationActionIds.CodeMirror,
      StorybookNavigationActionIds.Command,
      StorybookNavigationActionIds.Dialogs,
      StorybookNavigationActionIds.Fab,
      StorybookNavigationActionIds.Goto,
      StorybookNavigationActionIds.Inputs,
      StorybookNavigationActionIds.Links,
      StorybookNavigationActionIds.Markdown,
      StorybookNavigationActionIds.Menus,
      StorybookNavigationActionIds.ProgressIndicators,
      StorybookNavigationActionIds.Shortcuts,
      StorybookNavigationActionIds.Toggles,
      StorybookNavigationActionIds.Tooltips,
    ]
      .map((id) => command.get(id))
      .filter((action) => action !== undefined),
  );

  let profileActions = $derived(
    [
      // Theme
      ActionIds.LightTheme,
      ActionIds.DarkTheme,
      ActionIds.SystemTheme,

      // Account
      ActionIds.ProfileView,
      ActionIds.Logout,
    ]
      .map((id) => command.get(id))
      .filter((action) => action !== undefined),
  );
</script>

<nav
  class="bg-m3-surface-container shadow-m3-shadow/20 flex flex-col overflow-hidden border-b shadow"
>
  <div
    class="bg-m3-surface-container flex items-center overflow-hidden border-b p-2"
  >
    <div class="flex flex-col gap-2"></div>
    <div class="flex items-center">
      {#if navActions.length > 0}
        <Menu>
          <MenuTrigger
            title="Project menu"
            class={twMerge(
              baseClasses({
                variant: "plain",
                color: "primary",
                shape: "circle",
                focus: true,
                hover: true,
              }),
              "mr-1 p-2 transition-all active:scale-95",
            )}
          >
            <MenuIcon size={20} />
          </MenuTrigger>
          <MenuContent>
            <MenuActions actions={navActions} />
          </MenuContent>
        </Menu>
      {/if}
      <a href="/projects" aria-label="Home">
        <KosoLogo class="size-10" />
      </a>
      {@render left?.()}
    </div>

    <div class="ml-auto flex items-center gap-2">
      <CommandButton name="Undo" desktop />
      <CommandButton name="Redo" desktop />
      <CommandButton name="ShareProject" desktop />
      <CommandButton name="DetailPanelClose" desktop />
      <CommandButton name="DetailPanelOpen" desktop />
      <CommandButton name="Search" desktop />
      <CommandButton name="CommandPalette" />
      <NavigateButton name="InboxView" desktop />
      <NavigateButton name="PlanView" desktop />

      {#if auth.ok()}
        <Menu>
          <MenuTrigger
            title={auth.user.email}
            class="focus-visible:outline-m3-primary focus-visible:outline-1"
          >
            <Badge
              content={auth.fullUser?.premium ? "👑" : ""}
              variant="plain"
              class="m-[.23rem] rotate-45"
            >
              <Avatar
                src={auth.user.picture}
                alt={auth.user.email}
                shape="circle"
                class="transition-all active:scale-95 active:brightness-110"
              >
                <UserRound />
              </Avatar>
            </Badge>
          </MenuTrigger>
          <MenuContent>
            <UserAvatar class="p-1" user={auth.user} />
            <MenuDivider />
            <MenuActions actions={profileActions} />
          </MenuContent>
        </Menu>
      {/if}
    </div>
  </div>
  {#if breadcrumbs}
    <div class="flex items-center py-1 pl-4 text-sm font-thin">
      {#each breadcrumbs as crumb, i (i)}
        <span>{crumb}</span>
        {#if i < breadcrumbs.length - 1}
          <span class="px-1 text-gray-400">></span>
        {/if}
      {/each}
    </div>
  {/if}
</nav>
