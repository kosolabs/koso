<script lang="ts">
  import * as Command from "$lib/components/ui/command";
  import * as Popover from "$lib/components/ui/popover";
  import type { Koso } from "$lib/koso.svelte";
  import { Shortcut } from "$lib/shortcuts";
  import { getContext, type Snippet } from "svelte";

  type Props = {
    children: Snippet;
    visible: boolean;
    closeFocus: HTMLElement;
  };
  let { children, visible = $bindable(false), closeFocus }: Props = $props();

  const koso = getContext<Koso>("koso");

  let tasks = Object.values(koso.toJSON());
  let query = $state("");
  let results = $derived(
    tasks
      .filter(
        (task) =>
          task.id !== "root" &&
          (task.name.toLocaleLowerCase().includes(query.toLocaleLowerCase()) ||
            task.num.startsWith(query)),
      )
      .sort((t1, t2) => t2.children.length - t1.children.length),
  );
</script>

<Popover.Root bind:open={visible} {closeFocus} portal={null}>
  <Popover.Trigger>{@render children()}</Popover.Trigger>
  <Popover.Content
    class="w-auto max-w-2xl"
    onkeydown={(event) => {
      event.stopPropagation();
      if (Shortcut.CANCEL.matches(event)) {
        visible = false;
      }
    }}
  >
    <Command.Root shouldFilter={false}>
      <Command.Input
        placeholder="Type a task name or number to search..."
        bind:value={query}
      />
      <Command.List>
        <Command.Empty>No results found.</Command.Empty>
        {#each results as result (result.id)}
          <Command.Item>{result.name}</Command.Item>
        {/each}
      </Command.List>
    </Command.Root>
  </Popover.Content>
</Popover.Root>
