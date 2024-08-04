<script lang="ts">
  import { Avatar } from "flowbite-svelte";
  import type { User } from "./auth";
  import { cn } from "./utils";

  export let user: User;
</script>

<div class={cn("flex items-center gap-2")}>
  <Avatar src={user.picture} rounded />
  <div class="flex flex-col">
    <div class="text-sm">{user.name}</div>
    <div class="text-sm font-medium">{user.email}</div>
  </div>
</div>
