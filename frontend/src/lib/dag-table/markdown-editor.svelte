<script lang="ts">
  import { CodeMirror } from "$lib/components/ui/code-mirror";
  import { events } from "$lib/kosui";
  import { Button } from "$lib/kosui/button";
  import { Shortcut } from "$lib/kosui/shortcut";
  import { YTaskProxy, type Task } from "$lib/yproxy";
  import { Pencil, Trash, X } from "lucide-svelte";
  import { toast } from "svelte-sonner";
  import { slide } from "svelte/transition";
  import * as Y from "yjs";
  import type { Koso } from "./koso.svelte";
  import MarkdownViewer from "./markdown-viewer.svelte";

  type Props = {
    koso: Koso;
    task: YTaskProxy;
  };
  let { koso, task: yTask }: Props = $props();

  let task: Task = $state.raw(yTask.toJSON());
  let yDesc: Y.Text | null = $state.raw(yTask.desc);

  function hideDetails() {
    koso.detailPanel = "none";
  }

  function viewDetails() {
    koso.detailPanel = "view";
  }

  function editDetails() {
    if (!koso.isEditable(task.id)) {
      toast.warning(
        "This is a managed task and cannot be edited in Koso directly.",
      );
      return;
    }
    yDesc = yTask.getOrNewDesc();
    koso.detailPanel = "edit";
  }

  function deleteDetails() {
    yDesc = null;
    yTask.delDesc();
  }

  function handleKeyDownEditing(event: KeyboardEvent) {
    if (Shortcut.ESCAPE.matches(event)) {
      viewDetails();
    }
    event.stopImmediatePropagation();
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (Shortcut.ESCAPE.matches(event)) {
      if (koso.detailPanel === "edit") {
        viewDetails();
      } else {
        hideDetails();
      }
      event.stopImmediatePropagation();
    }
  }

  $effect(() => {
    task = yTask.toJSON();
    yDesc = yTask.desc;
    return yTask.observeDeep(() => {
      task = yTask.toJSON();
      yDesc = yTask.desc;
    });
  });

  $effect(() => {
    if (!koso.isEditable(task.id) && koso.detailPanel === "edit") {
      viewDetails();
    }
  });

  $effect(() => {
    if (koso.selected && koso.detailPanel !== "none") {
      return events.on("keydown", handleKeyDown);
    }
  });
</script>

{#if koso.detailPanel !== "none"}
  <div class="relative mb-2 rounded-md border" transition:slide>
    <div
      class="flex items-center p-2 text-lg font-extralight"
      role="heading"
      aria-level="1"
      ondblclick={editDetails}
    >
      {task.name}
      <div class="top-2 right-2 ml-auto flex gap-1">
        {#if koso.isEditable(task.id)}
          <Button icon={Pencil} variant="plain" onclick={editDetails} />
        {/if}
        {#if yDesc !== null}
          <Button icon={Trash} variant="plain" onclick={deleteDetails} />
        {/if}
        <Button icon={X} variant="plain" onclick={close} />
      </div>
    </div>
    <hr />
    <div
      class="h-96 max-h-96 overflow-scroll"
      role="document"
      ondblclick={editDetails}
    >
      {#if yDesc && koso.isEditable(task.id) && koso.detailPanel === "edit"}
        <CodeMirror yText={yDesc} onkeydown={handleKeyDownEditing} />
      {:else if task.desc}
        <MarkdownViewer class="p-2" value={task.desc} />
      {/if}
    </div>
  </div>
{/if}
