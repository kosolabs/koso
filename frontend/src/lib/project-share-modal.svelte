<script lang="ts">
  import UserAvatar from "./user-avatar.svelte";
  import { goto } from "$app/navigation";
  import { token, user, type User } from "$lib/auth";
  import {
    COMPARE_USERS_BY_NAME_AND_EMAIL,
    updateProjectPermissions,
    type Project,
  } from "$lib/projects";
  import { A, Alert, Button, Dropdown, Input, Modal } from "flowbite-svelte";
  import {
    UserPlus,
    CircleMinus,
    TriangleAlert,
    CircleCheck,
  } from "lucide-svelte";
  import { fade } from "svelte/transition";
  import * as AlertDialog from "$lib/components/ui/alert-dialog";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Description } from "./components/ui/alert";

  export let open: boolean;
  export let project: Project | null;
  export let projectUsers: User[];

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
    users.sort(COMPARE_USERS_BY_NAME_AND_EMAIL);

    cachedAllUsers = users;
    return cachedAllUsers;
  }

  async function addUser(add: User) {
    if (!$user) throw new Error("User is unauthorized");

    await updateProjectPermissions($token, {
      project_id: project?.project_id || "",
      add_emails: [add.email],
      remove_emails: [],
    });

    projectUsers.push(add);
    projectUsers.sort(COMPARE_USERS_BY_NAME_AND_EMAIL);
    projectUsers = projectUsers;

    openDropDown = false;
    addedOrRemovedMessage = `Added ${add.email}`;
  }

  async function removeUser(remove: User, forceRemoveSelf: boolean) {
    if (!$user) throw new Error("User is unauthorized");

    if ($user.email === remove.email && !forceRemoveSelf) {
      openWarnSelfRemovalModal = true;
      return;
    }

    let i = projectUsers.findIndex((u) => u.email === remove.email);
    if (i == -1) throw new Error("Could not find user");

    await updateProjectPermissions($token, {
      project_id: project?.project_id || "",
      add_emails: [],
      remove_emails: [remove.email],
    });

    projectUsers.splice(i, 1);
    projectUsers.sort(COMPARE_USERS_BY_NAME_AND_EMAIL);
    projectUsers = projectUsers;

    addedOrRemovedMessage = `Removed ${remove.email}`;
  }

  let openDropDown: boolean = false;
  let nonProjectUsers: User[] = [];
  let filteredUsers: User[] = [];
  let filter: string = "";
  let openWarnSelfRemovalModal = false;
  let addedOrRemovedMessage: string | null = null;

  $: loadAllUsers().then(
    (allUsers) =>
      (nonProjectUsers = allUsers.filter(
        (u) => !projectUsers.some((pu) => pu.email === u.email),
      )),
  );
  $: filteredUsers = nonProjectUsers.filter(
    (user) =>
      user.name.toLowerCase().includes(filter.toLowerCase()) ||
      user.email.toLowerCase().includes(filter.toLowerCase()),
  );
</script>

<Modal autoclose outsideclose size="md"></Modal>

<Dialog.Root
  bind:open
  onOpenChange={(open) => {
    if (!open) {
      addedOrRemovedMessage = null;
      filter = "";
    }
  }}
>
  <Dialog.Content>
    <Dialog.Header>
      <Dialog.Title>Share &quot;{project?.name || ""}&quot;</Dialog.Title>
      <Dialog.Description>Manage access to your project.</Dialog.Description>
    </Dialog.Header>
    <div class="flex flex-1 flex-col gap-2">
      {#if addedOrRemovedMessage}
        <div transition:fade={{ duration: 350 }}>
          <Alert color="green" class="flex items-center p-2 font-medium">
            <span><CircleCheck /></span>
            <span>{addedOrRemovedMessage}</span>
          </Alert>
        </div>
      {/if}
      <Input type="text" placeholder="Add people" bind:value={filter}>
        <UserPlus slot="left" class="h-4 w-4" />
      </Input>

      <Dropdown
        bind:open={openDropDown}
        class="max-h-96 overflow-y-auto"
        style="width: 39.5rem"
      >
        <div class="flex flex-col gap-2 p-2">
          {#if filteredUsers.length > 0}
            {#each filteredUsers as user}
              <button
                title="Add {user.email}"
                on:click={async () => {
                  await addUser(user);
                }}
              >
                <UserAvatar {user} />
              </button>
            {/each}
          {:else}
            <button disabled>No people found.</button>
          {/if}
        </div>
      </Dropdown>

      <div class="h3 mt-2">People with access</div>
      <div
        class="flex max-h-96 flex-col items-stretch overflow-y-auto [&>*:nth-child(even)]:bg-slate-50"
      >
        {#each projectUsers as projectUser}
          <div class="flex flex-row rounded border p-2">
            <A
              size="xs"
              class="b border-r-2 pr-2"
              title="Remove {projectUser.email}"
              on:click={async () => {
                await removeUser(projectUser, false);
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
  </Dialog.Content>
</Dialog.Root>

<AlertDialog.Root bind:open={openWarnSelfRemovalModal}>
  <AlertDialog.AlertDialogTrigger asChild>
    <Button variant="outline">Show Dialog</Button>
  </AlertDialog.AlertDialogTrigger>
  <AlertDialog.AlertDialogContent>
    <AlertDialog.AlertDialogHeader>
      <AlertDialog.AlertDialogTitle
        >Are you absolutely sure?</AlertDialog.AlertDialogTitle
      >
      <AlertDialog.AlertDialogDescription>
        <TriangleAlert
          class="mx-auto mb-4 h-12 w-12 text-gray-400 dark:text-gray-200"
        />
        You will <b>immediately lose access</b> if you remove yourself from this
        project.
      </AlertDialog.AlertDialogDescription>
    </AlertDialog.AlertDialogHeader>
    <AlertDialog.AlertDialogFooter>
      <AlertDialog.AlertDialogCancel>Cancel</AlertDialog.AlertDialogCancel>
      <AlertDialog.AlertDialogAction
        on:click={async () => {
          if (!$user) throw new Error("User is unauthorized");
          await removeUser($user, true);
          await goto("/projects");
        }}>Yes, I'm sure</AlertDialog.AlertDialogAction
      >
    </AlertDialog.AlertDialogFooter>
  </AlertDialog.AlertDialogContent>
</AlertDialog.Root>
