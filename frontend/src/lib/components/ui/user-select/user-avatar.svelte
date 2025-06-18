<script lang="ts">
  import { Avatar } from "$lib/kosui/avatar";
  import type { ClassName } from "$lib/kosui/utils";
  import { UserRound } from "@lucide/svelte";
  import { twMerge } from "tailwind-merge";

  type Props = {
    user: { email: string; name: string; picture: string };
  } & ClassName;
  const { user, class: className }: Props = $props();
</script>

<div class={twMerge("flex items-center gap-2 text-left", className)}>
  <Avatar src={user.picture} alt={user.email}>
    <UserRound />
  </Avatar>
  <div class="flex flex-col">
    <div class="text-sm">{user.name}</div>
    <div class="text-sm font-medium">{user.email}</div>
  </div>
</div>
