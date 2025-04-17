<script lang="ts">
  import { goto } from "$app/navigation";
  import { type User } from "$lib/auth.svelte";
  import { parseChipProps, type ChipProps } from "$lib/components/ui/chip";
  import { Editable } from "$lib/components/ui/editable";
  import { ManagedTaskIcon } from "$lib/components/ui/managed-task-icon";
  import { TaskStatus } from "$lib/components/ui/task-status";
  import { UserSelect } from "$lib/components/ui/user-select";
  import { Chip } from "$lib/kosui/chip";
  import { Link } from "$lib/kosui/link";
  import { cn } from "$lib/utils";
  import type { YTaskProxy } from "$lib/yproxy";
  import DescAction from "./desc-action.svelte";
  import { getInboxContext } from "./inbox-context.svelte";
  import LinkPanel, { type Mode } from "./link-panel.svelte";
  import TaskAction from "./task-action.svelte";

  type Props = {
    index: number;
    task: YTaskProxy;
    users: User[];
  };
  const { index, task, users }: Props = $props();

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
    index % 2 === 0 && "bg-m3-surface-container/30",
    inbox.selected?.id === task.id && "outline-m3-primary",
  )}
  aria-label={`Task ${task.num}`}
  onclick={handleRowClick}
  bind:this={rowElement}
>
  <td class={cn("border-t px-2")}>
    <div class="flex items-center">
      <div class="overflow-x-hidden whitespace-nowrap">
        <Link
          href={`/projects/${koso.projectId}?taskId=${task.id}`}
          onclick={(event) => event.stopPropagation()}
        >
          {task.num}
        </Link>
      </div>
    </div>
  </td>
  {#if koso.debug}
    <td class={cn("border-t border-l p-2 text-xs lg:text-nowrap")}>
      {task.id}
    </td>
  {/if}
  <td class={cn("border-t border-l p-2")}>
    <TaskStatus {koso} {task} inboxView={true} bind:this={taskStatus} />
  </td>
  <td class={cn("w-full border-t border-l px-2 py-1")}>
    <div class={cn("flex items-center gap-x-1")}>
      {#if koso.isManagedTask(task.id)}
        <ManagedTaskIcon kind={task.yKind ?? ""} />
      {/if}
      <div class="flex w-full flex-wrap-reverse gap-x-1">
        {#if tags.length > 0}
          <div class="flex flex-wrap items-center gap-x-1">
            {#each tags as { title, description, onClick, onDelete }}
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
            onsave={async (name) => {
              koso.setTaskName(task.id, name);
            }}
            ondone={() => edit(false)}
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
            disabled={!task.url}
            underline="none"
          >
            {task.name || "Untitled"}
          </Link>
        {/if}
        <LinkPanel
          {task}
          {koso}
          bind:open={linkOpen}
          bind:mode={linkMode}
          anchorEl={rowElement}
        />
      </div>
    </div>
  </td>
  <td class={cn("border-t px-1")}>
    <div class="flex items-center">
      <DescAction detailPanelRenderer={inbox} {task} />
      <TaskAction />
    </div>
  </td>
  <td class={cn("border-t border-l p-2")}>
    <UserSelect
      {users}
      value={assignee}
      {editable}
      onSelect={(user) => {
        koso.setAssignee(task.id, user);
      }}
    />
  </td>
</tr>
