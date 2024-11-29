<script lang="ts">
  import { goto } from "$app/navigation";
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

<nav class="flex items-center border-b bg-card p-2 shadow">
  <div class="flex items-center">
    <a href="/projects" aria-label="Home">
      <KosoLogo class="size-10" />
    </a>
    {@render left?.()}
  </div>

  <div class="ml-auto flex items-center gap-2">
    {@render right?.()}

    {#if auth.ok()}
      <DropdownMenu.Root
        onOpenChange={(open) => {
          if (!open) {
            document.body.focus();
          }
        }}
      >
        <DropdownMenu.Trigger title={auth.user.email}>
          <Avatar
            class="size-9 rounded transition-all hover:brightness-110 active:scale-95"
          >
            <AvatarImage src={auth.user.picture} alt={auth.user.email} />
          </Avatar>
        </DropdownMenu.Trigger>
        <DropdownMenu.Portal>
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
                <DropdownMenu.Item onSelect={() => setMode("light")}>
                  Light
                </DropdownMenu.Item>
                <DropdownMenu.Item onSelect={() => setMode("dark")}>
                  Dark
                </DropdownMenu.Item>
                <DropdownMenu.Item onSelect={() => resetMode()}>
                  System
                </DropdownMenu.Item>
              </DropdownMenu.SubContent>
            </DropdownMenu.Sub>
            <DropdownMenu.Item onSelect={() => goto("/projects")}>
              Projects
            </DropdownMenu.Item>
            <DropdownMenu.Item onSelect={() => auth.logout()}>
              Logout
            </DropdownMenu.Item>
          </DropdownMenu.Content>
        </DropdownMenu.Portal>
      </DropdownMenu.Root>
    {/if}
  </div>
</nav>
