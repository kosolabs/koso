<script lang="ts">
  import {
    Avatar,
    AvatarFallback,
    AvatarImage,
  } from "$lib/components/ui/avatar";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import { Input } from "$lib/components/ui/input";
  import { UserRound } from "lucide-svelte";
  import { createEventDispatcher, SvelteComponent } from "svelte";
  import type { User } from "./auth";
  import { handleOpenChange } from "./popover-monitor";
  import UserAvatar from "./user-avatar.svelte";

  const dispatch = createEventDispatcher<{ select: User | null }>();

  export let users: User[];
  export let value: User | null = null;
  export let unassigned: string = "Unassigned";

  let filter: string = "";

  let open: boolean;
  let component: SvelteComponent;
  $: handleOpenChange(open, component);

  function select(user: User | null) {
    value = user;
    dispatch("select", user);
  }

  $: filteredUsers = users.filter(
    (user) =>
      user.name.toLowerCase().includes(filter.toLowerCase()) ||
      user.email.toLowerCase().includes(filter.toLowerCase()),
  );
</script>

<DropdownMenu.Root bind:open bind:this={component}>
  <DropdownMenu.Trigger
    class="flex items-center gap-2"
    title={value?.email || "Unassigned"}
  >
    <Avatar class="size-6 rounded">
      <AvatarImage src={value?.picture || ""} />
      <AvatarFallback class="rounded">
        <UserRound />
      </AvatarFallback>
    </Avatar>
    <div class="whitespace-nowrap max-sm:hidden">
      {value?.name || unassigned}
    </div>
  </DropdownMenu.Trigger>
  <DropdownMenu.Content class="min-w-64">
    <DropdownMenu.Label>
      <Input placeholder="Filter users" bind:value={filter} />
    </DropdownMenu.Label>
    <DropdownMenu.Separator />
    <DropdownMenu.Group class="max-h-64 overflow-y-auto">
      <DropdownMenu.Item on:click={() => select(null)}>
        <UserAvatar
          user={{ name: "Unassigned", email: "", picture: "", exp: 0 }}
        />
      </DropdownMenu.Item>
      {#each filteredUsers as user}
        <DropdownMenu.Item on:click={() => select(user)}>
          <UserAvatar {user} />
        </DropdownMenu.Item>
      {/each}
    </DropdownMenu.Group>
  </DropdownMenu.Content>
</DropdownMenu.Root>
