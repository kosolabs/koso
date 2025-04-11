<script lang="ts">
  import { CodeMirror } from "$lib/components/ui/code-mirror";
  import { Button } from "$lib/kosui/button";
  import { Shortcut } from "$lib/kosui/shortcut";
  import { Eye, Pencil, Trash, X } from "lucide-svelte";
  import { tick } from "svelte";
  import { toast } from "svelte-sonner";
  import { slide } from "svelte/transition";
  import type { DetailPanelStates } from "./koso.svelte";
  import MarkdownViewer from "./markdown-viewer.svelte";
  import { getProjectContext } from "./project-context.svelte";

  type DetailPanelRenderer = {
    detailPanel: DetailPanelStates;
  };

  type Props = {
    taskId: string | undefined;
    detailPanelRenderer: DetailPanelRenderer;
  };
  let { taskId, detailPanelRenderer }: Props = $props();
  let editor: CodeMirror | undefined = $state();

  const { koso } = getProjectContext();

  let task = $derived(taskId ? koso.getTask(taskId) : undefined);

  function hideDetails() {
    detailPanelRenderer.detailPanel = "none";
  }

  function viewDetails() {
    detailPanelRenderer.detailPanel = "view";
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
    detailPanelRenderer.detailPanel = "edit";
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

{#if detailPanelRenderer.detailPanel !== "none"}
  <div class="relative mb-2 rounded-md border" transition:slide>
    <div
      class="flex items-center p-2 text-lg font-extralight"
      role="heading"
      aria-level="1"
      ondblclick={editDetails}
    >
      {$task?.name || "No task selected"}
      <div class="top-2 right-2 ml-auto flex gap-1">
        {#if taskId && koso.isEditable(taskId) && detailPanelRenderer.detailPanel === "view"}
          <Button
            aria-label="Edit task description"
            icon={Pencil}
            variant="plain"
            onclick={editDetails}
          />
        {/if}
        {#if detailPanelRenderer.detailPanel === "edit"}
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
    <div
      class="h-96 max-h-96 overflow-scroll"
      role="document"
      ondblclick={editDetails}
    >
      {#if $task && $task.desc && task && task.desc}
        {#if koso.isEditable($task.id) && detailPanelRenderer.detailPanel === "edit"}
          <CodeMirror
            bind:this={editor}
            yText={task.desc}
            onkeydown={handleKeyDownEditing}
          />
        {:else}
          <MarkdownViewer class="p-2" value={$task.desc.toString()} />
        {/if}
      {/if}
    </div>
  </div>
{/if}
