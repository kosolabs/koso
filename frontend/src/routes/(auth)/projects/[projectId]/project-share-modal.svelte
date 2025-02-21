<script lang="ts">
  import { headers, parse_response } from "$lib/api";
  import { auth, type User } from "$lib/auth.svelte";
  import { toast } from "$lib/components/ui/sonner";
  import { UserAvatar } from "$lib/components/ui/user-select";
  import { Button } from "$lib/kosui/button";
  import { dialog } from "$lib/kosui/dialog";
  import { Input } from "$lib/kosui/input";
  import { Menu, MenuItem } from "$lib/kosui/menu";
  import { Modal } from "$lib/kosui/modal";
  import {
    COMPARE_USERS_BY_NAME_AND_EMAIL,
    updateProjectUsers,
    type Project,
  } from "$lib/projects";
  import { cn, match } from "$lib/utils";
  import { CircleMinus, TriangleAlert, X } from "lucide-svelte";
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

  let filter: string = $state("");
  let users: User[] = $state([]);
  let wantSearchResultsOpen: boolean = $state(false);
  let openDropDown: boolean = $derived(
    wantSearchResultsOpen && users.length > 0,
  );

  async function addUser(add: User) {
    await updateProjectUsers({
      projectId: project.projectId,
      addEmails: [add.email],
      removeEmails: [],
    });

    projectUsers.push(add);
    projectUsers.sort(COMPARE_USERS_BY_NAME_AND_EMAIL);
    filter = "";

    toast.success(`Added ${add.email}`);
  }

  async function removeUser(remove: User) {
    if (auth.user.email === remove.email) {
      const confirmed = await dialog.confirm({
        icon: TriangleAlert,
        title: "Remove your own access?",
        message:
          "You will immediately lose access to this project. To regain access, you will need to contact another owner to have them grant you access.",
        acceptText: "Remove my access",
      });
      if (!confirmed) return;
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

    toast.success(`Removed ${remove.email}`);
  }

  const MIN_FILTER_LEN = 2;
  let req = 0;
  $effect(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    projectUsers;
    // reference project users so svelte treats it as a dependency.
    if (filter.trim().length < MIN_FILTER_LEN) {
      users = [];
      return;
    }

    (async () => {
      const thisReq = req + 1;
      req = thisReq;
      const response = await fetch(`/api/users?q=${filter}`, {
        headers: headers(),
      });
      let respUsers: User[] = await parse_response(response);
      if (thisReq !== req) {
        console.log(
          `Discarding request ${thisReq}. A newer request, ${req}, is running.`,
        );
        return;
      }
      users = respUsers
        .sort(COMPARE_USERS_BY_NAME_AND_EMAIL)
        .filter((u) => !projectUsers.some((pu) => pu.email === u.email))
        .filter((u) => match(u.name, filter) || match(u.email, filter));
    })();
  });

  $effect(() => {
    if (filter) {
      wantSearchResultsOpen = true;
    }
  });

  let searchInput: HTMLElement | undefined = $state();
</script>

<Modal bind:open class={cn("w-[min(calc(100%-1em),36em)]")} enableEscapeHandler>
  <div class="flex flex-col gap-2">
    <div>
      <div class="flex items-center">
        <div class={"text-xl"}>Share &quot;{project.name}&quot;</div>
        <Button
          variant="plain"
          class="ml-auto"
          aria-label="close"
          onclick={() => (open = false)}
        >
          <X />
        </Button>
      </div>
      <div class={"text-sm"}>Manage access to your project.</div>
    </div>
    <Input
      bind:value={filter}
      bind:ref={searchInput}
      autofocus
      type="text"
      placeholder="Add people"
      name="Add people"
      autocomplete="off"
      class="my-2"
      onclick={() => (wantSearchResultsOpen = true)}
      onfocus={() => (wantSearchResultsOpen = true)}
    />

    <Menu
      open={openDropDown}
      anchorEl={searchInput}
      ontoggle={(event) => {
        if (event.newState === "closed") {
          wantSearchResultsOpen = false;
        }
      }}
      class="w-[min(calc(100%-1em),32em)] max-w-full"
    >
      {#each users as user (user.email)}
        <MenuItem
          onclick={() => {
            wantSearchResultsOpen = false;
            addUser(user);
          }}
        >
          <UserAvatar {user} />
        </MenuItem>
      {/each}
    </Menu>

    <div class="h3">People with access</div>
    <div class="flex h-64 w-full flex-col items-stretch overflow-y-auto">
      {#each projectUsers as projectUser (projectUser.email)}
        <div
          class="flex items-center rounded p-2"
          animate:flip={{ duration: 250 }}
        >
          <UserAvatar user={projectUser} />
          <Button
            variant="plain"
            class="ml-auto"
            tooltip="Remove {projectUser.email}"
            aria-label="Remove {projectUser.email}"
            onclick={() => removeUser(projectUser)}
          >
            <CircleMinus />
          </Button>
        </div>
      {/each}
    </div>
  </div>
</Modal>
