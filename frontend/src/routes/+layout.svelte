<script lang="ts">
  import { dev } from "$app/environment";
  import { updated } from "$app/stores";
  import { AuthContext, setAuthContext } from "$lib/auth.svelte";
  import { CommandPalette } from "$lib/components/ui/command-palette";
  import { Confetti } from "$lib/components/ui/confetti";
  import { Prefs } from "$lib/components/ui/prefs";
  import { toast, Toaster } from "$lib/components/ui/sonner";
  import { Dialoguer } from "$lib/kosui/dialog";
  import { ModeWatcher } from "mode-watcher";
  import { Workbox } from "workbox-window";
  import "../app.css";

  const { children } = $props();

  function register(): Workbox | null {
    if (dev) return null;
    const wb = new Workbox("/service-worker.js");
    wb.addEventListener("waiting", (event) => {
      console.debug("Got 'waiting' wb event", event);
      wb.messageSkipWaiting();
    });

    // This block added to debug koso hanging on reload
    wb.addEventListener("activated", (event) => {
      console.debug("Got 'activated' wb event", event);
    });
    wb.addEventListener("activating", (event) => {
      console.debug("Got 'activating' wb event", event);
    });
    wb.addEventListener("installed", (event) => {
      console.debug("Got 'installed' wb event", event);
    });
    wb.addEventListener("installing", (event) => {
      console.debug("Got 'installing' wb event", event);
    });
    wb.addEventListener("message", (event) => {
      console.debug("Got 'message' wb event", event);
    });
    wb.addEventListener("redundant", (event) => {
      console.debug("Got 'redundant' wb event", event);
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
      console.debug("Update effect triggered. Calling wb.update()");
      toast.info("New updates are available. Installing in the background...");
      wb.update();
    }
  });

  const ctx = setAuthContext(new AuthContext());

  $effect(() => {
    ctx.load();
  });
</script>

<ModeWatcher />
<Toaster richColors />
<Confetti />

<Dialoguer>
  <CommandPalette>
    <Prefs>
      {@render children()}
    </Prefs>
  </CommandPalette>
</Dialoguer>
