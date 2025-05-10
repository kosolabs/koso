<script lang="ts">
  import { headers, parse_response } from "$lib/api";
  import { auth } from "$lib/auth.svelte";
  import { toast } from "$lib/components/ui/sonner";
  import { UserAvatar } from "$lib/components/ui/user-select";
  import { getProjectContext } from "$lib/dag-table";
  import {
    Autocomplete,
    AutocompleteContent,
    AutocompleteInput,
    AutocompleteItem,
  } from "$lib/kosui/autocomplete";
  import { Button } from "$lib/kosui/button";
  import { dialog } from "$lib/kosui/dialog";
  import { Modal } from "$lib/kosui/modal";
  import {
    COMPARE_USERS_BY_NAME_AND_EMAIL,
    updateProjectUsers,
  } from "$lib/projects";
  import type { User } from "$lib/users";
  import { cn, match } from "$lib/utils";
  import { CircleMinus, TriangleAlert, X } from "lucide-svelte";
  import { flip } from "svelte/animate";

  type Props = {
    open: boolean;
  };
  let { open = $bindable() }: Props = $props();
  const project = getProjectContext();

  let filter: string = $state("");
  let users: User[] = $state([]);
  let wantCompletions: boolean = $state(false);
  let showCompletions: boolean = $derived(wantCompletions && users.length > 0);

  async function addUser(add: User) {
    await updateProjectUsers({
      projectId: project.id,
      addEmails: [add.email],
      removeEmails: [],
    });

    project.users.push(add);
    project.users.sort(COMPARE_USERS_BY_NAME_AND_EMAIL);
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

    let i = project.users.findIndex((u) => u.email === remove.email);
    if (i == -1) throw new Error("Could not find user");

    await updateProjectUsers({
      projectId: project.id,
      addEmails: [],
      removeEmails: [remove.email],
    });

    project.users.splice(i, 1);
    project.users.sort(COMPARE_USERS_BY_NAME_AND_EMAIL);

    toast.success(`Removed ${remove.email}`);
  }

  const MIN_FILTER_LEN = 2;
  let req = 0;
  $effect(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    project.users;
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
        .filter((u) => !project.users.some((pu) => pu.email === u.email))
        .filter((u) => match(u.name, filter) || match(u.email, filter));
    })();
  });

  $effect(() => {
    if (filter) {
      wantCompletions = true;
    }
  });
</script>

<Modal bind:open class={cn("w-[min(calc(100%-1em),36em)]")}>
  <div class="flex flex-col gap-2">
    <div>
      <div class="flex items-center">
        <div class="text-xl">Share &quot;{project.name}&quot;</div>
        <Button
          variant="plain"
          class="ml-auto"
          aria-label="close"
          onclick={() => (open = false)}
        >
          <X />
        </Button>
      </div>
      <div class="text-sm">Manage access to your project.</div>
    </div>
    <Autocomplete>
      <AutocompleteInput
        bind:value={filter}
        autofocus
        class="my-2"
        placeholder="Add people"
        name="Add people"
        onclick={() => (wantCompletions = true)}
        onfocus={() => (wantCompletions = true)}
      />
      <AutocompleteContent
        open={showCompletions}
        ontoggle={(event) => {
          if (event.newState === "closed") {
            wantCompletions = false;
          }
        }}
        class="w-[min(calc(100%-1em),32em)] max-w-full"
      >
        {#each users as user (user.email)}
          <AutocompleteItem
            onSelect={() => {
              wantCompletions = false;
              addUser(user);
            }}
          >
            <UserAvatar {user} />
          </AutocompleteItem>
        {/each}
      </AutocompleteContent>
    </Autocomplete>

    <div class="h3">People with access</div>
    <div class="flex h-64 w-full flex-col items-stretch overflow-y-auto">
      {#each project.users as projectUser (projectUser.email)}
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
