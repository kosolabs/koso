<script lang="ts">
  import { parseChipProps, type ChipProps } from "$lib/components/ui/chip";
  import { match } from "$lib/utils";
  import { Clipboard, Network } from "@lucide/svelte";
  import {
    Chip,
    Command,
    CommandContent,
    CommandDivider,
    CommandItem,
    CommandSearch,
    Modal,
  } from "kosui";
  import { compareTasks } from "./compare-tasks.svelte";
  import { getPlanningContext } from "./planning-context.svelte";

  type Props = {
    open: boolean;
    selected?: string;
  };
  let { open = $bindable(false), selected = $bindable() }: Props = $props();

  const planningCtx = getPlanningContext();
  const { koso } = planningCtx;

  let query = $state("");
  let tasks = $derived(
    open
      ? koso.tasks
          .filter((task) => task.id !== "root")
          .filter((task) => match(task.num, query) || match(task.name, query))
          .sort((t1, t2) => compareTasks(t1, t2, koso))
          .slice(0, 50)
      : [],
  );

  function handleSelect(taskId: string) {
    selected = taskId;
    open = false;
    query = "";
  }

  function finalize() {
    if (selected) {
      planningCtx.select(selected);
    }
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

<Modal
  bind:open
  onoutroend={finalize}
  class="bg-m3-surface-container-high h-[min(40%,24em)] w-[min(calc(100%-1em),36em)] rounded-lg p-0"
>
  <Command>
    <CommandSearch bind:value={query} />
    <CommandDivider />
    <CommandContent>
      {#if tasks.length > 0}
        {#each tasks as task (task.id)}
          <CommandItem
            class="table-row"
            onSelect={() => handleSelect(task.id)}
            aria-label="Task {task.id} Search Item"
          >
            <div class="table-cell rounded-l px-2 py-2 align-middle">
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
        <div class="text-center">No results found.</div>
      {/if}
    </CommandContent>
  </Command>
</Modal>
