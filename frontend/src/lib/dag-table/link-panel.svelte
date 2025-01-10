<script lang="ts">
  import * as Command from "$lib/components/ui/command";
  import * as Popover from "$lib/components/ui/popover";
  import { Shortcut } from "$lib/shortcuts";
  import { match } from "$lib/utils";
  import { Clipboard, Network } from "lucide-svelte";
  import { getContext } from "svelte";
  import type { Koso, Node } from ".";
  import {
    Chip,
    parseChipProps,
    type ChipProps,
  } from "$lib/components/ui/chip";

  type Props = {
    open: boolean;
    closeFocus?: HTMLElement;
    node: Node;
  };
  let { open = $bindable(false), closeFocus, node }: Props = $props();

  const koso = getContext<Koso>("koso");

  let query = $state("");
  let tasks = $derived(
    open
      ? koso
          .getTasks()
          .filter((task) => match(task.num, query) || match(task.name, query))
          .filter((task) => koso.canLink(node.name, task.id))
          .sort((t1, t2) => t2.children.length - t1.children.length)
          .slice(0, 50)
      : [],
  );

  function link(taskId: string) {
    koso.link(node.name, taskId);
    query = "";
    open = false;
  }

  function getTags(taskId: string): ChipProps[] {
    return (
      koso.parents
        .get(taskId)
        ?.map((parent) => koso.getTask(parent))
        .filter((parent) => parent.name.length > 0)
        .map((parent) => parseChipProps(parent.name)) || []
    );
  }
</script>

<Popover.Root bind:open>
  <Popover.Trigger class="absolute left-[calc(100%/2)] h-6" />
  <div
    role="none"
    onkeydown={(event) => {
      if (Shortcut.CANCEL.matches(event)) {
        open = false;
      }
      event.stopPropagation();
    }}
  >
    <Popover.Content
      class="w-[calc(100%)] max-w-[calc(100vw-1em)]"
      portalProps={{ disabled: true }}
      onCloseAutoFocus={(event) => {
        event.preventDefault();
        closeFocus?.focus();
      }}
    >
      <Command.Root shouldFilter={false}>
        <Command.Input
          placeholder="Search by task name or number..."
          bind:value={query}
        />
        <Command.List>
          <Command.Empty>No tasks found.</Command.Empty>
          {#each tasks as task (task.id)}
            <Command.Item
              class="table-row"
              onSelect={() => link(task.id)}
              aria-label="Task {task.id} Command Item"
            >
              <div class="table-cell rounded-l px-2 align-middle">
                <div class="flex items-center gap-1 py-2" title="Task Number">
                  <Clipboard size={16} />
                  {task.num}
                </div>
              </div>
              <div class="table-cell w-full px-2 align-middle">
                <div class="flex items-center" title="Task Name">
                  {task.name || "Untitled task"}
                </div>
              </div>
              <div class="table-cell text-nowrap rounded-r px-2 align-middle">
                <div class="flex items-center gap-1" title="Subtasks">
                  {task.children.length}
                  <Network size={16} />
                </div>
              </div>
              <div class="table-cell text-nowrap rounded-r px-2 align-middle">
                <div class="flex items-center gap-1" title="Status">
                  {koso.getStatus(task.id)}
                </div>
              </div>
              <div class="table-cell text-nowrap rounded-r px-2 align-middle">
                <div class="flex items-center gap-1" title="Tags">
                  <div class="flex flex-wrap items-center gap-x-1">
                    {#each getTags(task.id) as tag}
                      <Chip {...tag} />
                    {/each}
                  </div>
                </div>
              </div>
            </Command.Item>
          {/each}
        </Command.List>
      </Command.Root>
    </Popover.Content>
  </div>
</Popover.Root>
