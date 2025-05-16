<script lang="ts">
  import { goto } from "$app/navigation";
  import { auth, getAuthContext } from "$lib/auth.svelte";
  import { command, type ActionID } from "$lib/components/ui/command-palette";
  import { KosoLogo } from "$lib/components/ui/koso-logo";
  import { UserAvatar } from "$lib/components/ui/user-select";
  import { Avatar } from "$lib/kosui/avatar";
  import { Badge } from "$lib/kosui/badge";
  import { baseClasses } from "$lib/kosui/base";
  import {
    Menu,
    MenuContent,
    MenuDivider,
    MenuHeader,
    MenuItem,
    MenuTrigger,
  } from "$lib/kosui/menu";
  import { Check, MenuIcon, UserRound } from "lucide-svelte";
  import { userPrefersMode as mode, resetMode, setMode } from "mode-watcher";
  import type { Snippet } from "svelte";
  import { twMerge } from "tailwind-merge";
  import CommandButton from "./command-button.svelte";
  import CommandMenuItem from "./command-menu-item.svelte";

  type Props = {
    left?: Snippet;
  };
  const { left }: Props = $props();

  const ctx = getAuthContext();
  $inspect(ctx);

  type Section = {
    heading: string;
    actions: ActionID[];
  }[];

  const menu: Section = [
    {
      heading: "Project",
      actions: ["ConnectToGitHub", "ExportProject", "ShareProject"],
    },
    {
      heading: "Navigation",
      actions: ["ProjectsView", "PlanView", "InboxView"],
    },
  ];

  let sections = $derived(
    menu.map((section) => {
      return {
        heading: section.heading,
        actions: section.actions
          .map((id) => command.get(id))
          .filter((action) => action !== undefined)
          .filter((action) => action.enabled()),
      };
    }),
  );

  let menuHasActions = $derived(
    sections.flatMap((section) => section.actions).length > 0,
  );
</script>

<nav
  class="bg-m3-surface-container shadow-m3-shadow/20 flex items-center border-b p-2 shadow"
>
  <div class="flex items-center">
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
      {#if menuHasActions}
        <MenuContent>
          {#each sections as section}
            {#if section.actions.length > 0}
              {#each section.actions as action}
                <CommandMenuItem {action} />
              {/each}
            {/if}
          {/each}
        </MenuContent>
      {/if}
    </Menu>
    <a href="/projects" aria-label="Home">
      <KosoLogo class="size-10" />
    </a>
    {@render left?.()}
  </div>

  <div class="ml-auto flex items-center gap-2">
    <CommandButton name="Undo" desktop />
    <CommandButton name="Redo" desktop />
    <CommandButton name="ShareProject" desktop />
    <CommandButton name="Search" desktop />
    <CommandButton name="CommandPalette" />
    <CommandButton name="InboxView" />
    <CommandButton name="PlanView" />

    {#if auth.ok()}
      <Menu>
        <MenuTrigger
          title={auth.user.email}
          class="focus-visible:outline-m3-primary focus-visible:outline-1"
        >
          <Badge
            content={ctx.user?.premium ? "👑" : ""}
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
          <MenuItem onSelect={() => goto("/profile")}>Profile</MenuItem>
          <MenuDivider />
          <MenuHeader>Theme</MenuHeader>
          <MenuItem onSelect={() => setMode("light")}>
            Light
            {#if mode.current === "light"}
              <Check class="text-m3-primary ml-auto" size={16} />
            {/if}
          </MenuItem>
          <MenuItem onSelect={() => setMode("dark")}>
            Dark
            {#if mode.current === "dark"}
              <Check class="text-m3-primary ml-auto" size={16} />
            {/if}
          </MenuItem>
          <MenuItem onSelect={() => resetMode()}>
            System
            {#if mode.current === "system"}
              <Check class="text-m3-primary ml-auto" size={16} />
            {/if}
          </MenuItem>
          <MenuDivider />
          <MenuItem onSelect={() => goto("/projects")}>Projects</MenuItem>
          <MenuItem onSelect={() => auth.logout()}>Logout</MenuItem>
        </MenuContent>
      </Menu>
    {/if}
  </div>
</nav>
