<script lang="ts">
  import { dev } from "$app/environment";
  import { updated } from "$app/stores";
  import { Confetti } from "$lib/components/ui/confetti";
  import { Toaster } from "$lib/components/ui/sonner";
  import { ModeWatcher } from "mode-watcher";
  import { toast } from "svelte-sonner";
  import { Workbox } from "workbox-window";
  import "../app.css";

  const { children } = $props();

  function register(): Workbox | null {
    if (dev) return null;
    const wb = new Workbox("/service-worker.js");
    wb.addEventListener("waiting", () => {
      wb.messageSkipWaiting();
    });
    wb.addEventListener("controlling", (event) => {
      console.debug("Reloading to activate new updates.", event);
      window.location.reload();
    });
    wb.register();
    return wb;
  }
  const wb = register();

  $effect(() => {
    if (wb && $updated) {
      toast.info("New updates are available. Installing in the background...");
      wb.update();
    }
  });
</script>

<ModeWatcher />
<Toaster />
<Confetti />
{@render children()}
