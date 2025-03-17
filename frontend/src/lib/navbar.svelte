<script lang="ts">
  import { goto } from "$app/navigation";
  import { auth } from "$lib/auth.svelte";
  import { KosoLogo } from "$lib/components/ui/koso-logo";
  import { UserAvatar } from "$lib/components/ui/user-select";
  import { Avatar } from "$lib/kosui/avatar";
  import { Menu, MenuItem } from "$lib/kosui/menu";
  import { Check, UserRound } from "lucide-svelte";
  import { userPrefersMode as mode, resetMode, setMode } from "mode-watcher";
  import type { Snippet } from "svelte";
  import MenuTrigger from "./kosui/menu/menu-trigger.svelte";

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

    {#if auth.ok()}
      <Menu>
        {#snippet trigger(menuTriggerProps)}
          <MenuTrigger
            title={auth.user.email}
            class="focus-visible:outline-m3-primary focus-visible:outline-1"
            {...menuTriggerProps}
          >
            <Avatar
              src={auth.user.picture}
              alt={auth.user.email}
              class="transition-all active:scale-95 active:brightness-110"
            >
              <UserRound />
            </Avatar>
          </MenuTrigger>
        {/snippet}
        {#snippet content(menuItemProps)}
          <UserAvatar class="p-1" user={auth.user} />
          <hr class="my-1" />
          <MenuItem onSelect={() => goto("/profile")} {...menuItemProps}>
            Profile
          </MenuItem>
          <hr class="my-1" />
          <div class="px-2 py-1 text-xs uppercase">Theme</div>
          <MenuItem onSelect={() => setMode("light")} {...menuItemProps}>
            Light
            {#if $mode === "light"}
              <Check class="text-m3-primary ml-auto" size={16} />
            {/if}
          </MenuItem>
          <MenuItem onSelect={() => setMode("dark")} {...menuItemProps}>
            Dark
            {#if $mode === "dark"}
              <Check class="text-m3-primary ml-auto" size={16} />
            {/if}
          </MenuItem>
          <MenuItem onSelect={() => resetMode()} {...menuItemProps}>
            System
            {#if $mode === "system"}
              <Check class="text-m3-primary ml-auto" size={16} />
            {/if}
          </MenuItem>
          <hr class="my-1" />
          <MenuItem onSelect={() => goto("/projects")} {...menuItemProps}>
            Projects
          </MenuItem>
          <MenuItem onSelect={() => auth.logout()} {...menuItemProps}>
            Logout
          </MenuItem>
        {/snippet}
      </Menu>
    {/if}
  </div>
</nav>
