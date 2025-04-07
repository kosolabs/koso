<script module lang="ts">
  import { Action, Commander, Registry } from "$lib/kosui/command";
  import { Shortcut } from "$lib/kosui/shortcut";
  import { Terminal } from "lucide-svelte";
  import { onMount } from "svelte";

  export type ActionID =
    | "Block"
    | "Clear"
    | "Collapse"
    | "CollapseAll"
    | "CommandPalette"
    | "DarkTheme"
    | "Delete"
    | "DetailPanelClose"
    | "DetailPanelViewer"
    | "DetailPanelEditor"
    | "Edit"
    | "Expand"
    | "ExpandAll"
    | "ExportProject"
    | "HideDoneTasks"
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
    | "Previous"
    | "PreviousLink"
    | "Redo"
    | "Search"
    | "ShowDoneTasks"
    | "SystemTheme"
    | "ToggleTaskStatus"
    | "Undent"
    | "Undo";
  export const command = new Registry<ActionID>();
</script>

<script lang="ts">
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
  ];

  onMount(() => {
    return command.register(...actions);
  });
</script>

<Commander bind:open={paletteOpen} {command} />
