<script lang="ts">
  import type { User } from "$lib/auth.svelte";
  import { Avatar } from "$lib/kosui/avatar";
  import { UserRound } from "lucide-svelte";

  type Props = { user: User };
  const { user }: Props = $props();
</script>

<div class="flex items-center gap-2 text-left">
  <Avatar src={user.picture} alt={user.email}>
    <UserRound />
  </Avatar>
  <div class="flex flex-col">
    <div class="text-sm">{user.name}</div>
    <div class="text-sm font-medium">{user.email}</div>
  </div>
</div>
