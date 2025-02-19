<script lang="ts">
  import type { User } from "$lib/auth.svelte";
  import {
    Avatar,
    AvatarFallback,
    AvatarImage,
  } from "$lib/components/ui/avatar";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import { Input } from "$lib/kosui/input";
  import { Shortcut } from "$lib/shortcuts";
  import { UserRound } from "lucide-svelte";
  import { tick } from "svelte";
  import { UserAvatar } from ".";
  import ResponsiveText from "../responsive-text/responsive-text.svelte";

  type Props = {
    users: User[];
    value: User | null;
    unassigned?: string;
    editable?: boolean;
    onOpenChange?: (open: boolean) => void;
    onSelect?: (select: User | null) => void;
  };
  let {
    users,
    value = null,
    unassigned = "Unassigned",
    editable = true,
    onOpenChange,
    onSelect,
  }: Props = $props();

  let filter: string = $state("");

  function select(user: User | null) {
    value = user;
    onSelect?.(user);
  }

  let open: boolean = $state(false);
</script>

{#if editable}
  <DropdownMenu.Root
    bind:open={
      () => open,
      (newOpen) => {
        onOpenChange?.(newOpen);
        tick().then(() => (open = newOpen));
      }
    }
  >
    <DropdownMenu.Trigger
      class="flex items-center gap-2"
      title={value?.email || "Unassigned"}
    >
      <Avatar class="size-6 rounded">
        <AvatarImage src={value?.picture || ""} alt={value?.email} />
        <AvatarFallback class="rounded">
          <UserRound />
        </AvatarFallback>
      </Avatar>
      <ResponsiveText>{value?.name || unassigned}</ResponsiveText>
    </DropdownMenu.Trigger>
    <div
      role="none"
      onkeydown={(event) => {
        if (Shortcut.CANCEL.matches(event)) {
          open = false;
        }
        event.stopPropagation();
      }}
    >
      <DropdownMenu.Content
        class="min-w-64"
        portalProps={{ disabled: true }}
        preventScroll={false}
      >
        <DropdownMenu.Label>
          <Input
            bind:value={filter}
            class="w-full"
            placeholder="Filter users"
            name="Filter users"
          />
        </DropdownMenu.Label>
        <DropdownMenu.Separator />
        <DropdownMenu.Group class="max-h-64 overflow-y-auto">
          <DropdownMenu.Item onSelect={() => select(null)}>
            <UserAvatar
              user={{ name: "Unassigned", email: "", picture: "", exp: 0 }}
            />
          </DropdownMenu.Item>
          {#each users as user}
            <DropdownMenu.Item onSelect={() => select(user)}>
              <UserAvatar {user} />
            </DropdownMenu.Item>
          {/each}
        </DropdownMenu.Group>
      </DropdownMenu.Content>
    </div>
  </DropdownMenu.Root>
{:else}
  <div class="flex items-center gap-2" title={value?.email || "Unassigned"}>
    <Avatar class="size-6 rounded">
      <AvatarImage src={value?.picture || ""} alt={value?.email} />
      <AvatarFallback class="rounded">
        <UserRound />
      </AvatarFallback>
    </Avatar>
    <ResponsiveText>{value?.name || unassigned}</ResponsiveText>
  </div>
{/if}
