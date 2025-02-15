<script lang="ts">
  import { PlainTooltip } from "$lib/kosui/tooltip";
  import { cn } from "$lib/kosui/utils";
  import type { YTaskProxy } from "$lib/yproxy";
  import { ArrowBigDown } from "lucide-svelte";

  type Props = {
    src: YTaskProxy;
    dest: YTaskProxy;
    width: number;
    offset: number;
    type: "Peer" | "Child";
  };
  const { src, dest, width, offset, type }: Props = $props();

  let tooltip: PlainTooltip | undefined = $state();
  $effect(() => tooltip?.show());
</script>

<PlainTooltip bind:this={tooltip} arrow>
  {#snippet trigger(ref)}
    <button
      bind:this={ref.value}
      class={cn(
        "absolute -my-[0.1rem] h-1 rounded",
        type === "Peer" ? "bg-primary" : "bg-secondary",
      )}
      style="width: {width}px; margin-left: {offset}px;"
      aria-label={`Task ${dest.num} ${type} Drop Indicator`}
    ></button>
  {/snippet}
  {#snippet children()}
    <div class="flex flex-col items-center text-sm">
      <div class="flex items-center gap-1">
        <span>Task</span>
        <span class="font-bold">{src.num}</span>
      </div>
      <ArrowBigDown size={16} />
      <div class="flex items-center gap-1">
        <span class="font-bold">{type}</span>
        <span>of task</span>
        <span class="font-bold">{dest.num}</span>
      </div>
    </div>
  {/snippet}
</PlainTooltip>
