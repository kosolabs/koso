<script lang="ts">
  import { goto } from "$app/navigation";
  import { token, user, type User } from "$lib/auth";
  import * as AlertDialog from "$lib/components/ui/alert-dialog";
  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Input } from "$lib/components/ui/input";
  import * as Popover from "$lib/components/ui/popover";
  import { logout_on_authentication_error } from "$lib/errors";
  import { DialogMonitoredRoot } from "$lib/popover-monitors";
  import {
    COMPARE_USERS_BY_NAME_AND_EMAIL,
    updateProjectPermissions,
    type Project,
  } from "$lib/projects";
  import UserAvatar from "$lib/user-avatar.svelte";
  import { CircleMinus, TriangleAlert } from "lucide-svelte";
  import { toast } from "svelte-sonner";
  import { flip } from "svelte/animate";

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
      logout_on_authentication_error(response);
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
    toast.success(`Added ${add.email}`);
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

    toast.success(`Removed ${remove.email}`);
  }

  let openDropDown: boolean = false;
  let nonProjectUsers: User[] = [];
  let filteredUsers: User[] = [];
  let filter: string = "";
  let openWarnSelfRemovalModal = false;

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

<DialogMonitoredRoot
  bind:open
  onOpenChange={() => {
    filter = "";
  }}
>
  <Dialog.Content>
    <Dialog.Header>
      <Dialog.Title>Share &quot;{project?.name || ""}&quot;</Dialog.Title>
      <Dialog.Description>Manage access to your project.</Dialog.Description>
    </Dialog.Header>
    <div class="flex flex-col gap-2">
      <Popover.Root
        disableFocusTrap={true}
        openFocus={false}
        bind:open={openDropDown}
      >
        <Popover.Trigger>
          <Input type="text" placeholder="Add people" bind:value={filter} />
        </Popover.Trigger>
        <Popover.Content sameWidth={true}>
          {#if filteredUsers.length > 0}
            {#each filteredUsers as user}
              <button
                class="w-full cursor-pointer rounded p-2 hover:bg-accent"
                title="Add {user.email}"
                on:click={() => addUser(user)}
              >
                <UserAvatar {user} />
              </button>
            {/each}
          {:else}
            <div>No people found.</div>
          {/if}
        </Popover.Content>
      </Popover.Root>

      <div class="h3 mt-2">People with access</div>
      <div
        class="flex h-64 w-full flex-col items-stretch overflow-y-auto [&>*:nth-child(even)]:bg-row-even"
      >
        {#each projectUsers as projectUser (projectUser.email)}
          <div
            class="flex items-center rounded p-2"
            animate:flip={{ duration: 250 }}
          >
            <UserAvatar user={projectUser} />
            <Button
              class="ml-auto"
              variant="link"
              title="Remove {projectUser.email}"
              onclick={async () => {
                await removeUser(projectUser, false);
              }}
            >
              <CircleMinus />
            </Button>
          </div>
        {/each}
      </div>
    </div>
  </Dialog.Content>
</DialogMonitoredRoot>

<AlertDialog.Root bind:open={openWarnSelfRemovalModal}>
  <AlertDialog.AlertDialogContent>
    <AlertDialog.AlertDialogHeader>
      <AlertDialog.AlertDialogTitle>
        Are you absolutely sure?
      </AlertDialog.AlertDialogTitle>
      <AlertDialog.AlertDialogDescription>
        <TriangleAlert class="mx-auto mb-4 h-12 w-12 text-yellow-300" />
        You will <b>immediately lose access</b> if you remove yourself from this
        project.
      </AlertDialog.AlertDialogDescription>
    </AlertDialog.AlertDialogHeader>
    <AlertDialog.AlertDialogFooter>
      <AlertDialog.AlertDialogCancel>Cancel</AlertDialog.AlertDialogCancel>
      <AlertDialog.AlertDialogAction
        class="bg-destructive text-white"
        on:click={async () => {
          if (!$user) throw new Error("User is unauthorized");
          await removeUser($user, true);
          await goto("/projects");
        }}
      >
        Yes, I'm sure
      </AlertDialog.AlertDialogAction>
    </AlertDialog.AlertDialogFooter>
  </AlertDialog.AlertDialogContent>
</AlertDialog.Root>
