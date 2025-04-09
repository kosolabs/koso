<script module lang="ts">
  import { auth } from "$lib/auth.svelte";
  import { parseChipProps, type ChipProps } from "$lib/components/ui/chip";
  import { Chip } from "$lib/kosui/chip";
  import {
    Command,
    CommandContent,
    CommandDivider,
    CommandItem,
    CommandSearch,
  } from "$lib/kosui/command";
  import { mergeComponentProps } from "$lib/kosui/merge-props";
  import { Popover, type PopoverProps } from "$lib/kosui/popover";
  import { Shortcut } from "$lib/kosui/shortcut";
  import { ToggleButton, ToggleGroup } from "$lib/kosui/toggle";
  import { match } from "$lib/utils";
  import type { YTaskProxy } from "$lib/yproxy";
  import { Clipboard, Network } from "lucide-svelte";
  import { getContext } from "svelte";
  import { compareTasks, type Koso } from ".";
  import { TaskLinkage } from "./koso.svelte";

  export type Mode = "link" | "block";

  export type LinkPanelProps = {
    open: boolean;
    mode?: Mode;
    task: YTaskProxy;
  } & Omit<PopoverProps, "children">;
</script>

<script lang="ts">
  let {
    open = $bindable(false),
    mode = $bindable("link"),
    task,
    anchorEl,
    ...restProps
  }: LinkPanelProps = $props();

  const koso = getContext<Koso>("koso");

  let query = $state("");
  let setStatusBlocked: boolean = $state(true);
  let tasks = $derived(
    open
      ? koso.tasks
          .filter((t) => match(t.num, query) || match(t.name, query))
          .filter((t) => {
            if (mode === "link") {
              return koso.canLink(TaskLinkage.create(t.id, task.id));
            } else {
              return koso.canLink(TaskLinkage.create(task.id, t.id));
            }
          })
          .sort((t1, t2) => compareTasks(t1, t2, koso))
          .slice(0, 50)
      : [],
  );

  function link(taskId: string) {
    if (mode === "link") {
      koso.link(TaskLinkage.create(taskId, task.id));
    } else if (mode === "block") {
      koso.doc.transact(() => {
        koso.link(TaskLinkage.create(task.id, taskId));
        if (setStatusBlocked) {
          koso.setKind(task.id, "Task");
          koso.setTaskStatus(task.id, "Blocked", auth.user);
        }
      });
    } else {
      throw new Error(`Unknown mode: ${mode}`);
    }
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
    <CommandSearch
      bind:value={query}
      placeholder={mode === "link"
        ? "Link this task to..."
        : "Block this task on..."}
    />
    <CommandDivider />
    <CommandContent>
      {#if tasks.length > 0}
        {#each tasks as task (task.id)}
          <CommandItem
            class="table-row"
            onSelect={() => link(task.id)}
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
                  {#each getTags(task.id) as { title, description }}
                    <Chip title={description}>{title}</Chip>
                  {/each}
                </div>
              </div>
            </div>
          </CommandItem>
        {/each}
      {:else}
        <div class="text-center">No results found.</div>
      {/if}
    </CommandContent>
  </Command>
</Popover>
