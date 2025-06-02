<script module lang="ts">
  import { goto } from "$app/navigation";
  import { Action, Commander, Registry } from "$lib/kosui/command";
  import { Shortcut } from "$lib/kosui/shortcut";
  import { Book, Moon, Sun, SunMoon, Terminal } from "lucide-svelte";
  import { resetMode, setMode } from "mode-watcher";
  import type { Snippet } from "svelte";
  import { getContext, onMount, setContext } from "svelte";

  export const Categories = {
    Edit: "Edit",
    Graph: "Graph",
    Navigate: "Navigate",
    MarkdownPanel: "Markdown Panel",
    Project: "Project",
    Select: "Select",
    Storybook: "Storybook",
    Task: "Task",
    Theme: "Theme",
    Tools: "Tools",
    View: "View",
  };

  export const ActionIds = {
    Block: "Block",
    Clear: "Clear",
    Collapse: "Collapse",
    CollapseAll: "CollapseAll",
    CopyTaskInfo: "CopyTaskInfo",
    CopyTaskLink: "CopyTaskLink",
    CommandPalette: "CommandPalette",
    ConnectToGitHub: "ConnectToGitHub",
    DarkTheme: "DarkTheme",
    Delete: "Delete",
    DetailPanelClose: "DetailPanelClose",
    DetailPanelOpen: "DetailPanelOpen",
    DetailPanelViewer: "DetailPanelViewer",
    DetailPanelEditor: "DetailPanelEditor",
    Edit: "Edit",
    Expand: "Expand",
    ExpandAll: "ExpandAll",
    ExportProject: "ExportProject",
    HideDoneTasks: "HideDoneTasks",
    InboxView: "InboxView",
    Indent: "Indent",
    Insert: "Insert",
    InsertAbove: "InsertAbove",
    InsertSubtask: "InsertSubtask",
    InsertSubtaskAbove: "InsertSubtaskAbove",
    LightTheme: "LightTheme",
    Link: "Link",
    MoveDown: "MoveDown",
    MoveToEnd: "MoveToEnd",
    MoveToStart: "MoveToStart",
    MoveUp: "MoveUp",
    Next: "Next",
    NextLink: "NextLink",
    Organize: "Organize",
    PlanView: "PlanView",
    Previous: "Previous",
    PreviousLink: "PreviousLink",
    ProjectsView: "ProjectsView",
    Redo: "Redo",
    Search: "Search",
    ShareProject: "ShareProject",
    ShowDoneTasks: "ShowDoneTasks",
    Storybook: "Storybook",
    SystemTheme: "SystemTheme",
    ToggleTaskStatus: "ToggleTaskStatus",
    Undent: "Undent",
    Undo: "Undo",
  };

  export function setRegistryContext(ctx: Registry): Registry {
    return setContext<Registry>(Registry, ctx);
  }

  export function getRegistryContext(): Registry {
    const ctx = getContext<Registry>(Registry);
    if (!ctx) throw new Error("RegistryContext is undefined");
    return ctx;
  }
</script>

<script lang="ts">
  type Props = {
    children: Snippet;
  };
  let { children }: Props = $props();

  const command = setRegistryContext(new Registry());

  let paletteOpen = $state(false);

  const actions: Action[] = [
    new Action({
      id: ActionIds.CommandPalette,
      callback: () => (paletteOpen = !paletteOpen),
      category: Categories.Tools,
      name: "Command Palette",
      description: "Show the command palette",
      icon: Terminal,
      enabled: () => true,
      shortcut: new Shortcut({ key: "p", shift: true, meta: true }),
    }),
    new Action({
      id: ActionIds.LightTheme,
      callback: () => setMode("light"),
      category: Categories.Theme,
      name: "Light",
      description: "Set the theme to light mode",
      icon: Sun,
    }),
    new Action({
      id: ActionIds.DarkTheme,
      callback: () => setMode("dark"),
      category: Categories.Theme,
      name: "Dark",
      description: "Set the theme to dark mode",
      icon: Moon,
    }),
    new Action({
      id: ActionIds.SystemTheme,
      callback: () => resetMode(),
      category: Categories.Theme,
      name: "System",
      description: "Set the theme to system",
      icon: SunMoon,
    }),
    new Action({
      id: ActionIds.Storybook,
      callback: () => goto("/storybook"),
      category: Categories.Navigate,
      name: "Storybook",
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
