<script lang="ts">
  import * as Command from "$lib/components/ui/command";
  import { Shortcut } from "$lib/shortcuts";
  import { cn, match } from "$lib/utils";
  import { Clipboard, Network } from "lucide-svelte";
  import { getContext } from "svelte";
  import type { Koso } from ".";

  type Props = {
    open: boolean;
  };
  let { open = $bindable(false) }: Props = $props();

  const koso = getContext<Koso>("koso");

  let query = $state("");
  let tasks = $derived(
    open
      ? koso
          .getTasks()
          .filter((task) => match(task.num, query) || match(task.name, query))
          .sort((t1, t2) => t2.children.length - t1.children.length)
          .slice(0, 50)
      : [],
  );

  function handleSelect(taskId: string) {
    open = false;
    query = "";
    koso.select(taskId);
  }
</script>

<Command.Dialog
  bind:open
  class={cn("")}
  shouldFilter={false}
  portalProps={{ disabled: true }}
  onCloseAutoFocus={(e) => e.preventDefault()}
  onkeydown={(event) => {
    if (Shortcut.CANCEL.matches(event)) {
      open = false;
    }
    event.stopPropagation();
  }}
>
  <Command.Input
    bind:value={query}
    placeholder="Search by task name or number..."
  />
  <Command.List>
    <Command.Empty>No results found.</Command.Empty>
    {#each tasks as task (task.id)}
      <Command.Item
        class="table-row"
        onSelect={() => handleSelect(task.id)}
        aria-label="Task {task.id} Search Item"
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
</Command.Dialog>
