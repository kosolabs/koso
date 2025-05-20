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

  const prefs = getPrefsContext();
</script>

{#if task.desc}
  <Tooltip rich click class="max-h-2/5 max-w-3/5 overflow-y-scroll p-1">
    {#snippet trigger(props)}
      <Button
        class="m-0 p-2"
        variant="plain"
        color="primary"
        shape="circle"
        aria-label="Show task description panel"
        icon={task.desc !== null ? FileText : FilePlus2}
        onclick={() => (prefs.detailPanel = "view")}
        {...props}
      />
    {/snippet}
    <MarkdownViewer class="p-1" value={task.desc.toString()} />
  </Tooltip>
{/if}
