<script lang="ts">
  import { CodeMirror } from "$lib/components/ui/code-mirror";
  import { events } from "$lib/kosui";
  import { Button } from "$lib/kosui/button";
  import { Shortcut } from "$lib/kosui/shortcut";
  import { YTaskProxy, type Task } from "$lib/yproxy";
  import { Pencil, Trash, X } from "lucide-svelte";
  import { slide } from "svelte/transition";
  import * as Y from "yjs";
  import type { Koso } from "./koso.svelte";
  import MarkdownViewer from "./markdown-viewer.svelte";

  type Props = {
    koso: Koso;
    task: YTaskProxy;
  };
  let { koso, task: yTask }: Props = $props();

  let yDesc: Y.Text | null = $state(yTask.desc);
  let task: Task = $state(yTask.toJSON());
  let desc: string | null = $state(yTask.desc ? yTask.desc.toString() : null);

  function hideDetails() {
    koso.hideDetailPanel();
  }

  function viewDetails() {
    koso.showDetailViewer();
  }

  function editDetails() {
    yDesc = yTask.getOrNewDesc();
    koso.showDetailEditor();
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
    const observer = () => {
      task = yTask.toJSON();
      if (yDesc !== yTask.desc) {
        yDesc = yTask.desc;
      }
    };
    return yTask.observe(observer);
  });

  $effect(() => {
    const maybeYDesc = yDesc;
    if (maybeYDesc) {
      const observer = () => (desc = maybeYDesc.toString());
      maybeYDesc.observe(observer);
      return () => maybeYDesc.unobserve(observer);
    } else {
      desc = null;
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
        <Button icon={Pencil} variant="plain" onclick={editDetails} />
        {#if yDesc}
          <Button icon={Trash} variant="plain" onclick={deleteDetails} />
        {/if}
        <Button icon={X} variant="plain" onclick={close} />
      </div>
    </div>
    {#if yDesc && koso.detailPanel === "edit"}
      <hr />
      <CodeMirror yText={yDesc} onkeydown={handleKeyDownEditing} />
    {:else if desc !== null}
      <hr />
      <div class="p-2" role="document" ondblclick={editDetails}>
        <MarkdownViewer value={desc} />
      </div>
    {/if}
  </div>
{/if}
