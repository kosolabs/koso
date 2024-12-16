<script lang="ts">
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import { ResponsiveText } from "$lib/components/ui/responsive-text";
  import { Shortcut } from "$lib/shortcuts";
  import type { Status } from "$lib/yproxy";
  import { tick } from "svelte";
  import { TaskStatusIcon } from ".";

  const statuses: Status[] = ["Not Started", "In Progress", "Done"];

  type Props = {
    value: Status | null;
    open: boolean;
    statusTime: Date | null;
    editable: boolean;
    onOpenChange?: (open: boolean) => void;
    onSelect?: (status: Status) => void;
  };

  let {
    value = $bindable(),
    open = $bindable(),
    statusTime,
    onOpenChange,
    onSelect,
    editable = true,
  }: Props = $props();

  function handleOpenChange(o: boolean) {
    onOpenChange?.(o);
    tick().then(() => (open = o));
  }

  function select(status: Status) {
    value = status;
    onSelect?.(status);
  }
</script>

{#if editable}
  <DropdownMenu.Root controlledOpen {open} onOpenChange={handleOpenChange}>
    <DropdownMenu.Trigger
      class="flex items-center gap-2"
      title={(value || "Not Started") +
        (statusTime ? " - " + statusTime.toLocaleString() : "")}
    >
      <TaskStatusIcon status={value} />
      <ResponsiveText>{value || "Not Started"}</ResponsiveText>
    </DropdownMenu.Trigger>
    <div
      role="none"
      onkeydown={(event) => {
        if (Shortcut.CANCEL.matches(event)) {
          open = false;
        }
        event.stopPropagation();
      }}
    >
      <DropdownMenu.Content
        portalProps={{ disabled: true }}
        preventScroll={false}
      >
        {#each statuses as status}
          <DropdownMenu.Item
            class="flex items-center gap-2 rounded p-2"
            onSelect={() => select(status)}
          >
            <TaskStatusIcon {status} />
            {status}
          </DropdownMenu.Item>
        {/each}
      </DropdownMenu.Content>
    </div>
  </DropdownMenu.Root>
{:else}
  <div
    class="flex items-center gap-2"
    title={(value || "Not Started") +
      (statusTime ? " - " + statusTime.toLocaleString() : "")}
  >
    <TaskStatusIcon status={value} />
    <ResponsiveText>{value || "Not Started"}</ResponsiveText>
  </div>
{/if}
