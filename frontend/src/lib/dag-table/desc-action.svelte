<script lang="ts">
  import { Button } from "$lib/kosui/button";
  import { Tooltip, type TooltipTriggerProps } from "$lib/kosui/tooltip";
  import type { YTaskProxy } from "$lib/yproxy";
  import { FilePlus2, FileText } from "lucide-svelte";
  import type { DetailPanelStates } from "./koso.svelte";
  import MarkdownViewer from "./markdown-viewer.svelte";

  interface DetailPanelRenderer {
    detailPanel: DetailPanelStates;
  }

  type Props = {
    detailPanelRenderer: DetailPanelRenderer;
    task: YTaskProxy;
  };
  let { detailPanelRenderer, task }: Props = $props();
</script>

{#snippet button(props: TooltipTriggerProps)}
  <Button
    class="m-0 p-2"
    variant="plain"
    color="primary"
    shape="circle"
    aria-label="Show task description panel"
    icon={task.desc !== null ? FileText : FilePlus2}
    onclick={() => (detailPanelRenderer.detailPanel = "view")}
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
