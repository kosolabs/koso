<script lang="ts">
  import { MarkdownViewer } from "$lib/components/ui/markdown-viewer";
  import { getPrefsContext } from "$lib/components/ui/prefs";
  import { Button } from "$lib/kosui/button";
  import { Tooltip } from "$lib/kosui/tooltip";
  import type { YTaskProxy } from "$lib/yproxy";
  import { Eye, FileText, Pencil } from "lucide-svelte";

  type Props = {
    task: YTaskProxy;
    onSelect?: () => void;
  };
  let { task, onSelect }: Props = $props();

  let open = $state(false);

  const prefs = getPrefsContext();

  function view() {
    open = false;
    prefs.detailPanel = "view";
    onSelect?.();
  }

  function edit() {
    open = false;
    prefs.detailPanel = "edit";
    onSelect?.();
  }
</script>

{#if task.desc}
  <Tooltip bind:open rich click class="flex max-h-2/5 max-w-3/5 p-0">
    {#snippet trigger({ onclick, ...restProps })}
      <Button
        variant="plain"
        color="primary"
        shape="circle"
        aria-label="Show task description panel"
        icon={FileText}
        onclick={(event) => {
          onclick?.();
          event?.stopImmediatePropagation();
        }}
        ondblclick={view}
        {...restProps}
      />
    {/snippet}
    <div class="flex grow flex-col">
      <div class="grow overflow-y-scroll p-2">
        <MarkdownViewer value={task.desc.toString()} />
      </div>
      <div class="flex grow-0 place-content-end border-t p-2">
        <Button variant="plain" icon={Eye} onclick={view}>View</Button>
        <Button variant="plain" icon={Pencil} onclick={edit}>Edit</Button>
      </div>
    </div>
  </Tooltip>
{/if}
