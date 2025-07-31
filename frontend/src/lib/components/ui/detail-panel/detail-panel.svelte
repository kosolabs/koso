<script module lang="ts">
  import { CodeMirror } from "$lib/components/ui/code-mirror";
  import { MarkdownViewer } from "$lib/components/ui/markdown-viewer";
  import { getPrefsContext } from "$lib/components/ui/prefs";
  import { getProjectContext } from "$lib/dag-table/project-context.svelte";
  import { Eye, Pencil, Trash, X } from "@lucide/svelte";
  import { Button, Shortcut } from "kosui";
  import { tick } from "svelte";
  import { toast } from "svelte-sonner";
  import { Editable } from "../editable";

  export type DetailPanelState = "none" | "view" | "edit";

  export type DetailPanelProps = {
    taskId: string | undefined;
  };
</script>

<script lang="ts">
  let { taskId }: DetailPanelProps = $props();
  let editor: CodeMirror | undefined = $state();

  let titleEditable: Editable | undefined = $state();

  const { koso } = getProjectContext();
  const prefs = getPrefsContext();

  let task = $derived(taskId ? koso.getTask(taskId) : undefined);

  export function editTitle() {
    titleEditable?.edit();
  }

  function hideDetails() {
    prefs.detailPanel = "none";
  }

  function viewDetails() {
    prefs.detailPanel = "view";
  }

  function editDetails() {
    if (!task) {
      throw new Error("Failed to edit task details because task was null");
    }
    if (!koso.isEditable(task.id)) {
      toast.warning(
        "This is a managed task and cannot be edited in Koso directly.",
      );
      return;
    }
    task.newDesc();
    prefs.detailPanel = "edit";
    // Focus the editor after it gets rendered (after a tick)
    tick().then(() => editor?.focus());
  }

  function deleteDetails() {
    if (!task) {
      throw new Error(
        "Failed to delete task description because task was null",
      );
    }
    task.delDesc();
  }

  function handleKeyDownEditing(event: KeyboardEvent) {
    if (Shortcut.ESCAPE.matches(event)) {
      viewDetails();
    }
    event.stopImmediatePropagation();
  }
</script>

<div class="relative flex h-full flex-col rounded-md border">
  <div
    class="flex items-center gap-2 p-2"
    role="heading"
    aria-label="Task details"
    aria-level="1"
  >
    {#if $task}
      {#if koso.isEditable($task.id)}
        <Editable
          bind:this={titleEditable}
          class="text-lg font-extralight"
          value={$task.name}
          onsave={async (name) => {
            koso.setTaskName($task.id, name);
          }}
        />
      {:else}
        <div class="text-lg font-extralight">
          {$task.name || "Untitled"}
        </div>
      {/if}
    {:else}
      <div class="text-lg font-extralight">No task selected</div>
    {/if}
    <div class="ml-auto flex gap-1">
      {#if taskId && koso.isEditable(taskId) && (prefs.detailPanel === "view" || !$task || !$task.desc)}
        <Button
          aria-label="Edit task description"
          icon={Pencil}
          variant="plain"
          onclick={editDetails}
        />
      {/if}
      {#if $task && $task.desc && prefs.detailPanel === "edit"}
        <Button
          aria-label="View task description"
          icon={Eye}
          variant="plain"
          onclick={viewDetails}
        />
      {/if}
      {#if $task && $task.desc !== null}
        <Button
          aria-label="Delete task description"
          icon={Trash}
          variant="plain"
          onclick={deleteDetails}
        />
      {/if}
      <Button
        aria-label="Hide task description panel"
        icon={X}
        variant="plain"
        onclick={hideDetails}
      />
    </div>
  </div>
  <hr />
  <div class="grow overflow-y-scroll" role="document" ondblclick={editDetails}>
    {#if $task && task && $task.desc && task.desc}
      {#if koso.isEditable($task.id) && prefs.detailPanel === "edit"}
        <CodeMirror
          bind:this={editor}
          yText={task.desc}
          onkeydown={handleKeyDownEditing}
        />
      {:else}
        <MarkdownViewer
          class="p-2"
          value={$task.desc.toString()}
          ondblclick={editDetails}
        />
      {/if}
    {/if}
  </div>
</div>
