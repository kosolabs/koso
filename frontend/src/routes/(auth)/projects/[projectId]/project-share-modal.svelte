<script lang="ts">
  import { goto } from "$app/navigation";
  import { headers, parse_response } from "$lib/api";
  import { auth, type User } from "$lib/auth.svelte";
  import * as AlertDialog from "$lib/components/ui/alert-dialog";
  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Input } from "$lib/components/ui/input";
  import * as Popover from "$lib/components/ui/popover";
  import { UserAvatar } from "$lib/components/ui/user-select";
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

  async function addUser(add: User) {
    await updateProjectUsers({
      projectId: project.projectId,
      addEmails: [add.email],
      removeEmails: [],
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
      projectId: project.projectId,
      addEmails: [],
      removeEmails: [remove.email],
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
  let users: Promise<User[]> = $derived.by(async () => {
    if (filter.length < MIN_FILTER_LEN) {
      return [];
    }
    const response = await fetch(`/api/users?q=${filter}`, {
      headers: headers(),
    });
    let users: User[] = await parse_response(response);
    return users
      .sort(COMPARE_USERS_BY_NAME_AND_EMAIL)
      .filter((u) => !projectUsers.some((pu) => pu.email === u.email))
      .filter((u) => match(u.name, filter) || match(u.email, filter));
  });

  let searchInput = $state<HTMLElement | null>(null);

  $effect(() => {
    if (searchInput && filter) {
      openDropDown = true;
    }
  });
</script>

<Dialog.Root
  bind:open
  onOpenChange={() => {
    filter = "";
    openDropDown = false;
  }}
>
  <Dialog.Content
    portalProps={{ disabled: true }}
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
      <Popover.Root bind:open={openDropDown}>
        <Popover.Trigger bind:ref={searchInput}>
          {#snippet child({ props })}
            <Input
              {...props}
              type="text"
              placeholder="Add people"
              name="Add people"
              bind:value={filter}
              autocomplete="off"
            />
          {/snippet}
        </Popover.Trigger>
        <Popover.Content
          trapFocus={false}
          class="max-h-96 overflow-y-auto"
          onOpenAutoFocus={(e) => {
            e.preventDefault();
            searchInput?.focus();
          }}
          onCloseAutoFocus={(e) => {
            e.preventDefault();
          }}
        >
          {#await users then users}
            {#if filter.length < MIN_FILTER_LEN}
              Search for people.
            {:else if users.length > 0}
              {#each users as user}
                <button
                  class="hover:bg-accent w-full cursor-pointer rounded p-2"
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
        class="[&>*:nth-child(even)]:bg-muted flex h-64 w-full flex-col items-stretch overflow-y-auto"
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
  <AlertDialog.AlertDialogContent portalProps={{ disabled: true }}>
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
