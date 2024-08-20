<script lang="ts">
  import {
    Avatar,
    AvatarFallback,
    AvatarImage,
  } from "$lib/components/ui/avatar";
  import { Dropdown } from "flowbite-svelte";
  import { UserRound } from "lucide-svelte";
  import { createEventDispatcher } from "svelte";
  import type { User } from "./auth";
  import { Input } from "./components/ui/input";
  import UserAvatar from "./user-avatar.svelte";

  const dispatch = createEventDispatcher<{ select: User | null }>();

  export let users: User[];
  export let value: User | null = null;
  export let unassigned: string = "Unassigned";

  let open: boolean = false;
  let filter: string = "";

  function select(user: User | null) {
    value = user;
    open = false;
    dispatch("select", user);
  }

  $: filteredUsers = users.filter(
    (user) =>
      user.name.toLowerCase().includes(filter.toLowerCase()) ||
      user.email.toLowerCase().includes(filter.toLowerCase()),
  );
</script>

<button class="flex gap-1" title={value?.email || "Unassigned"}>
  <Avatar class="size-6 rounded">
    <AvatarImage src={value?.picture || ""} />
    <AvatarFallback class="rounded">
      <UserRound />
    </AvatarFallback>
  </Avatar>
  <div class="whitespace-nowrap max-md:hidden">
    {value?.name || unassigned}
  </div>
</button>
<Dropdown bind:open class="max-h-72 overflow-y-auto">
  <div slot="header" class="mb-2 ml-2 mr-2 mt-2 gap-2">
    <Input placeholder="Filter users" bind:value={filter} />
  </div>

  <div class="flex flex-col gap-2 p-2">
    <button on:click={() => select(null)}>
      <UserAvatar
        user={{ name: "Unassigned", email: "", picture: "", exp: 0 }}
      />
    </button>

    {#each filteredUsers as user}
      <button
        on:click={() => {
          value = user;
          open = false;
          dispatch("select", user);
        }}
      >
        <UserAvatar {user} />
      </button>
    {/each}
  </div>
</Dropdown>
