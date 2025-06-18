<script lang="ts">
  import { Tooltip } from "$lib/kosui/tooltip";
  import type { YTaskProxy } from "$lib/yproxy";
  import { ArrowBigRight } from "@lucide/svelte";

  type Props = {
    src: YTaskProxy;
    dest: YTaskProxy;
    height: number;
    width: number;
    type: "Peer" | "Child";
  };
  const { src, dest, width, height, type }: Props = $props();

  let tooltip: Tooltip | undefined = $state();
  $effect(() => tooltip?.show());
</script>

<Tooltip
  bind:this={tooltip}
  arrow
  aria-label={`Task ${dest.num} ${type} Drop Indicator`}
>
  {#snippet trigger({ ref })}
    <div
      use:ref
      class="bg-m3-primary absolute h-0 -translate-y-1/2 rounded"
      style:width={type === "Peer" ? `${width}px` : `${width - 20}px`}
      style:height={type === "Peer" ? "4px" : "0"}
      style:top={type === "Peer" ? `${height}px` : `${height / 2}px`}
    ></div>
  {/snippet}
  <div class="flex items-center gap-1">
    <span>Task</span>
    <span class="font-bold">{src.num}</span>
    <ArrowBigRight size={16} />
    <span class="font-bold">{type}</span>
    <span>of task</span>
    <span class="font-bold">{dest.num}</span>
  </div>
</Tooltip>
