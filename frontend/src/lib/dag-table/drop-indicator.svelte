<script lang="ts">
  import { PlainTooltip } from "$lib/kosui/tooltip";
  import { cn } from "$lib/utils";
  import type { YTaskProxy } from "$lib/yproxy";
  import { ArrowBigRight } from "lucide-svelte";

  type Props = {
    src: YTaskProxy;
    dest: YTaskProxy;
    height: number;
    width: number;
    offset: number;
    type: "Peer" | "Child";
  };
  const { src, dest, width, height, offset, type }: Props = $props();

  let tooltip: PlainTooltip | undefined = $state();
  $effect(() => tooltip?.show());
</script>

<PlainTooltip
  bind:this={tooltip}
  arrow
  aria-label={`Task ${dest.num} ${type} Drop Indicator`}
>
  {#snippet trigger(ref)}
    <div
      bind:this={ref.value}
      class={cn("bg-m3-primary absolute rounded")}
      style:height={type === "Peer" ? "4px" : "0"}
      style:margin-top={type === "Peer" ? "-2px" : `-${height / 2}px`}
      style:width={`${width}px`}
      style:margin-left={`${offset}px`}
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
</PlainTooltip>
