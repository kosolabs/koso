<script lang="ts">
  import { MarkdownViewer } from "$lib/components/ui/markdown-viewer";
  import { getPrefsContext } from "$lib/components/ui/prefs";
  import { Button } from "$lib/kosui/button";
  import { Tooltip, type TooltipTriggerProps } from "$lib/kosui/tooltip";
  import type { YTaskProxy } from "$lib/yproxy";
  import { FilePlus2, FileText } from "lucide-svelte";

  type Props = {
    task: YTaskProxy;
  };
  let { task }: Props = $props();

  const prefs = getPrefsContext();
</script>

{#snippet button(props: TooltipTriggerProps)}
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

{#if task.desc}
  <Tooltip class="bg-m3-surface text-m3-on-surface border shadow">
    {#snippet trigger(props)}
      {@render button(props)}
    {/snippet}
    <MarkdownViewer class="p-2" value={task.desc.toString()} />
  </Tooltip>
{:else}
  {@render button({})}
{/if}
