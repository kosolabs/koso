<script lang="ts">
  import kosoLogo from "$lib/assets/koso.svg";
  import { logout, user } from "$lib/auth";
  import { Dropdown, DropdownHeader, DropdownItem } from "flowbite-svelte";
  import { Avatar } from "./components/ui/avatar";
  import AvatarImage from "./components/ui/avatar/avatar-image.svelte";
  import UserAvatar from "./user-avatar.svelte";
</script>

<nav class="mb-2 flex items-center">
  <div class="flex items-center">
    <a href="/projects">
      <img class="w-14" alt="Koso Logo" src={kosoLogo} />
    </a>
    <slot name="left-items"></slot>
  </div>

  <div class="ml-auto flex items-center gap-2">
    <slot name="right-items"></slot>

    {#if $user}
      <button id="profile-menu">
        <Avatar
          class="rounded transition-all hover:brightness-110 active:scale-95"
        >
          <AvatarImage src={$user.picture}></AvatarImage>
        </Avatar>
      </button>
      <Dropdown triggeredBy="#profile-menu">
        <DropdownHeader>
          <UserAvatar user={$user} />
        </DropdownHeader>
        <DropdownItem href="/projects">Projects</DropdownItem>
        <DropdownItem on:click={() => logout()}>Logout</DropdownItem>
      </Dropdown>
    {/if}
  </div>
</nav>
