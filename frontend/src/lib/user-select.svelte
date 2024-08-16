<script lang="ts">
  import { Avatar, Dropdown, Input } from "flowbite-svelte";
  import { createEventDispatcher } from "svelte";
  import type { User } from "./auth";
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

<button class="flex gap-1">
  <Avatar src={value?.picture || ""} rounded size="xs" />
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
