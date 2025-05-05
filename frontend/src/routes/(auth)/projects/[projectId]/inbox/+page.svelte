<script lang="ts">
  import { command, type ActionID } from "$lib/components/ui/command-palette";
  import { DetailPanel } from "$lib/components/ui/detail-panel";
  import { Navbar } from "$lib/components/ui/navbar";
  import { Toolbar } from "$lib/components/ui/toolbar";
  import { TaskTable } from "$lib/dag-table";
  import { newInboxContext } from "$lib/dag-table/inbox-context.svelte";
  import OfflineAlert from "$lib/dag-table/offline-alert.svelte";
  import { getProjectContext } from "$lib/dag-table/project-context.svelte";
  import { Action } from "$lib/kosui/command";
  import { PanelTopClose, PanelTopOpen, SquarePen } from "lucide-svelte";
  import { onMount } from "svelte";

  const project = getProjectContext();
  const { koso } = project;
  const inbox = newInboxContext(koso);

  const actions: Action<ActionID>[] = [
    new Action({
      id: "DetailPanelClose",
      callback: () => (inbox.detailPanel = "none"),
      title: "Close task description",
      description: "Close / hide the task description markdown panel",
      icon: PanelTopClose,
      enabled: () => inbox.detailPanel !== "none",
    }),
    new Action({
      id: "DetailPanelOpen",
      callback: () => (inbox.detailPanel = "view"),
      title: "Open task description",
      description: "Open / show the task description markdown panel",
      icon: PanelTopOpen,
      enabled: () => inbox.detailPanel === "none",
    }),
    new Action({
      id: "DetailPanelViewer",
      callback: () => (inbox.detailPanel = "view"),
      title: "View task description",
      description: "Open / show the task description markdown viewer",
      icon: PanelTopOpen,
      enabled: () => !!inbox.selected,
    }),
    new Action({
      id: "DetailPanelEditor",
      callback: () => (inbox.detailPanel = "edit"),
      title: "Edit task description",
      description: "Open / show the task description markdown editor",
      icon: SquarePen,
      enabled: () => !!inbox.selected && koso.isEditable(inbox.selected.name),
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

  <div class="relative grow overflow-y-hidden p-2">
    <div class="flex h-full flex-col gap-2">
      {#if inbox.detailPanel !== "none"}
        <div class="flex-1 overflow-y-scroll">
          <DetailPanel
            taskId={inbox.selected?.id}
            detailPanelRenderer={inbox}
          />
        </div>
      {/if}
      <div class="flex-1 overflow-y-scroll">
        <TaskTable users={project.users} />
      </div>
    </div>
  </div>

  <div class="sm:hidden">
    <Toolbar
      actions={["Undo", "Redo", "DetailPanelClose", "DetailPanelOpen"]}
    />
  </div>
</div>
