<script lang="ts">
  import { goto } from "$app/navigation";
  import { headers } from "$lib/api";
  import { auth, type User } from "$lib/auth.svelte";
  import * as AlertDialog from "$lib/components/ui/alert-dialog";
  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Input } from "$lib/components/ui/input";
  import * as Popover from "$lib/components/ui/popover";
  import { UserAvatar } from "$lib/components/ui/user-select";
  import { logout_on_authentication_error } from "$lib/errors";
  import {
    COMPARE_USERS_BY_NAME_AND_EMAIL,
    updateProjectUsers,
    type Project,
  } from "$lib/projects";
  import { Shortcut } from "$lib/shortcuts";
  import { match } from "$lib/utils";
  import { CircleMinus, TriangleAlert } from "lucide-svelte";
  import { toast } from "svelte-sonner";
  import { flip } from "svelte/animate";

  type Props = {
    open: boolean;
    project: Project;
    projectUsers: User[];
  };
  let {
    open = $bindable(),
    project,
    projectUsers = $bindable(),
  }: Props = $props();

  let users: Promise<User[]> = $derived.by(async () => {
    const response = await fetch(`/api/users`, {
      headers: headers(),
    });
    if (!response.ok) {
      logout_on_authentication_error(response);
      throw new Error(
        `Failed to fetch all users: ${response.statusText} (${response.status})`,
      );
    }
    let users: User[] = await response.json();
    users.sort(COMPARE_USERS_BY_NAME_AND_EMAIL);
    return users;
  });

  async function addUser(add: User) {
    await updateProjectUsers({
      project_id: project.project_id,
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
    if (auth.user.email === remove.email && !forceRemoveSelf) {
      openWarnSelfRemovalModal = true;
      return;
    }

    let i = projectUsers.findIndex((u) => u.email === remove.email);
    if (i == -1) throw new Error("Could not find user");

    await updateProjectUsers({
      project_id: project.project_id,
      add_emails: [],
      remove_emails: [remove.email],
    });

    projectUsers.splice(i, 1);
    projectUsers.sort(COMPARE_USERS_BY_NAME_AND_EMAIL);
    projectUsers = projectUsers;

    toast.success(`Removed ${remove.email}`);
  }

  let filter: string = $state("");
  let openDropDown: boolean = $state(false);
  let openWarnSelfRemovalModal = $state(false);

  const MIN_FILTER_LEN = 2;
  let filteredUsers: Promise<User[]> = $derived.by(async () => {
    if (filter.length < MIN_FILTER_LEN) {
      return [];
    }
    const allUsers = await users;
    return allUsers
      .filter((u) => !projectUsers.some((pu) => pu.email === u.email))
      .filter((u) => match(u.name, filter) || match(u.email, filter));
  });
</script>

<Dialog.Root
  bind:open
  portal={null}
  onOpenChange={() => {
    filter = "";
    openDropDown = false;
  }}
>
  <Dialog.Content
    onkeydown={(event) => {
      event.stopPropagation();
      if (Shortcut.CANCEL.matches(event)) {
        open = false;
        filter = "";
        openDropDown = false;
      }
    }}
  >
    <Dialog.Header>
      <Dialog.Title>Share &quot;{project.name}&quot;</Dialog.Title>
      <Dialog.Description>Manage access to your project.</Dialog.Description>
    </Dialog.Header>
    <div class="flex flex-col gap-2">
      <Popover.Root
        disableFocusTrap={true}
        openFocus={false}
        bind:open={openDropDown}
      >
        <Popover.Trigger>
          <Input
            type="text"
            placeholder="Add people"
            name="Add people"
            bind:value={filter}
          />
        </Popover.Trigger>
        <Popover.Content sameWidth={true} class="max-h-96 overflow-y-auto">
          {#await filteredUsers then filteredUsers}
            {#if filter.length < MIN_FILTER_LEN}
              Search for people.
            {:else if filteredUsers.length > 0}
              {#each filteredUsers as user}
                <button
                  class="w-full cursor-pointer rounded p-2 hover:bg-accent"
                  title="Add {user.email}"
                  onclick={() => addUser(user)}
                >
                  <UserAvatar {user} />
                </button>
              {/each}
            {:else}
              <div>No people found.</div>
            {/if}
          {/await}
        </Popover.Content>
      </Popover.Root>

      <div class="h3 mt-2">People with access</div>
      <div
        class="flex h-64 w-full flex-col items-stretch overflow-y-auto [&>*:nth-child(even)]:bg-muted"
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
</Dialog.Root>

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
        onclick={async () => {
          await removeUser(auth.user, true);
          await goto("/projects");
        }}
      >
        Yes, I'm sure
      </AlertDialog.AlertDialogAction>
    </AlertDialog.AlertDialogFooter>
  </AlertDialog.AlertDialogContent>
</AlertDialog.Root>
