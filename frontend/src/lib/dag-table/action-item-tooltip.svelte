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
    ResponsibleForParent: Blocks,
  };
</script>

<script lang="ts">
  import { goto } from "$app/navigation";
  import Link from "$lib/kosui/link/link.svelte";

  const { koso } = getInboxContext();

  let { item }: ActionItemTooltipProps = $props();
</script>

<Tooltip arrow>
  {#snippet trigger(props)}
    {@const reason = item.reasons[0]}
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
      <div class="pr-2 max-md:hidden">
        {#if reason.name === "Actionable"}
          <div>Assigned to you</div>
        {:else if reason.name === "ResponsibleForParent"}
          {@const task = reason.parents[0]}
          <div>
            Responsible for
            <Link
              href={`/projects/${koso.projectId}?taskId=${task.id}`}
              onclick={(event) => {
                event.stopPropagation();
                event.preventDefault();
                goto(`/projects/${koso.projectId}?taskId=${task.id}`);
              }}
            >
              Task {task.num}
            </Link>
          </div>
        {/if}
      </div>
    </div>
  {/snippet}
  <div class="flex flex-col gap-2 wrap-normal">
    {#each item.reasons as reason}
      <div class="flex items-center gap-2">
        <Lightbulb size={12} />
        {#if reason.name === "Actionable"}
          <div>
            This task is in your inbox because it is <b>not blocked</b> and it
            is <b>assigned to you</b>. Complete it and <b>mark it done</b> to clear
            it from your inbox.
          </div>
        {:else if reason.name === "ResponsibleForParent"}
          {@const task = reason.parents[0]}
          <div>
            It is not assigned and you are responsible for it's parent
            <Link
              class="text-m3-inverse-primary"
              href={`/projects/${koso.projectId}?taskId=${task.id}`}
              onclick={(event) => {
                event.stopPropagation();
                event.preventDefault();
                goto(`/projects/${koso.projectId}?taskId=${task.id}`);
              }}
            >
              Task {task.num} - {task.name}
            </Link>. Assign the task to clear it from your inbox.
          </div>
        {/if}
      </div>
    {/each}
  </div>
</Tooltip>
