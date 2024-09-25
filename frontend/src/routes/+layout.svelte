<script lang="ts">
  import { Toaster } from "$lib/components/ui/sonner";
  import { ModeWatcher } from "mode-watcher";
  import "../app.css";
  import { updated } from "$app/stores";
  import { version } from "$app/environment";

  $: if ($updated) {
    console.log(
      `Reloading page following version update Prior version was ${version}`,
    );
    window.location.reload();
  }
</script>

<ModeWatcher />
<Toaster />
<slot />
