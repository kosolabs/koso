<script module lang="ts">
  import { getAuthContext } from "$lib/auth.svelte";
  import { parseChipProps, type ChipProps } from "$lib/components/ui/chip";
  import { match } from "$lib/utils";
  import {
    Clipboard,
    ClipboardPlus,
    Network,
    SquarePlus,
  } from "@lucide/svelte";
  import {
    Button,
    Chip,
    Command,
    CommandContent,
    CommandDivider,
    CommandItem,
    CommandSearch,
    mergeComponentProps,
    Popover,
    Shortcut,
    ToggleButton,
    ToggleGroup,
    type PopoverProps,
  } from "kosui";
  import { getContext } from "svelte";
  import { compareTasks } from "./compare-tasks.svelte";
  import type { Koso } from "./koso.svelte";
  import { TaskLinkage } from "./koso.svelte";
  import { PlanningContext, type Node } from "./planning-context.svelte";

  export type Mode = "link" | "move" | "block";

  export type LinkPanelProps = {
    open: boolean;
    mode?: Mode;
    node: Node;
    onBlockNewTask?: (taskId: string) => void;
    koso: Koso;
  } & Omit<PopoverProps, "children">;
</script>

<script lang="ts">
  let {
    open = $bindable(false),
    mode = $bindable("link"),
    node,
    onBlockNewTask,
    koso,
    anchorEl,
    ...restProps
  }: LinkPanelProps = $props();

  const plan = getContext<PlanningContext>(PlanningContext);
  const auth = getAuthContext();

  let query = $state("");
  let setStatusBlocked: boolean = $state(true);
  let tasks = $derived(
    open
      ? koso.tasks
          .filter((t) => match(t.num, query) || match(t.name, query))
          .filter((t) => {
            if (mode === "link") {
              return koso.canLink(
                new TaskLinkage({ parentId: t.id, id: node.name }),
              );
            } else {
              return koso.canLink(
                new TaskLinkage({ parentId: node.name, id: t.id }),
              );
            }
          })
          .sort((t1, t2) => compareTasks(t1, t2, koso))
          .slice(0, 50)
      : [],
  );

  function select(taskId: string) {
    if (mode === "link") {
      koso.link(new TaskLinkage({ parentId: taskId, id: node.name }));
    } else if (mode === "move") {
      const dest = plan.getNodes(taskId)[0];
      plan.moveNode(node, dest, 0);
    } else if (mode === "block") {
      koso.doc.transact(() => {
        koso.link(new TaskLinkage({ parentId: node.name, id: taskId }));
        if (setStatusBlocked) {
          koso.setKind(node.name, "Task");
          koso.setTaskStatus(node.name, "Blocked", auth.user);
        }
      });
    } else {
      throw new Error(`Unknown mode: ${mode}`);
    }
    open = false;
  }

  function createAndLink() {
    if (mode !== "block") {
      throw new Error(`createAndLink only works on block, got: ${mode}`);
    }
    koso.doc.transact(() => {
      const newTaskId = koso.insertTask({
        name: query,
        parent: node.name,
        reporter: auth.user.email,
        assignee: auth.user.email,
      });
      if (setStatusBlocked) {
        koso.setKind(node.name, "Task");
        koso.setTaskStatus(node.name, "Blocked", auth.user);
        onBlockNewTask?.(newTaskId);
      }
    });
    open = false;
  }

  function getTags(taskId: string): ChipProps[] {
    let parents = koso.parents.get(taskId);
    if (!parents) return [];

    return parents
      .map((parent) => koso.getTask(parent))
      .filter((parent) => parent.name.length > 0)
      .map((parent) => parseChipProps(parent.name));
  }
</script>

<Popover
  bind:open
  {anchorEl}
  placement="bottom"
  class="shadow-m3-shadow/20 bg-m3-surface-container-high h-[min(40%,24em)] w-[min(calc(100%-1em),36em)] rounded-lg border shadow"
  {...mergeComponentProps(
    Popover,
    {
      onoutroend: () => (query = ""),
      onkeydown: (event) => {
        if (!Shortcut.ESCAPE.matches(event)) {
          event.stopImmediatePropagation();
        }
      },
    },
    restProps,
  )}
