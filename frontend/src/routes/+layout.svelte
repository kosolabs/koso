<script lang="ts">
  import { dev } from "$app/environment";
  import { updated } from "$app/stores";
  import { Confetti } from "$lib/components/ui/confetti";
  import { toast, Toaster } from "$lib/components/ui/sonner";
  import { command, Commander } from "$lib/kosui/command";
  import { Dialoguer } from "$lib/kosui/dialog";
  import { Shortcut } from "$lib/kosui/shortcut";
  import { Action } from "$lib/shortcuts";
  import { Moon, Sun, SunMoon, Terminal } from "lucide-svelte";
  import { ModeWatcher, resetMode, setMode } from "mode-watcher";
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

  const actions: Action[] = [
    new Action({
      callback: command.show,
      title: "Palette",
      description: "Show the command palette",
      icon: Terminal,
      toolbar: true,
      shortcut: new Shortcut({ key: "p", shift: true, meta: true }),
    }),
    new Action({
      callback: () => setMode("light"),
      title: "Light",
      description: "Set the theme to light mode",
      icon: Sun,
    }),
    new Action({
      callback: () => setMode("dark"),
      title: "Dark",
      description: "Set the theme to dark mode",
      icon: Moon,
    }),
    new Action({
      callback: () => resetMode(),
      title: "System",
      description: "Set the theme to system",
      icon: SunMoon,
    }),
  ];

  $effect(() => {
    if (wb && $updated) {
      toast.info("New updates are available. Installing in the background...");
      wb.update();
    }
  });

  $effect(() => {
    for (const action of actions) {
      command.register(action);
    }

    return () => {
      for (const action of actions) {
        command.unregister(action);
      }
    };
  });
</script>

<ModeWatcher />
<Toaster richColors />
<Dialoguer />
<Commander />
<Confetti />
{@render children()}
