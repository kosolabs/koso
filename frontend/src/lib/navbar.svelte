<script lang="ts">
  import { logout, user } from "$lib/auth";
  import { Avatar, AvatarImage } from "$lib/components/ui/avatar";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import { UserAvatar } from "$lib/components/ui/user-select";
  import { resetMode, setMode } from "mode-watcher";
  import { KosoLogo } from "./components/ui/koso-logo";
</script>

<nav class="flex items-center bg-card p-2 shadow">
  <div class="flex items-center">
    <a href="/projects" aria-label="Home">
      <KosoLogo class="size-10" />
    </a>
    <slot name="left-items"></slot>
  </div>

  <div class="ml-auto flex items-center gap-2">
    <slot name="right-items"></slot>

    {#if $user}
      <DropdownMenu.Root closeFocus={document.body} portal={null}>
        <DropdownMenu.Trigger title={$user.email}>
          <Avatar
            class="size-9 rounded transition-all hover:brightness-110 active:scale-95"
          >
            <AvatarImage src={$user.picture} alt={$user.email} />
          </Avatar>
        </DropdownMenu.Trigger>
        <DropdownMenu.Content
          onkeydown={(event) => {
            event.stopPropagation();
          }}
        >
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
