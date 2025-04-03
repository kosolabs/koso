<script lang="ts">
  import { events } from "$lib/kosui";
  import { Button } from "$lib/kosui/button";
  import { Shortcut } from "$lib/kosui/shortcut";
  import type { YTaskProxy } from "$lib/yproxy";
  import { Pencil, X } from "lucide-svelte";
  import { onMount } from "svelte";
  import { slide } from "svelte/transition";
  import type { Koso } from "./koso.svelte";
  import MarkdownViewer from "./markdown-viewer.svelte";

  type Props = {
    koso: Koso;
    task: YTaskProxy;
  };
  let { koso, task }: Props = $props();

  let editing = $state(false);

  function stopEditing() {
    editing = false;
  }

  function toggleEditing() {
    editing = !editing;
  }

  function close() {
    stopEditing();
    koso.editor = false;
  }

  function handleKeyDownEditing(event: KeyboardEvent) {
    if (Shortcut.ESCAPE.matches(event)) {
      stopEditing();
    }
    event.stopImmediatePropagation();
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (Shortcut.ESCAPE.matches(event)) {
      close();
      event.stopImmediatePropagation();
    }
  }

  onMount(() => {
    return events.on("keydown", handleKeyDown);
  });
</script>

{#if koso.editor}
  <div class="relative mb-2 rounded-md border" transition:slide>
    <div
      class="flex items-center p-2 text-lg font-extralight"
      role="heading"
      aria-level="1"
      ondblclick={toggleEditing}
    >
      {task.name}
      <div class="top-2 right-2 ml-auto flex gap-2">
        <Button icon={Pencil} onclick={toggleEditing} />
        <Button icon={X} onclick={close} />
      </div>
    </div>
    {#if editing}
      <!-- svelte-ignore a11y_autofocus -->
      <textarea
        bind:value={task.desc}
        autofocus
        class="h-[40vh] w-full p-2"
        onblur={stopEditing}
        onkeydown={handleKeyDownEditing}
      ></textarea>
    {:else if task.desc !== ""}
      <hr />
      <div
        class="p-2"
        role="document"
        ondblclick={toggleEditing}
        transition:slide
      >
        <MarkdownViewer value={task.desc} />
      </div>
    {/if}
  </div>
{/if}
