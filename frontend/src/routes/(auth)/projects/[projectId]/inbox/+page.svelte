<script lang="ts">
  import { getAuthContext } from "$lib/auth.svelte";
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

  const auth = getAuthContext();
  const project = getProjectContext();
  const { koso } = project;
  const prefs = getPrefsContext();
  const inbox = setInboxContext(new InboxContext(auth.user, koso));

  let detailPanel: DetailPanel | undefined = $state();
  let offline: boolean = $derived(project.socket.offline);
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

    <OfflineAlert {offline} />
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
