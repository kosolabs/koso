<script lang="ts">
  import type { User } from "$lib/auth";
  import {
    Avatar,
    AvatarFallback,
    AvatarImage,
  } from "$lib/components/ui/avatar";
  import { UserRound } from "lucide-svelte";

  type Props = { user: User };
  const { user }: Props = $props();
</script>

<div class="flex items-center gap-2 text-left">
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
