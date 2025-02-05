<script lang="ts">
  import * as Tooltip from "$lib/components/ui/tooltip";
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

  let open = $state(false);

  $effect(() => {
    const timeout = window.setTimeout(() => (open = true), 500);
    return () => clearTimeout(timeout);
  });
</script>

<Tooltip.Provider>
  <Tooltip.Root {open}>
    <Tooltip.Trigger>
      {#snippet child({ props })}
        <button
          {...props}
          class={cn(
            "absolute -my-[0.1rem] h-1 rounded",
            type === "Peer" ? "bg-primary" : "bg-secondary",
          )}
          style="width: {width}px; margin-left: {offset}px;"
          aria-label={`Task ${dest.num} ${type} Drop Indicator`}
        ></button>
      {/snippet}
    </Tooltip.Trigger>
    <Tooltip.Portal>
      <Tooltip.Content
        class={cn(
          type === "Peer"
            ? "bg-primary text-primary-foreground"
            : "bg-secondary text-secondary-foreground",
        )}
      >
        <Tooltip.Arrow
          class={cn(
            "z-50",
            type === "Peer" ? "text-primary" : "text-secondary",
          )}
        />
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
      </Tooltip.Content>
    </Tooltip.Portal>
  </Tooltip.Root>
</Tooltip.Provider>
