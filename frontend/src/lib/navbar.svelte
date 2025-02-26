<script lang="ts">
  import { goto } from "$app/navigation";
  import { auth } from "$lib/auth.svelte";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import { KosoLogo } from "$lib/components/ui/koso-logo";
  import { UserAvatar } from "$lib/components/ui/user-select";
  import { Avatar } from "$lib/kosui/avatar";
  import { UserRound } from "lucide-svelte";
  import { resetMode, setMode } from "mode-watcher";
  import type { Snippet } from "svelte";

  type Props = {
    left?: Snippet;
    right?: Snippet;
  };
  const { left, right }: Props = $props();

  let open: boolean = $state(false);
</script>

<nav class="bg-card flex items-center border-b p-2 shadow-sm">
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
        bind:open={
          () => open,
          (newOpen) => {
            open = newOpen;
          }
        }
      >
        <DropdownMenu.Trigger
          title={auth.user.email}
          class="focus-visible:outline-m3-primary rounded-m3 focus-visible:outline-1"
        >
          <Avatar
            src={auth.user.picture}
            alt={auth.user.email}
            class="transition-all active:scale-95 active:brightness-110"
          >
            <UserRound />
          </Avatar>
        </DropdownMenu.Trigger>
        <div
          role="none"
          onkeydown={(event) => {
            if (event.key === "Escape") {
              open = false;
            }
            event.stopPropagation();
          }}
        >
          <DropdownMenu.Content
            preventScroll={false}
            portalProps={{ disabled: true }}
          >
            <DropdownMenu.Label>
              <UserAvatar user={auth.user} />
            </DropdownMenu.Label>
            <DropdownMenu.Separator />
            <DropdownMenu.Item onSelect={() => goto("/profile")}>
              Profile
            </DropdownMenu.Item>
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
        </div>
      </DropdownMenu.Root>
    {/if}
  </div>
</nav>
