<script lang="ts">
  import * as Command from "$lib/components/ui/command";
  import * as Popover from "$lib/components/ui/popover";
  import type { Koso, Node } from "$lib/koso.svelte";
  import { Shortcut } from "$lib/shortcuts";
  import { match } from "$lib/utils";
  import { Clipboard, Network } from "lucide-svelte";
  import { getContext } from "svelte";

  type Props = {
    open: boolean;
    closeFocus: HTMLElement;
    node: Node;
  };
  let { open = $bindable(false), closeFocus, node }: Props = $props();

  const koso = getContext<Koso>("koso");

  let query = $state("");
  let tasks = $derived(
    open
      ? Array.from(koso.graph.tasks())
          .filter((task) => match(task.num, query) || match(task.name, query))
          .filter((task) => task.id !== "root")
          .filter((task) => koso.canLink(node.name, task.id))
          .sort((t1, t2) => t2.children.length - t1.children.length)
          .slice(0, 50)
      : [],
  );

  function link(taskId: string) {
    koso.linkTask(node.name, taskId, koso.getChildCount(taskId));
    query = "";
    open = false;
  }
</script>

<Popover.Root bind:open {closeFocus} portal={null}>
  <Popover.Trigger class="absolute left-[calc(100%/2)] h-6" />
  <Popover.Content
    class="w-[calc(100%-2em)] max-w-2xl"
    onkeydown={(event) => {
      event.stopPropagation();
      if (Shortcut.CANCEL.matches(event)) {
        query = "";
        open = false;
      }
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
            role="button"
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
          </Command.Item>
        {/each}
      </Command.List>
    </Command.Root>
  </Popover.Content>
</Popover.Root>
