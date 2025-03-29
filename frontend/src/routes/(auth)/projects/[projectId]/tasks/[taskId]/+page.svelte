<script lang="ts">
  import { page } from "$app/state";
  import { showUnauthorizedDialog } from "$lib/auth.svelte";
  import { Navbar } from "$lib/components/ui/navbar";
  import { Koso, KosoSocket } from "$lib/dag-table";
  import OfflineAlert from "$lib/dag-table/offline-alert.svelte";
  import * as Y from "yjs";

  const projectId = page.params.projectId;

  const koso = new Koso(projectId, new Y.Doc());
  const kosoSocket = new KosoSocket(koso, projectId);
  window.koso = koso;
  window.Y = Y;

  $effect(() => {
    if (kosoSocket.unauthorized) {
      showUnauthorizedDialog();
    }
  });
</script>

<Navbar />

<OfflineAlert offline={kosoSocket.offline} />
