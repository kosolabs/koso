<script lang="ts">
  import UserAvatar from "./user-avatar.svelte";
  import { goto } from "$app/navigation";
  import { token, user, type User } from "$lib/auth";
  import { updateProjectPermissions, type Project } from "$lib/projects";
  import { A, Alert, Button, Dropdown, Input, Modal } from "flowbite-svelte";
  import { UserPlus, CircleMinus, TriangleAlert } from "lucide-svelte";
  import { fade } from "svelte/transition";

  export let open: boolean;
  export let project: Project | null;
  export let projectUsers: User[];

  const COMPARE_USER_BY_NAME = (a: User, b: User) =>
    a.name < b.name ? -1 : a.name > b.name ? 1 : 0;

  let cachedAllUsers: User[] | null = null;
  export async function loadAllUsers(): Promise<User[]> {
    if (cachedAllUsers !== null) return cachedAllUsers;
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

    cachedAllUsers = users;
    return cachedAllUsers;
  }

  async function addUser(user: User) {
    if (!$user) throw new Error("User is unauthorized");

    await updateProjectPermissions($token, {
      project_id: project?.project_id || "",
      add_emails: [user.email],
      remove_emails: [],
    });

    projectUsers.push(user);
    projectUsers.sort(COMPARE_USER_BY_NAME);
    projectUsers = projectUsers;
  }

  async function removeUser(user: User, forceRemoveSelf: boolean) {
    if (!$user) throw new Error("User is unauthorized");

    if ($user.email === user.email && !forceRemoveSelf) {
      openWarnSelfRemovalModal = true;
      return;
    }

    await updateProjectPermissions($token, {
      project_id: project?.project_id || "",
      add_emails: [],
      remove_emails: [user.email],
    });

    let i = projectUsers.findIndex((u) => u.email === user.email);
    if (i == -1) throw new Error("Could not find user");

    projectUsers.splice(i, 1);
    projectUsers.sort(COMPARE_USER_BY_NAME);
    projectUsers = projectUsers;
  }

  let openDropDown: boolean = false;
  let nonProjectUsers: User[] = [];
  let filteredUsers: User[] = [];
  let filter: string = "";
  let openWarnSelfRemovalModal = false;
  let addedOrRemovedMessage: string | null = null;

  $: loadAllUsers().then(
    (allUsers) =>
      (filteredUsers = allUsers.filter(
        (u) => !projectUsers.some((pu) => pu.email === u.email),
      )),
  );
  $: filteredUsers = nonProjectUsers.filter(
    (user) =>
      user.name.toLowerCase().includes(filter.toLowerCase()) ||
      user.email.toLowerCase().includes(filter.toLowerCase()),
  );
</script>

<!-- TODO: Figure out how to keep focus on the modal after adding a user so ESC works. -->
<!-- TODO: Removing users cause tasks to display as unassigned, but adding them back reassigns. -->
<!-- TODO: Set focus on filter input when the modal opens. -->
<Modal
  title="Share &quot;{project?.name || ''}&quot;"
  bind:open
  autoclose
  outsideclose
  size="xs"
  class="max-h-96"
  on:close={() => {
    addedOrRemovedMessage = null;
    filter = "";
  }}
>
  <div class="flex flex-1 flex-col gap-2">
    {#if addedOrRemovedMessage}
      <div transition:fade={{ duration: 350 }}>
        <Alert color="green">
          <span class="font-medium">{addedOrRemovedMessage}</span>
        </Alert>
      </div>
    {/if}
    <Input type="text" placeholder="Add a user" bind:value={filter}>
      <UserPlus slot="left" class="h-4 w-4" />
    </Input>

    {#await loadAllUsers() then allUsers}
      <Dropdown
        bind:openDropDown
        class="max-h-72 w-96 overflow-y-auto"
        on:select={async (event) => {}}
      >
        <div class="flex flex-col gap-2 p-2">
          {#each filteredUsers as user}
            <button
              on:click={async () => {
                await addUser(user);
                addedOrRemovedMessage = `Added ${user.email}`;
                openDropDown = false;
              }}
            >
              <UserAvatar {user} />
            </button>
          {/each}
        </div>
      </Dropdown>
    {/await}

    <div class="h3 mt-2">People with access</div>
    <div class="flex flex-col items-stretch [&>*:nth-child(even)]:bg-slate-50">
      {#each projectUsers as projectUser, i}
        <div class="flex flex-row rounded border p-2">
          <A
            size="xs"
            class="b border-r-2 pr-2"
            title="Remove {projectUser.email}"
            on:click={async () => {
              await removeUser(projectUser, false);
              addedOrRemovedMessage = `Removed ${projectUser.email}`;
            }}
          >
            <CircleMinus />
          </A>
          <div>
            <UserAvatar user={projectUser} />
          </div>
        </div>
      {/each}
    </div>
  </div>
</Modal>

<Modal bind:open={openWarnSelfRemovalModal} size="xs" autoclose>
  <div class="text-center">
    <TriangleAlert
      class="mx-auto mb-4 h-12 w-12 text-gray-400 dark:text-gray-200"
    />
    <h3 class="mb-5 text-lg font-normal text-gray-500 dark:text-gray-400">
      Are you sure? You will <b>immediately lose access</b> if you remove yourself
      from this project.
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
