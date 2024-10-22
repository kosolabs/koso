<script lang="ts">
  import { dev } from "$app/environment";
  import { updated } from "$app/stores";
  import { Toaster } from "$lib/components/ui/sonner";
  import { ModeWatcher } from "mode-watcher";
  import { toast } from "svelte-sonner";
  import { Workbox } from "workbox-window";
  import "../app.css";

  const { children } = $props();

  const wb = new Workbox("/service-worker.js", {
    type: dev ? "module" : "classic",
  });
  wb.addEventListener("waiting", () => {
    wb.messageSkipWaiting();
  });
  wb.addEventListener("controlling", (event) => {
    console.debug("Reloading to activate new updates.", event);
    window.location.reload();
  });
  wb.register();

  $effect(() => {
    if ($updated) {
      toast.info("New updates are available. Installing in the background...");
      wb.update();
    }
  });
</script>

<ModeWatcher />
<Toaster />
{@render children()}
