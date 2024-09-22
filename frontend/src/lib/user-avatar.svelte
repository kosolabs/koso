<script lang="ts">
  import { Avatar, AvatarFallback, AvatarImage } from "$lib/ui/avatar";
  import { UserRound } from "lucide-svelte";
  import type { User } from "./auth";

  export let user: User;

  console.log(user.picture);
</script>

<div class="flex items-center gap-2 text-left">
  <Avatar>
    {#if user.picture}
      <AvatarImage src={user.picture} alt={user.email} />
    {:else}
      <AvatarFallback class="rounded">
        <UserRound />
      </AvatarFallback>
    {/if}
  </Avatar>
  <div class="flex flex-col">
    <div class="text-sm">{user.name}</div>
    <div class="text-sm font-medium">{user.email}</div>
  </div>
</div>
