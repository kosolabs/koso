<script lang="ts">
  import { goto } from "$app/navigation";
  import { parseChipProps, type ChipProps } from "$lib/components/ui/chip";
  import { Deadline } from "$lib/components/ui/deadline";
  import { Editable } from "$lib/components/ui/editable";
  import { Estimate } from "$lib/components/ui/estimate";
  import { ManagedTaskIcon } from "$lib/components/ui/managed-task-icon";
  import { TaskStatus } from "$lib/components/ui/task-status";
  import { UserSelect } from "$lib/components/ui/user-select";
  import { Chip } from "$lib/kosui/chip";
  import { Goto } from "$lib/kosui/goto";
  import { Link } from "$lib/kosui/link";
  import type { User } from "$lib/users";
  import { cn } from "$lib/utils";
  import { Grip } from "lucide-svelte";
  import { tick } from "svelte";
  import ActionItemTooltip from "./action-item-tooltip.svelte";
  import DescAction from "./desc-action.svelte";
  import { ActionItem, getInboxContext } from "./inbox-context.svelte";
  import LinkTaskPanel, { type Mode } from "./link-task-panel.svelte";
  import TaskAction from "./task-action.svelte";

  type Props = {
    index: number;
    item: ActionItem;
    users: User[];
  };
  const { index, item, users }: Props = $props();
  const task = $derived(item.task);

  const inbox = getInboxContext();
  const { koso } = inbox;

  let rowElement: HTMLTableRowElement | undefined = $state();
  let taskStatus = $state<TaskStatus | undefined>();

  let isEditing = $state(false);
  let linkOpen = $state(false);
  let linkMode: Mode = $state("block");

  let assignee = $derived(getUser(users, task.assignee));
  let editable = $derived(koso.isEditable(task.id));
  let tags = $derived(getTags());
  let progress = $derived(koso.getProgress(task.id));

  export function edit(editing: boolean) {
    isEditing = editing;
  }

  export function showDoneConfetti() {
    taskStatus?.showDoneConfetti();
  }

  export function linkPanel(visible: boolean, mode: Mode) {
    linkOpen = visible;
    linkMode = mode;
  }

  function getTags(): ChipProps[] {
    const parents = inbox.koso.parents.get(task.id);
    if (!parents) return [];
    return parents
      .filter((parent) => parent !== "root")
      .map((parent) => koso.getTask(parent))
      .filter((parent) => parent.name.length > 0)
      .map((parent) => {
        const props: ChipProps = parseChipProps(parent.name);
        props.onClick = (event) => {
          event.stopPropagation();
          goto(`/projects/${koso.projectId}?taskId=${parent.id}`);
        };
        return props;
      });
  }

  function getUser(users: User[], email: string | null): User | null {
    for (const user of users) {
      if (user.email === email) {
        return user;
      }
    }
    return null;
  }

  function handleRowClick(event: MouseEvent) {
    event.preventDefault();
    inbox.selected = task.id;
  }
</script>

<tr
  tabindex="0"
  class={cn(
    "rounded outline-2 outline-transparent",
    index % 2 === 0 ? "bg-m3-surface-container" : "bg-m3-surface-container-low",
    inbox.selected?.id === task.id && "outline-m3-primary",
  )}
  aria-label={`Task ${task.num}`}
  onclick={handleRowClick}
  bind:this={rowElement}
>
  <td class={cn("border-t px-2")}>
    <div class="flex items-center gap-1">
      <Grip class="w-4" />
      <Goto href={`/projects/${koso.projectId}?taskId=${task.id}`}>
        {task.num}
      </Goto>
    </div>
  </td>
  {#if koso.debug}
    <td class={cn("border-t border-l p-2 text-xs lg:text-nowrap")}>
      {task.id}
    </td>
    <td class={cn("border-t border-l p-2 text-xs lg:text-nowrap")}>
      {item.priority}
    </td>
  {/if}
  <td class={cn("border-t border-l p-2")}>
    <TaskStatus {koso} {task} inboxView={true} bind:this={taskStatus} />
  </td>
  <td class={cn("w-full border-t border-l px-2 py-1")}>
    <div class={cn("flex items-center gap-x-1")}>
      {#if koso.isManagedTask(task.id)}
        <ManagedTaskIcon kind={task.kind} />
      {/if}
      <div class="flex w-full flex-wrap-reverse gap-x-1">
        {#if tags.length > 0}
          <div class="flex flex-wrap items-center gap-x-1">
            {#each tags as tag (tag)}
              {@const { title, description, onClick, onDelete } = tag}
              <Chip color="tertiary" title={description} {onClick} {onDelete}>
                {title}
              </Chip>
            {/each}
          </div>
        {/if}

        {#if editable}
          <Editable
            value={task.name}
            aria-label={`Task ${task.num} Edit Name`}
            editing={isEditing}
            closeFocus={rowElement}
            onclick={() => (inbox.selected = task.id)}
            onsave={async (name) => {
              koso.setTaskName(task.id, name);
            }}
          />
        {:else}
          <Link
            class={cn(
              "h-auto p-0 text-left text-sm text-wrap whitespace-normal",
              task.url ? "text" : "",
            )}
            aria-label={`Task ${task.num} Name`}
            onclick={() => {
              if (!task.url) throw new Error(`No URL set on task ${task}`);
              window.open(task.url, "_blank")!.focus();
            }}
            underline="none"
          >
            {task.name || "Untitled"}
          </Link>
        {/if}

        <LinkTaskPanel
          {task}
          {koso}
          bind:open={linkOpen}
          bind:mode={linkMode}
          anchorEl={rowElement}
          onBlockNewTask={async (taskId) => {
            await tick();
            inbox.selected = taskId;
          }}
        />
      </div>
    </div>
  </td>
  <td class={cn("border-t px-1")}>
    <div class="flex place-content-end items-center gap-0.5">
      <div class="max-sm:hidden">
        <DescAction {task} onSelect={() => (inbox.selected = task.id)} />
      </div>
      <ActionItemTooltip {item} />
      <TaskAction class="max-sm:hidden" />
    </div>
  </td>
  <td class={cn("w-0 border-t border-l p-2")}>
    <UserSelect
      {users}
      value={assignee}
      {editable}
      onSelect={(user) => {
        koso.setAssignee(task.id, user);
      }}
    />
  </td>
  <td class={cn("border-t border-l max-md:hidden")}>
    <div class="flex place-content-center items-center">
      <Estimate
        value={task.kind === "Rollup"
          ? progress.remainingEstimate
          : progress.estimate}
        editable={editable && task.kind !== "Rollup"}
        onSelect={(estimate) => {
          task.estimate = estimate;
        }}
      />
    </div>
  </td>
  {#if koso.debug}
    <td class={cn("border-t border-l p-2 max-md:hidden")}>
      <Deadline
        value={task.deadline}
        {editable}
        onSelect={(deadline) => {
          task.deadline = deadline;
        }}
      />
    </td>
  {/if}
</tr>
