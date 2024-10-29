<script lang="ts">
  import * as Command from "$lib/components/ui/command";
  import * as Popover from "$lib/components/ui/popover";
  import { Shortcut } from "$lib/shortcuts";
  import type { Snippet } from "svelte";

  type Props = {
    children: Snippet;
    visible: boolean;
    closeFocus: HTMLElement;
  };
  let { children, visible = $bindable(false), closeFocus }: Props = $props();
</script>

<Popover.Root bind:open={visible} {closeFocus} portal={null}>
  <Popover.Trigger>{@render children()}</Popover.Trigger>
  <Popover.Content
    onkeydown={(event) => {
      event.stopPropagation();
      if (Shortcut.CANCEL.matches(event)) {
        visible = false;
      }
    }}
  >
    <Command.Root>
      <Command.Input placeholder="Type a command or search..." />
      <Command.List>
        <Command.Empty>No results found.</Command.Empty>
        <Command.Group heading="Suggestions">
          <Command.Item>Calendar</Command.Item>
          <Command.Item>Search Emoji</Command.Item>
          <Command.Item>Calculator</Command.Item>
        </Command.Group>
        <Command.Separator />
        <Command.Group heading="Settings">
          <Command.Item>Profile</Command.Item>
          <Command.Item>Billing</Command.Item>
          <Command.Item>Settings</Command.Item>
        </Command.Group>
      </Command.List>
    </Command.Root>
  </Popover.Content>
</Popover.Root>
