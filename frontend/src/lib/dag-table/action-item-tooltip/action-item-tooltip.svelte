<script module lang="ts">
  import { Button } from "$lib/kosui/button";
  import { Goto } from "$lib/kosui/goto";
  import { Tooltip } from "$lib/kosui/tooltip";
  import { Blocks, CalendarCheck2, CirclePlay, Icon } from "lucide-svelte";
  import {
    ActionItem,
    getInboxContext,
    type Reason,
  } from "../inbox-context.svelte";
  import TooltipAction from "./tooltip-action.svelte";
  import TooltipReason from "./tooltip-reason.svelte";

  export type ActionItemTooltipProps = {
    item: ActionItem;
  };

  type IconMap = { [key in Reason["name"]]: typeof Icon };
  const icons: IconMap = {
    Actionable: CirclePlay,
    ParentOwner: Blocks,
    NeedsEstimate: CalendarCheck2,
  };
</script>

<script lang="ts">
  const { koso } = getInboxContext();

  let { item }: ActionItemTooltipProps = $props();
</script>

<Tooltip class="w-[min(calc(80%),32em)]" rich arrow click>
  {#snippet trigger(props)}
    <div class="flex items-center text-sm">
      <Button
        variant="plain"
        color={item.reasons.length === 1 &&
        item.reasons[0].name === "Actionable"
          ? "secondary"
          : "primary"}
        shape="circle"
        aria-label="Show reasons why task is actionable"
        icon={icons[item.reasons[0].name]}
        {...props}
      />
    </div>
  {/snippet}
  <div class="flex flex-col gap-2">
    {#each item.reasons as reason, index (reason.name)}
      {#if index > 0}
        <hr />
      {/if}
      {#if reason.name === "Actionable"}
        <TooltipReason title="Actionable" icon={icons[reason.name]}>
          This task is in your inbox because it is not blocked and it is
          assigned to you.
        </TooltipReason>
        <TooltipAction title="Complete the Task" score={reason.actions.done}>
          Complete the task and set it's status to Done.
        </TooltipAction>
        <TooltipAction title="Block the Task" score={reason.actions.block}>
          If the task is not currently actionable, block this task on another
          task.
        </TooltipAction>
        <TooltipAction
          title="Unassign Yourself"
          score={reason.actions.unassign}
        >
          If the task is ready for work, but you are already working on a task,
          unassign yourself from the task so that it shows up in the next
          available engineer's inbox.
        </TooltipAction>
      {:else if reason.name === "ParentOwner"}
        {@const task = reason.parents[0]}
        <TooltipReason title="Owner of Parent" icon={icons[reason.name]}>
          This task is in your inbox because you are the owner of it's parent:
          <Goto href={`/projects/${koso.projectId}?taskId=${task.id}`}>
            Task {task.num} - {task.name}
          </Goto>
          and it is not assigned.
        </TooltipReason>
        <TooltipAction
          title="Set the Task to Ready"
          score={reason.actions.ready}
        >
          If the task is ready for work, set the task to ready so that it shows
          up in the next available engineer's inbox.
        </TooltipAction>
        <TooltipAction title="Assign the Task" score={reason.actions.assign}>
          Assign the task to someone or to the team.
        </TooltipAction>
      {:else if reason.name === "NeedsEstimate"}
        <TooltipReason title="Task Needs Estimate" icon={icons[reason.name]}>
          This task is in your inbox because it is part of the current iteration
          and it needs an estimate. Estimate the task or assign it to someone
          else who has the context to estimate it.
        </TooltipReason>
        <TooltipAction
          title="Estimate the Task"
          score={reason.actions.estimate}
        >
          Estimate the effort needed to complete this task. Each point is worth
          approximately half a day.
        </TooltipAction>
        <TooltipAction title="Assign the Task" score={reason.actions.assign}>
          Assign the task to someone else on the team who has the context to
          estimate it.
        </TooltipAction>
      {/if}
    {/each}
  </div>
</Tooltip>
