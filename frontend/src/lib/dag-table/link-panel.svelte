<script lang="ts">
  import { parseChipProps, type ChipProps } from "$lib/components/ui/chip";
  import { Chip } from "$lib/kosui/chip";
  import {
    Command,
    CommandDivider,
    CommandInput,
    CommandItem,
  } from "$lib/kosui/command";
  import { Popover } from "$lib/kosui/popover";
  import { Shortcut } from "$lib/kosui/shortcut";
  import { ToggleButton, ToggleGroup } from "$lib/kosui/toggle";
  import { match } from "$lib/utils";
  import { Clipboard, Network, SearchIcon } from "lucide-svelte";
  import { getContext } from "svelte";
  import { compareTasks, type Koso, type Node } from ".";

  type Mode = "link" | "block";

  type Props = {
    open: boolean;
    mode?: Mode;
    closeFocus?: HTMLElement;
    node: Node;
  };
  let {
    open = $bindable(false),
    mode = $bindable("link"),
    node,
  }: Props = $props();

  const koso = getContext<Koso>("koso");

  let anchorEl: HTMLElement | undefined = $state();
  let query = $state("");
  let tasks = $derived(
    open
      ? koso.tasks
          .filter((task) => match(task.num, query) || match(task.name, query))
          .filter((task) => {
            if (mode === "link") {
              return koso.canLink(node.name, task.id);
            } else {
              return koso.canLink(task.id, node.name);
            }
          })
          .sort((t1, t2) => compareTasks(t1, t2, koso))
          .slice(0, 50)
      : [],
  );

  function link(taskId: string) {
    if (mode === "link") {
      koso.link(node.name, taskId);
    } else {
      koso.link(taskId, node.name);
    }
    query = "";
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

<div bind:this={anchorEl} class="absolute left-[calc(100%/2)] h-6"></div>

<Popover
  bind:open
  {anchorEl}
  placement="bottom"
  class="shadow-m3-shadow/20 bg-m3-surface-container-high h-[min(40%,24em)] w-[min(calc(100%-1em),36em)] rounded-lg border shadow"
  onkeydown={(event) => {
    if (!Shortcut.ESCAPE.matches(event)) {
      event.stopImmediatePropagation();
    }
  }}
>
  <div class="flex place-content-center p-1">
    <ToggleGroup bind:value={mode}>
      <ToggleButton value="link">Link to</ToggleButton>
      <ToggleButton value="block">Block on</ToggleButton>
    </ToggleGroup>
  </div>
  <hr />
  <Command class="flex h-full flex-col">
    <div class="flex items-center px-2">
      <SearchIcon size={16} />
      <CommandInput
        autofocus
        bind:value={query}
        placeholder="Search by task name or number..."
      />
    </div>
    <CommandDivider />
    <div class="h-full overflow-scroll">
      {#if tasks.length > 0}
        {#each tasks as task (task.id)}
          <CommandItem
            class="table-row"
            onSelect={() => link(task.id)}
            aria-label="Task {task.id} Command Item"
          >
            <div class="table-cell rounded-l px-2 py-2 align-middle">
              <div class="flex items-center gap-1" title="Task Number">
                <Clipboard size={16} />
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
                <Network size={16} />
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
    </div>
  </Command>
</Popover>
