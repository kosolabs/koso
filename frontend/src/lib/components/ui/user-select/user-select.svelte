<script lang="ts">
  import type { User } from "$lib/auth.svelte";
  import { Avatar } from "$lib/kosui/avatar";
  import { Menu, MenuContent, MenuItem, MenuTrigger } from "$lib/kosui/menu";
  import { UserRound } from "lucide-svelte";
  import { UserAvatar } from ".";
  import ResponsiveText from "../responsive-text/responsive-text.svelte";

  type Props = {
    users: User[];
    value: User | null;
    unassigned?: string;
    editable?: boolean;
    onSelect?: (select: User | null) => void;
  };
  let {
    users,
    value = null,
    unassigned = "Unassigned",
    editable = true,
    onSelect,
  }: Props = $props();

  function select(user: User | null) {
    value = user;
    onSelect?.(user);
  }

  let open: boolean = $state(false);
</script>

{#if editable}
  <Menu bind:open>
    <MenuTrigger
      class="flex items-center gap-2"
      title={value?.email || "Unassigned"}
    >
      <Avatar class="size-6" src={value?.picture || ""} alt={value?.email}>
        <UserRound size={20} />
      </Avatar>
      <ResponsiveText>{value?.name || unassigned}</ResponsiveText>
    </MenuTrigger>
    <MenuContent>
      <MenuItem onSelect={() => select(null)}>
        <UserAvatar
          user={{ name: "Unassigned", email: "", picture: "", exp: 0 }}
        />
      </MenuItem>
      {#each users as user (user.email)}
        <MenuItem onSelect={() => select(user)}>
          <UserAvatar {user} />
        </MenuItem>
      {/each}</MenuContent
    >
  </Menu>
{:else}
  <div class="flex items-center gap-2" title={value?.email || "Unassigned"}>
    <Avatar class="size-6" src={value?.picture || ""} alt={value?.email}>
      <UserRound size={20} />
    </Avatar>
    <ResponsiveText>{value?.name || unassigned}</ResponsiveText>
  </div>
{/if}
