<script lang="ts">
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import { ResponsiveText } from "$lib/components/ui/responsive-text";
  import { Shortcut } from "$lib/shortcuts";
  import type { Status } from "$lib/yproxy";
  import { TaskStatusIcon } from ".";

  const statuses: Status[] = ["Not Started", "In Progress", "Done"];

  type Props = {
    value: Status | null;
    statusTime: Date | null;
    onselect: (status: Status) => void;
  };

  let { value = $bindable(), statusTime, onselect }: Props = $props();

  let open: boolean = $state(false);

  function select(status: Status) {
    value = status;
    onselect(status);
  }
</script>

<DropdownMenu.Root bind:open>
  <DropdownMenu.Trigger
    class="flex items-center gap-2"
    title={(value || "Not Started") +
      (statusTime ? " - " + statusTime.toLocaleString() : "")}
  >
    <TaskStatusIcon status={value} />
    <ResponsiveText>{value || "Not Started"}</ResponsiveText>
  </DropdownMenu.Trigger>
  <DropdownMenu.Portal>
    <div
      role="none"
      onkeydown={(event) => {
        if (Shortcut.CANCEL.matches(event)) {
          open = false;
        }
        event.stopPropagation();
      }}
    >
      <DropdownMenu.Content portalProps={{ disabled: true }}>
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
  </DropdownMenu.Portal>
</DropdownMenu.Root>
