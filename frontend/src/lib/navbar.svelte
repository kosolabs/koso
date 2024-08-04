<script lang="ts">
  import kosoLogo from "$lib/assets/koso.svg";
  import { logout, user } from "$lib/auth";
  import {
    Avatar,
    Button,
    Dropdown,
    DropdownHeader,
    DropdownItem,
    Navbar,
  } from "flowbite-svelte";
  import NavContainer from "flowbite-svelte/NavContainer.svelte";
  import User from "./user.svelte";
</script>

<Navbar color="primary" class="mb-4" fluid={true}>
  <NavContainer fluid={true}>
    <div class="flex items-center">
      <a href="/projects">
        <img class="w-14" alt="Koso Logo" src={kosoLogo} />
      </a>
      <slot name="left-items"></slot>
    </div>

    <div class="flex md:order-2">
      <slot name="right-items"></slot>

      {#if $user}
        <Button
          id="profile-menu"
          class="ms-3 rounded-full border bg-slate-200 p-2"
          title="Profile"
        >
          <div><Avatar src={$user.picture} size="xs" /></div>
        </Button>
        <Dropdown triggeredBy="#profile-menu">
          <DropdownHeader>
            <User user={$user} />
          </DropdownHeader>
          <DropdownItem href="/projects">Projects</DropdownItem>
          <DropdownItem on:click={() => logout()}>Logout</DropdownItem>
        </Dropdown>
      {/if}
    </div>
  </NavContainer>
</Navbar>
