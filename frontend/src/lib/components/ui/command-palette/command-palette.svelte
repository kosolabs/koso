<script module lang="ts">
  import { goto } from "$app/navigation";
  import { Action, Commander, Registry } from "$lib/kosui/command";
  import { Shortcut } from "$lib/kosui/shortcut";
  import { Book, Moon, Sun, SunMoon, Terminal } from "lucide-svelte";
  import { resetMode, setMode } from "mode-watcher";
  import type { Snippet } from "svelte";
  import { getContext, onMount, setContext } from "svelte";

  export type ActionID =
    | "Block"
    | "Clear"
    | "Collapse"
    | "CollapseAll"
    | "CopyTaskInfo"
    | "CopyTaskLink"
    | "CommandPalette"
    | "ConnectToGitHub"
    | "DarkTheme"
    | "Delete"
    | "DetailPanelClose"
    | "DetailPanelOpen"
    | "DetailPanelViewer"
    | "DetailPanelEditor"
    | "Edit"
    | "Expand"
    | "ExpandAll"
    | "ExportProject"
    | "HideDoneTasks"
    | "InboxView"
    | "Indent"
    | "Insert"
    | "InsertAbove"
    | "InsertSubtask"
    | "InsertSubtaskAbove"
    | "LightTheme"
    | "Link"
    | "MoveDown"
    | "MoveToEnd"
    | "MoveToStart"
    | "MoveUp"
    | "Next"
    | "NextLink"
    | "Organize"
    | "PlanView"
    | "Previous"
    | "PreviousLink"
    | "ProjectsView"
    | "Redo"
    | "Search"
    | "ShareProject"
    | "ShowDoneTasks"
    | "Storybook"
    | "StorybookAlerts"
    | "StorybookAutocomplete"
    | "StorybookAvatar"
    | "StorybookBadge"
    | "StorybookButtons"
    | "StorybookChips"
    | "StorybookCodeMirror"
    | "StorybookCommand"
    | "StorybookDialogs"
    | "StorybookFab"
    | "StorybookGoto"
    | "StorybookInputs"
    | "StorybookLinks"
    | "StorybookMarkdown"
    | "StorybookMenus"
    | "StorybookProgressIndicators"
    | "StorybookShortcuts"
    | "StorybookToggles"
    | "StorybookTooltips"
    | "SystemTheme"
    | "ToggleTaskStatus"
    | "Undent"
    | "Undo";

  export function setRegistryContext(
    ctx: Registry<ActionID>,
  ): Registry<ActionID> {
    return setContext<Registry<ActionID>>(Registry<ActionID>, ctx);
  }

  export function getRegistryContext(): Registry<ActionID> {
    const ctx = getContext<Registry<ActionID>>(Registry<ActionID>);
    if (!ctx) throw new Error("RegistryContext is undefined");
    return ctx;
  }
</script>

<script lang="ts">
  type Props = {
    children: Snippet;
  };
  let { children }: Props = $props();

  const command = setRegistryContext(new Registry<ActionID>());

  let paletteOpen = $state(false);

  const actions: Action<ActionID>[] = [
    new Action({
      id: "CommandPalette",
      callback: () => (paletteOpen = !paletteOpen),
      title: "Command palette",
      description: "Show the command palette",
      icon: Terminal,
      enabled: () => true,
      shortcut: new Shortcut({ key: "p", shift: true, meta: true }),
    }),
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

  onMount(() => {
    return command.register(...actions);
  });
</script>

<Commander bind:open={paletteOpen} {command} />

{@render children()}
