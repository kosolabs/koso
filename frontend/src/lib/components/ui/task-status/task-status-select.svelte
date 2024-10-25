<script lang="ts">
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import { ResponsiveText } from "$lib/components/ui/responsive-text";
  import type { Status } from "$lib/koso.svelte";
  import { TaskStatusIcon } from ".";

  const statuses: Status[] = ["Not Started", "In Progress", "Done"];

  type Props = {
    value: Status | null;
    statusTime: Date | null;
    closeFocus: HTMLElement;
    onselect: (status: Status) => void;
  };
  let {
    value = $bindable(),
    statusTime,
    closeFocus,
    onselect,
  }: Props = $props();

  function select(status: Status) {
    value = status;
    onselect(status);
  }
</script>

<DropdownMenu.Root {closeFocus} portal={null}>
  <DropdownMenu.Trigger
    class="flex items-center gap-2"
    title={(value || "Not Started") +
      (statusTime ? " - " + statusTime.toLocaleString() : "")}
  >
    <TaskStatusIcon status={value} />
    <ResponsiveText>{value || "Not Started"}</ResponsiveText>
  </DropdownMenu.Trigger>
  <DropdownMenu.Content
    onkeydown={(event) => {
      event.stopPropagation();
    }}
  >
    {#each statuses as status}
      <DropdownMenu.Item
        class="flex items-center gap-2 rounded p-2"
        on:click={() => select(status)}
      >
        <TaskStatusIcon {status} />
        {status}
      </DropdownMenu.Item>
    {/each}
  </DropdownMenu.Content>
</DropdownMenu.Root>
