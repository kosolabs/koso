<script module lang="ts">
  import { Button } from "$lib/kosui/button";
  import { Tooltip } from "$lib/kosui/tooltip";
  import { Blocks, CirclePlay, Icon, Lightbulb } from "lucide-svelte";
  import {
    getInboxContext,
    type ActionItem,
    type Reason,
  } from "./inbox-context.svelte";

  export type ActionItemTooltipProps = {
    item: ActionItem;
  };

  type IconMap = { [key in Reason["name"]]: typeof Icon };
  const icons: IconMap = {
    Actionable: CirclePlay,
    ParentOwner: Blocks,
  };
</script>

<script lang="ts">
  import { goto } from "$app/navigation";
  import Link from "$lib/kosui/link/link.svelte";

  const { koso } = getInboxContext();

  let { item }: ActionItemTooltipProps = $props();
</script>

<Tooltip class="w-[min(calc(80%),32em)]" rich arrow click>
  {#snippet trigger(props)}
    <div class="flex items-center text-sm">
      <Button
        class="h-auto p-2"
        variant="plain"
        color="primary"
        shape="circle"
        aria-label="Show reasons why task is actionable"
        icon={icons[item.reasons[0].name]}
        {...props}
      />
    </div>
  {/snippet}
  <div class="flex flex-col gap-2">
    {#each item.reasons as reason}
      <div class="flex items-center gap-2">
        <Lightbulb class="w-10" />
        <div>
          {#if reason.name === "Actionable"}
            This task is in your inbox because it is <b>not blocked</b> and it
            is <b>assigned to you</b>. Complete it and <b>mark it done</b> to clear
            it from your inbox.
          {:else if reason.name === "ParentOwner"}
            {@const task = reason.parents[0]}
            This task is in your inbox because you are the
            <b>owner of it's parent</b>
            <Link
              href={`/projects/${koso.projectId}?taskId=${task.id}`}
              onclick={(event) => {
                event.stopPropagation();
                event.preventDefault();
                goto(`/projects/${koso.projectId}?taskId=${task.id}`);
              }}
            >
              Task {task.num} - {task.name}
            </Link>
            and it is not assigned. Assign the task to clear it from your inbox.
          {/if}
        </div>
      </div>
    {/each}
  </div>
</Tooltip>
