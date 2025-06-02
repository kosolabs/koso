<script lang="ts">
  import { getAuthContext } from "$lib/auth.svelte";
  import {
    ActionIds,
    Categories,
    getRegistryContext,
  } from "$lib/components/ui/command-palette";
  import { DetailPanel } from "$lib/components/ui/detail-panel";
  import { Navbar } from "$lib/components/ui/navbar";
  import { getPrefsContext } from "$lib/components/ui/prefs";
  import { Toolbar } from "$lib/components/ui/toolbar";
  import { getProjectContext, TaskTable } from "$lib/dag-table";
  import {
    InboxContext,
    setInboxContext,
  } from "$lib/dag-table/inbox-context.svelte";
  import OfflineAlert from "$lib/dag-table/offline-alert.svelte";
  import { Action } from "$lib/kosui/command";
  import { Shortcut } from "$lib/kosui/shortcut";
  import { Pencil } from "lucide-svelte";
  import { onMount, tick } from "svelte";

  const auth = getAuthContext();
  const project = getProjectContext();
  const { koso } = project;
  const prefs = getPrefsContext();
  const inbox = setInboxContext(new InboxContext(auth, koso));
  const command = getRegistryContext();

  let detailPanel: DetailPanel | undefined = $state();

  async function edit() {
    if (prefs.detailPanel === "none") {
      prefs.detailPanel = "view";
      await tick();
    }
    detailPanel?.editTitle();
  }

  const actions: Action[] = [
    new Action({
      id: ActionIds.Edit,
      callback: edit,
      category: Categories.Edit,
      name: "Task Name",
      description: "Edit the current task",
      icon: Pencil,
      shortcut: new Shortcut({ key: "Enter" }),
      enabled: () => !!inbox.selected && koso.isEditable(inbox.selected.id),
    }),
  ];

  onMount(() => {
    return command.register(...actions);
  });
</script>

<div class="flex h-dvh flex-col">
  <div class="grow-0">
    <Navbar>
      {#snippet left()}
        <div>
          <h1 class="ml-2 text-lg">
            Inbox - {project.name}
          </h1>
        </div>
      {/snippet}
    </Navbar>

    <OfflineAlert offline={project.socket.offline} />
  </div>

  <div class="grow overflow-y-hidden p-1">
    <div class="flex h-full flex-row-reverse max-2xl:flex-col">
      {#if prefs.detailPanel !== "none"}
        <div class="flex-1 overflow-y-scroll p-1">
          <DetailPanel bind:this={detailPanel} taskId={inbox.selected?.id} />
        </div>
      {/if}
      <div class="flex-2 overflow-y-scroll p-1">
        <TaskTable users={project.users} />
      </div>
    </div>
  </div>

  <div class="sm:hidden">
    <Toolbar selected={!!inbox.selected} />
  </div>
</div>
