<script lang="ts">
  import { dev } from "$app/environment";
  import { goto } from "$app/navigation";
  import { updated } from "$app/stores";
  import { newAuthContext } from "$lib/auth.svelte";
  import {
    command,
    CommandPalette,
    type ActionID,
  } from "$lib/components/ui/command-palette";
  import { Confetti } from "$lib/components/ui/confetti";
  import { Prefs } from "$lib/components/ui/prefs";
  import { toast, Toaster } from "$lib/components/ui/sonner";
  import { Action } from "$lib/kosui/command";
  import { Dialoguer } from "$lib/kosui/dialog";
  import { Book, Moon, Sun, SunMoon } from "lucide-svelte";
  import { ModeWatcher, resetMode, setMode } from "mode-watcher";
  import { onMount } from "svelte";
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

  const actions: Action<ActionID>[] = [
    new Action({
      id: "LightTheme",
      callback: () => setMode("light"),
      title: "Light",
      description: "Set the theme to light mode",
      icon: Sun,
    }),
    new Action({
      id: "DarkTheme",
      callback: () => setMode("dark"),
      title: "Dark",
      description: "Set the theme to dark mode",
      icon: Moon,
    }),
    new Action({
      id: "SystemTheme",
      callback: () => resetMode(),
      title: "System",
      description: "Set the theme to system",
      icon: SunMoon,
    }),
    new Action({
      id: "Storybook",
      callback: () => goto("/storybook"),
      title: "Storybook",
      description: "Navigate to Koso's component library storybook",
      icon: Book,
    }),
  ];

  $effect(() => {
    if (wb && $updated) {
      toast.info("New updates are available. Installing in the background...");
      wb.update();
    }
  });

  onMount(() => {
    return command.register(...actions);
  });

  const ctx = newAuthContext();

  $effect(() => {
    ctx.load();
  });
</script>

<ModeWatcher />
<Toaster richColors />
<Dialoguer />
<CommandPalette />
<Confetti />
<Prefs>
  {@render children()}
</Prefs>
