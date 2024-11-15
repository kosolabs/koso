<script lang="ts">
  import type { User } from "$lib/auth.svelte";
  import {
    Avatar,
    AvatarFallback,
    AvatarImage,
  } from "$lib/components/ui/avatar";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import { Input } from "$lib/components/ui/input";
  import { UserRound } from "lucide-svelte";
  import { UserAvatar } from ".";

  type Props = {
    users: User[];
    value: User | null;
    closeFocus?: HTMLElement;
    unassigned?: string;
    onselect?: (select: User | null) => void;
  };
  let {
    users,
    value = null,
    closeFocus,
    unassigned = "Unassigned",
    onselect,
  }: Props = $props();

  let filter: string = $state("");

  function select(user: User | null) {
    value = user;
    onselect?.(user);
  }

  const filteredUsers = $derived.by(() =>
    users.filter(
      (user) =>
        user.name.toLowerCase().includes(filter.toLowerCase()) ||
        user.email.toLowerCase().includes(filter.toLowerCase()),
    ),
  );
</script>

<DropdownMenu.Root {closeFocus} portal={null}>
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
    <div class="whitespace-nowrap max-sm:hidden">
      {value?.name || unassigned}
    </div>
  </DropdownMenu.Trigger>
  <DropdownMenu.Content
    class="min-w-64"
    onkeydown={(event) => {
      event.stopPropagation();
    }}
  >
    <DropdownMenu.Label>
      <Input
        placeholder="Filter users"
        name="Filter users"
        bind:value={filter}
      />
    </DropdownMenu.Label>
    <DropdownMenu.Separator />
    <DropdownMenu.Group class="max-h-64 overflow-y-auto">
      <DropdownMenu.Item on:click={() => select(null)}>
        <UserAvatar
          user={{
            name: "Unassigned",
            email: "",
            picture: "",
            exp: 0,
          }}
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
