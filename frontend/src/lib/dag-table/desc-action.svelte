<script lang="ts">
  import { MarkdownViewer } from "$lib/components/ui/markdown-viewer";
  import { getPrefsContext } from "$lib/components/ui/prefs";
  import { Button } from "$lib/kosui/button";
  import { Tooltip } from "$lib/kosui/tooltip";
  import type { YTaskProxy } from "$lib/yproxy";
  import { FilePlus2, FileText } from "lucide-svelte";

  type Props = {
    task: YTaskProxy;
  };
  let { task }: Props = $props();

  let open = $state(false);

  const prefs = getPrefsContext();
</script>

{#if task.desc}
  <Tooltip
    bind:open
    rich
    click
    class="max-h-2/5 max-w-3/5 overflow-y-scroll p-1"
  >
    {#snippet trigger({ onclick, ...restProps })}
      <Button
        class="m-0 p-2"
        variant="plain"
        color="primary"
        shape="circle"
        aria-label="Show task description panel"
        icon={task.desc !== null ? FileText : FilePlus2}
        onclick={(event) => {
          onclick?.();
          event?.stopImmediatePropagation();
        }}
        ondblclick={() => {
          if (prefs.detailPanel === "none") {
            prefs.detailPanel = "view";
          } else {
            prefs.detailPanel = "none";
          }
          open = false;
        }}
        {...restProps}
      />
    {/snippet}
    <MarkdownViewer class="p-1" value={task.desc.toString()} />
  </Tooltip>
{/if}
