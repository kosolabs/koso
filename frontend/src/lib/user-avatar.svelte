<script lang="ts">
  import { UserRound } from "lucide-svelte";
  import type { User } from "./auth";
  import { Avatar, AvatarImage } from "./components/ui/avatar";
  import AvatarFallback from "./components/ui/avatar/avatar-fallback.svelte";

  export let user: User;
</script>

<div
  class="hover:bg-primary-100 hover:dark:bg-primary-900 flex items-center gap-2 p-2 text-left"
>
  <Avatar class="rounded">
    <AvatarImage src={user.picture} alt={user.email} />
    <AvatarFallback class="rounded">
      <UserRound />
    </AvatarFallback>
  </Avatar>
  <div class="flex flex-col">
    <div class="text-sm">{user.name}</div>
    <div class="text-sm font-medium">{user.email}</div>
  </div>
</div>
