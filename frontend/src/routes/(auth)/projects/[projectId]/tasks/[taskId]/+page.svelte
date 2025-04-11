<script lang="ts">
  import { showUnauthorizedDialog } from "$lib/auth.svelte";
  import { Navbar } from "$lib/components/ui/navbar";
  import OfflineAlert from "$lib/dag-table/offline-alert.svelte";
  import { getProjectContext } from "$lib/dag-table/project-context.svelte";

  const project = getProjectContext();

  $effect(() => {
    if (project.socket.unauthorized) {
      showUnauthorizedDialog();
    }
  });
</script>

<Navbar />

<OfflineAlert offline={project.socket.offline} />