>
  <Command>
    <div class="flex place-content-center gap-1 p-1">
      <ToggleGroup bind:value={mode}>
        <ToggleButton value="link">Link to</ToggleButton>
        <ToggleButton value="move">Move to</ToggleButton>
        <ToggleButton value="block">Block on</ToggleButton>
      </ToggleGroup>
    </div>
    {#if mode === "block"}
      <div class="flex place-content-center gap-1 p-1 text-sm">
        <input
          type="checkbox"
          id="also-block"
          bind:checked={setStatusBlocked}
        />
        <label for="also-block">Set task to blocked after linking</label>
      </div>
    {/if}
    <CommandDivider />
    <div class="flex items-center gap-2">
      <CommandSearch
        bind:value={query}
        class="grow"
        placeholder={(() => {
          if (mode === "link") return "Link this task to...";
          if (mode === "move") return "Move this task to...";
          if (mode === "block") return "Block this task on...";
        })()}
      />
      {#if mode === "block"}
        <Button
          variant="plain"
          shape="circle"
          icon={SquarePlus}
          title="Create a new task"
          disabled={query.length === 0}
        />
      {/if}
    </div>
    <CommandDivider />
    <CommandContent>
      {#if tasks.length > 0}
        {#each tasks as task (task.id)}
          <CommandItem
            class="table-row"
            onSelect={() => select(task.id)}
            aria-label="Task {task.id} Command Item"
          >
            <div class="table-cell rounded-l px-2 py-1 align-middle">
              <div class="flex items-center gap-1" title="Task Number">
                <Clipboard size={14} />
                {task.num}
              </div>
            </div>
            <div class="table-cell w-full px-2 align-middle">
              <div class="flex items-center" title="Task Name">
                {task.name || "Untitled task"}
              </div>
            </div>
            <div class="table-cell px-2 align-middle text-nowrap">
              <div class="flex items-center gap-1" title="Subtasks">
                {task.children.length}
                <Network size={14} />
              </div>
            </div>
            <div class="table-cell px-2 align-middle text-nowrap">
              <div class="flex items-center gap-1" title="Status">
                {koso.getStatus(task.id)}
              </div>
            </div>
            <div class="table-cell rounded-r px-2 align-middle text-nowrap">
              <div class="flex items-center gap-1" title="Tags">
                <div class="flex flex-wrap items-center gap-x-1">
                  {#each getTags(task.id) as tag (tag)}
                    {@const { title, description } = tag}
                    <Chip title={description}>{title}</Chip>
                  {/each}
                </div>
              </div>
            </div>
          </CommandItem>
        {/each}
      {:else}
        <div class="p-2 text-center">No tasks found.</div>
      {/if}
      {#if mode === "block" && query.length > 0}
        <CommandItem
          class="table-row"
          onSelect={() => createAndLink()}
          title="Create new task"
        >
          <div class="table-cell rounded-l px-2 py-1 align-middle">
            <div class="flex items-center gap-1" title="Task Number">
              <ClipboardPlus size={14} />
              New
            </div>
          </div>
          <div class="table-cell w-full px-2 align-middle">
            <div class="flex items-center" title="Task Name">
              {query}
            </div>
          </div>
          <div class="table-cell px-2 align-middle text-nowrap">
            <div class="flex items-center gap-1" title="Subtasks">
              - <Network size={14} />
            </div>
          </div>
          <div class="table-cell px-2 align-middle text-nowrap">
            <div class="flex items-center gap-1" title="Status"></div>
          </div>
          <div class="table-cell rounded-r px-2 align-middle text-nowrap">
            <div class="flex items-center gap-1" title="Tags">
              <div class="flex flex-wrap items-center gap-x-1"></div>
            </div>
          </div>
        </CommandItem>
      {/if}
    </CommandContent>
  </Command>
</Popover>
