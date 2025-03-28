<script lang="ts">
  import { goto } from "$app/navigation";
  import { auth } from "$lib/auth.svelte";
  import { KosoLogo } from "$lib/components/ui/koso-logo";
  import { UserAvatar } from "$lib/components/ui/user-select";
  import { Avatar } from "$lib/kosui/avatar";
  import { Menu, MenuContent, MenuDivider, MenuItem } from "$lib/kosui/menu";
  import { Check, UserRound } from "lucide-svelte";
  import { userPrefersMode as mode, resetMode, setMode } from "mode-watcher";
  import type { Snippet } from "svelte";
  import MenuHeader from "../../../kosui/menu/menu-header.svelte";
  import MenuTrigger from "../../../kosui/menu/menu-trigger.svelte";
  import NavbarCommandButton from "./navbar-command-button.svelte";

  type Props = {
    context?: Snippet;
    left?: Snippet;
    right?: Snippet;
  };
  const { context, left, right }: Props = $props();
</script>

<nav
  class="bg-m3-surface-container shadow-m3-shadow/20 flex items-center border-b p-2 shadow"
>
  <div class="flex items-center">
    {@render context?.()}
    <a href="/projects" aria-label="Home">
      <KosoLogo class="size-10" />
    </a>
    {@render left?.()}
  </div>

  <div class="ml-auto flex items-center gap-2">
    {@render right?.()}

    <NavbarCommandButton name="CommandPalette" />

    {#if auth.ok()}
      <Menu>
        <MenuTrigger
          title={auth.user.email}
          class="focus-visible:outline-m3-primary focus-visible:outline-1"
        >
          <Avatar
            src={auth.user.picture}
            alt={auth.user.email}
            shape="circle"
            class="transition-all active:scale-95 active:brightness-110"
          >
            <UserRound />
          </Avatar>
        </MenuTrigger>
        <MenuContent>
          <UserAvatar class="p-1" user={auth.user} />
          <MenuDivider />
          <MenuItem onSelect={() => goto("/profile")}>Profile</MenuItem>
          <MenuDivider />
          <MenuHeader>Theme</MenuHeader>
          <MenuItem onSelect={() => setMode("light")}>
            Light
            {#if $mode === "light"}
              <Check class="text-m3-primary ml-auto" size={16} />
            {/if}
          </MenuItem>
          <MenuItem onSelect={() => setMode("dark")}>
            Dark
            {#if $mode === "dark"}
              <Check class="text-m3-primary ml-auto" size={16} />
            {/if}
          </MenuItem>
          <MenuItem onSelect={() => resetMode()}>
            System
            {#if $mode === "system"}
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
