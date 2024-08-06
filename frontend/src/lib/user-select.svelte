<script lang="ts">
  import { Input } from "flowbite-svelte";
  import { createEventDispatcher } from "svelte";
  import type { User } from "./auth";
  import UserAvatar from "./user-avatar.svelte";

  const dispatch = createEventDispatcher<{ select: User | null }>();

  export let users: User[];
  export let selectedUser: User | null = null;
  let filter: string = "";

  function select(user: User | null) {
    selectedUser = user;
    dispatch("select", user);
  }

  $: filteredUsers = users.filter(
    (user) =>
      user.name.toLowerCase().includes(filter.toLowerCase()) ||
      user.email.toLowerCase().includes(filter.toLowerCase()),
  );
</script>

<div class="flex flex-col gap-2 p-2">
  <div>
    <Input placeholder="Filter users" bind:value={filter} />
  </div>

  <button on:click={() => select(null)}>
    <UserAvatar user={{ name: "Unassigned", email: "", picture: "", exp: 0 }} />
  </button>

  {#each filteredUsers as user}
    <button on:click={() => select(user)}>
      <UserAvatar {user} />
    </button>
  {/each}
</div>
