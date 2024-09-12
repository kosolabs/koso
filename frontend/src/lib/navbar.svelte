<script lang="ts">
  import kosoLogo from "$lib/assets/koso.svg";
  import { logout, user } from "$lib/auth";
  import { Avatar } from "$lib/components/ui/avatar";
  import AvatarImage from "$lib/components/ui/avatar/avatar-image.svelte";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import { resetMode, setMode } from "mode-watcher";
  import UserAvatar from "./user-avatar.svelte";
</script>

<nav class="flex items-center bg-card p-2 shadow">
  <div class="flex items-center">
    <a href="/projects">
      <img class="size-10" alt="Koso Logo" src={kosoLogo} />
    </a>
    <slot name="left-items"></slot>
  </div>

  <div class="ml-auto flex items-center gap-2">
    <slot name="right-items"></slot>

    {#if $user}
      <DropdownMenu.Root>
        <DropdownMenu.Trigger>
          <Avatar
            class="size-9 rounded transition-all hover:brightness-110 active:scale-95"
          >
            <AvatarImage src={$user.picture}></AvatarImage>
          </Avatar>
        </DropdownMenu.Trigger>
        <DropdownMenu.Content>
          <DropdownMenu.Label>
            <UserAvatar user={$user} />
          </DropdownMenu.Label>
          <DropdownMenu.Separator />
          <DropdownMenu.Sub>
            <DropdownMenu.SubTrigger>Theme</DropdownMenu.SubTrigger>
            <DropdownMenu.SubContent>
              <DropdownMenu.Item on:click={() => setMode("light")}>
                Light
              </DropdownMenu.Item>
              <DropdownMenu.Item on:click={() => setMode("dark")}>
                Dark
              </DropdownMenu.Item>
              <DropdownMenu.Item on:click={() => resetMode()}>
                System
              </DropdownMenu.Item>
            </DropdownMenu.SubContent>
          </DropdownMenu.Sub>
          <DropdownMenu.Item href="/projects">Projects</DropdownMenu.Item>
          <DropdownMenu.Item on:click={() => logout()}>Logout</DropdownMenu.Item
          >
        </DropdownMenu.Content>
      </DropdownMenu.Root>
    {/if}
  </div>
</nav>
