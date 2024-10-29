<script lang="ts">
  import { auth } from "$lib/auth.svelte";
  import { Avatar, AvatarImage } from "$lib/components/ui/avatar";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import { UserAvatar } from "$lib/components/ui/user-select";
  import { resetMode, setMode } from "mode-watcher";
  import type { Snippet } from "svelte";
  import { KosoLogo } from "./components/ui/koso-logo";

  type Props = {
    left?: Snippet;
    right?: Snippet;
  };
  const { left, right }: Props = $props();
</script>

<nav class="flex items-center bg-card p-2 shadow">
  <div class="flex items-center">
    <a href="/projects" aria-label="Home">
      <KosoLogo class="size-10" />
    </a>
    {@render left?.()}
  </div>

  <div class="ml-auto flex items-center gap-2">
    {@render right?.()}

    {#if auth.ok()}
      <DropdownMenu.Root closeFocus={document.body} portal={null}>
        <DropdownMenu.Trigger title={auth.user.email}>
          <Avatar
            class="size-9 rounded transition-all hover:brightness-110 active:scale-95"
          >
            <AvatarImage src={auth.user.picture} alt={auth.user.email} />
          </Avatar>
        </DropdownMenu.Trigger>
        <DropdownMenu.Content
          onkeydown={(event) => {
            event.stopPropagation();
          }}
        >
          <DropdownMenu.Label>
            <UserAvatar user={auth.user} />
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
          <DropdownMenu.Item on:click={() => auth.logout()}>
            Logout
          </DropdownMenu.Item>
        </DropdownMenu.Content>
      </DropdownMenu.Root>
    {/if}
  </div>
</nav>
