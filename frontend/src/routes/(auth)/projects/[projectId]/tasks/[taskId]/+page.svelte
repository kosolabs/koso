<script lang="ts">
  import { Navbar } from "$lib/components/ui/navbar";
  import OfflineAlert from "$lib/dag-table/offline-alert.svelte";
  import { getProjectContext } from "$lib/dag-table/project-context.svelte";

  const project = getProjectContext();
  let offline: boolean = $derived(project.socket.offline);
</script>

<Navbar />

<OfflineAlert {offline} />
