<script module lang="ts">
  import { NavigationAction } from "$lib/navigation-action";
  import { House, Moon, Sun, SunMoon, Terminal } from "@lucide/svelte";
  import { Action, Commander, Registry, Shortcut } from "kosui";
  import { userPrefersMode as mode, resetMode, setMode } from "mode-watcher";
  import type { Snippet } from "svelte";
  import { getContext, onMount, setContext } from "svelte";

  export const Categories = {
    Account: "Account",
    Agent: "Agent",
    Edit: "Edit",
    Graph: "Graph",
    Navigation: "Navigation",
    MarkdownPanel: "Markdown Panel",
    Project: "Project",
    Select: "Select",
    Task: "Task",
    Theme: "Theme",
    Tools: "Tools",
    View: "View",
  };

  export const ActionIds = {
    Archive: "Archive",
    Block: "Block",
    BreakDown: "BreakDown",
    Clear: "Clear",
    Collapse: "Collapse",
    CollapseAll: "CollapseAll",
    CopyTaskInfo: "CopyTaskInfo",
    CopyTaskLink: "CopyTaskLink",
    CommandPalette: "CommandPalette",
    ConnectToGitHub: "ConnectToGitHub",
    DarkTheme: "DarkTheme",
    DashView: "DashView",
    Delete: "Delete",
    DetailPanelClose: "DetailPanelClose",
    DetailPanelOpen: "DetailPanelOpen",
    DetailPanelViewer: "DetailPanelViewer",
    DetailPanelEditor: "DetailPanelEditor",
    Edit: "Edit",
    Expand: "Expand",
    ExpandAll: "ExpandAll",
    ExportProject: "ExportProject",
    GenerateDesignDoc: "GenerateDesignDoc",
    HideArchivedTasks: "HideArchivedTasks",
    Home: "Home",
    InboxView: "InboxView",
    Indent: "Indent",
    Insert: "Insert",
    InsertAbove: "InsertAbove",
    InsertSubtask: "InsertSubtask",
    InsertSubtaskAbove: "InsertSubtaskAbove",
    LightTheme: "LightTheme",
    Link: "Link",
    Logout: "Logout",
    MoveDown: "MoveDown",
    MoveToEnd: "MoveToEnd",
    MoveToStart: "MoveToStart",
    MoveUp: "MoveUp",
    Next: "Next",
    NextDashView: "NextDashView",
    NextLink: "NextLink",
    Organize: "Organize",
    PlanView: "PlanView",
    Previous: "Previous",
    PreviousLink: "PreviousLink",
    ProfileView: "ProfileView",
    ProjectsView: "ProjectsView",
    Redo: "Redo",
    Search: "Search",
    ShareProject: "ShareProject",
    ShowArchivedTasks: "ShowArchivedTasks",
    SystemTheme: "SystemTheme",
    ToggleTaskStatus: "ToggleTaskStatus",
    Unarchive: "Unarchive",
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
      selected: () => mode.current === "light",
    }),
    new Action({
      id: ActionIds.DarkTheme,
      callback: () => setMode("dark"),
      category: Categories.Theme,
      name: "Dark",
      description: "Set the theme to dark mode",
      icon: Moon,
      selected: () => mode.current === "dark",
    }),
    new Action({
      id: ActionIds.SystemTheme,
      callback: () => resetMode(),
      category: Categories.Theme,
      name: "System",
      description: "Set the theme to system",
      icon: SunMoon,
      selected: () => mode.current === "system",
    }),
    new NavigationAction({
      id: ActionIds.Home,
      href: "/",
      category: Categories.Navigation,
      name: "Home",
      description: "Navigate Home",
      icon: House,
    }),
  ];

  onMount(() => {
    return command.register(...actions);
  });
</script>

<Commander bind:open={paletteOpen} {command} />

{@render children()}
