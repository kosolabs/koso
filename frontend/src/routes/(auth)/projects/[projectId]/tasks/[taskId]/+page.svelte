<script lang="ts">
  import { Navbar } from "$lib/components/ui/navbar";
  import OfflineAlert from "$lib/dag-table/offline-alert.svelte";
  import { getProjectContext } from "$lib/dag-table/project-context.svelte";

  const project = getProjectContext();
</script>

<Navbar />

<OfflineAlert offline={project.socket.offline} />
