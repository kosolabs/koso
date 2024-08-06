<script lang="ts">
  import { Avatar } from "flowbite-svelte";
  import type { User } from "./auth";

  export let user: User;
</script>

<div class="flex items-center gap-2 rounded p-2 text-left hover:bg-primary-100">
  <Avatar src={user.picture} rounded />
  <div class="flex flex-col">
    <div class="text-sm">{user.name}</div>
    <div class="text-sm font-medium">{user.email}</div>
  </div>
</div>
