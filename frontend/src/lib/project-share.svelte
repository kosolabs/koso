<script lang="ts">
  import kosoLogo from "$lib/assets/koso.svg";
  import {
    Avatar,
    Dropdown,
    DropdownHeader,
    DropdownItem,
    Navbar,
  } from "flowbite-svelte";
  import NavContainer from "flowbite-svelte/NavContainer.svelte";
  import UserAvatar from "./user-avatar.svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { token, user, type User } from "$lib/auth";
  import { DagTable } from "$lib/DagTable";
  import { Koso } from "$lib/koso";
  import { lastVisitedProjectId } from "$lib/nav";
  import {
    fetchProjects,
    type Project,
    updateProject,
    updateProjectPermissions,
    fetchProjectUsers,
  } from "$lib/projects";
  import UserSelect from "$lib/user-select.svelte";

  import { A, Alert, Button, Input, Label, Modal } from "flowbite-svelte";
  import { UserPlus, CircleMinus, TriangleAlert } from "lucide-svelte";
  import { createEventDispatcher, onDestroy, onMount } from "svelte";
  import * as Y from "yjs";

  export let open = false;
  export let projectId: string;
  export let projectUsers: User[];

  let emptyUser: User | null = null;
  let showWarnSelfRemovalModal = false;

  const COMPARE_USER_BY_NAME = (a: User, b: User) =>
    a.name < b.name ? -1 : a.name > b.name ? 1 : 0;

  let allUsers = loadAllUsers();

  export async function loadAllUsers(): Promise<User[]> {
    if (!$user || !$token) throw new Error("User is unauthorized");

    const response = await fetch(`/api/users`, {
      headers: { Authorization: "Bearer " + $token },
    });
    if (!response.ok) {
      throw new Error(
        `Failed to fetch all users: ${response.statusText} (${response.status})`,
      );
    }
    let users: User[] = await response.json();
    users.sort(COMPARE_USER_BY_NAME);
    return users;
  }

  async function addUser(user: User) {
    if (!$user) throw new Error("User is unauthorized");

    emptyUser = null;

    await updateProjectPermissions($token, {
      project_id: projectId,
      add_emails: [user.email],
      remove_emails: [],
    });
    projectUsers.push(user);
    projectUsers.sort(COMPARE_USER_BY_NAME);
    projectUsers = projectUsers;
  }

  async function removeUser(user: User, forceRemoveSelf: boolean) {
    if (!$user) throw new Error("User is unauthorized");

    if ($user.email === user.email) {
      showWarnSelfRemovalModal = true;
      return;
    }

    await updateProjectPermissions($token, {
      project_id: projectId,
      add_emails: [],
      remove_emails: [user.email],
    });

    let i = projectUsers.findIndex((u) => u.email === user.email);
    if (i == -1) throw new Error("Could not find user");
    projectUsers.splice(i, 1);

    projectUsers.sort(COMPARE_USER_BY_NAME);
    projectUsers = projectUsers;
  }
</script>

<Modal title="Share your project" bind:open autoclose outsideclose>
  <UserSelect
    users={(await allUsers).filter(
      (u) => !projectUsers.some((pu) => pu.email === u.email),
    )}
    defaultLabel="Select a user"
    showUnassigned={false}
    value={emptyUser}
    on:select={async (event) => {
      emptyUser = null;
      if (!event.detail) {
        return;
      }
      await addUser(event.detail);
    }}
  />

  <div class="flex flex-col items-stretch [&>*:nth-child(even)]:bg-slate-50">
    {#each projectUsers as projectUser, i}
      <div class="flex flex-row rounded border p-2">
        <A
          size="xs"
          class="b"
          title="Remove user {projectUser.email}"
          on:click={async () => {
            await removeUser(projectUser, false);
          }}
        >
          <CircleMinus />
        </A>
        <div class="ml-4">
          <UserAvatar user={projectUser} />
        </div>
      </div>
    {/each}
  </div>
</Modal>

<Modal bind:open={showWarnSelfRemovalModal} size="xs" autoclose>
  <div class="text-center">
    <TriangleAlert
      class="mx-auto mb-4 h-12 w-12 text-gray-400 dark:text-gray-200"
    />
    <h3 class="mb-5 text-lg font-normal text-gray-500 dark:text-gray-400">
      Are you sure? You will immediately lose access if you remove yourself from
      this project.
    </h3>
    <Button
      color="red"
      class="me-2"
      on:click={async () => {
        if (!$user) throw new Error("User is unauthorized");
        await removeUser($user, true);
        await goto("/projects");
      }}>Yes, I'm sure</Button
    >
    <Button color="alternative">No, cancel</Button>
  </div>
</Modal>
