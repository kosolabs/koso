<script lang="ts">
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import { ResponsiveText } from "$lib/components/ui/responsive-text";
  import type { Status } from "$lib/yproxy";
  import { tick } from "svelte";
  import { TaskStatusIcon } from ".";
  import { Shortcut } from "$lib/shortcuts";

  const statuses: Status[] = ["Not Started", "In Progress", "Done"];

  type Props = {
    value: Status | null;
    statusTime: Date | null;
    editable: boolean;
    onOpenChange?: (open: boolean) => void;
    onSelect?: (status: Status) => void;
  };

  let {
    value = $bindable(),
    statusTime,
    onOpenChange,
    onSelect,
    editable = true,
  }: Props = $props();

  function select(status: Status) {
    value = status;
    onSelect?.(status);
  }

  let open: boolean = $state(false);
</script>

{#if editable}
  <DropdownMenu.Root
    bind:open={
      () => open,
      (newOpen) => {
        onOpenChange?.(newOpen);
        tick().then(() => (open = newOpen));
      }
    }
  >
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
