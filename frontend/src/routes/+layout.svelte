<script lang="ts">
  import { dev } from "$app/environment";
  import { updated } from "$app/state";
  import { AuthContext, setAuthContext } from "$lib/auth.svelte";
  import { CommandPalette } from "$lib/components/ui/command-palette";
  import { Confetti } from "$lib/components/ui/confetti";
  import { Prefs } from "$lib/components/ui/prefs";
  import { toast, Toaster } from "$lib/components/ui/sonner";
  import { Dialoguer } from "kosui";
  import { ModeWatcher } from "mode-watcher";
  import { untrack } from "svelte";
  import { Workbox } from "workbox-window";
  import "../app.css";

  const { children } = $props();

  function register(): Workbox | null {
    if (dev) return null;
    const wb = new Workbox("/service-worker.js");
    wb.addEventListener("waiting", (event) => {
      console.debug("Sending SKIP_WAITING message to service worker", event);
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
    if (updated.current) {
      // Untrack everything else. This effect should only run when
      // updated.current changes.
      untrack(() => {
        if (wb) {
          console.debug("New version available. Calling wb.update()");
          untrack(() =>
            toast.info(
              "New updates are available. Installing in the background...",
            ),
          );
          wb.update();
        }
      });
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
