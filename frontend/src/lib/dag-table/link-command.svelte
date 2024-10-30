<script lang="ts">
  import * as Command from "$lib/components/ui/command";
  import * as Popover from "$lib/components/ui/popover";
  import type { Koso, Node } from "$lib/koso.svelte";
  import { Shortcut } from "$lib/shortcuts";
  import { Grip } from "lucide-svelte";
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
    Object.values(koso.graph)
      .filter(
        (task) =>
          task.name.toLocaleLowerCase().includes(query.toLocaleLowerCase()) ||
          task.num.startsWith(query),
      )
      .filter((task) => task.id !== "root")
      .filter((task) => koso.canLink(node, task.id))
      .sort((t1, t2) => t2.children.length - t1.children.length),
  );

  function link(taskId: string) {
    koso.linkNode(node, taskId, koso.getChildCount(taskId));
    open = false;
  }
</script>

<Popover.Root bind:open {closeFocus} portal={null}>
  <Popover.Trigger class="h-auto"></Popover.Trigger>
  <Popover.Content
    class="w-[calc(100%-1em)] max-w-2xl"
    onkeydown={(event) => {
      event.stopPropagation();
      if (Shortcut.CANCEL.matches(event)) {
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
          <Command.Item class="flex" onSelect={() => link(task.id)}>
            <Grip size={16} />
            <div class="w-10 pl-1">{task.num}</div>
            <div class="w-full">{task.name || "Untitled task"}</div>
            <div class="ml-auto text-nowrap">
              ({task.children.length} Children)
            </div>
          </Command.Item>
        {/each}
      </Command.List>
    </Command.Root>
  </Popover.Content>
</Popover.Root>
