<script lang="ts">
  import {
    ActionIds,
    getRegistryContext,
  } from "$lib/components/ui/command-palette";
  import { MoreVertical } from "@lucide/svelte";
  import {
    Menu,
    MenuActions,
    MenuContent,
    MenuTriggerButton,
    type MenuTriggerButtonProps,
  } from "kosui";

  type Props = {} & MenuTriggerButtonProps;
  let { ...restProps }: Props = $props();

  const command = getRegistryContext();

  const actions = $derived(
    [
      ActionIds.InsertSubtask,
      ActionIds.Indent,
      ActionIds.Undent,
      ActionIds.MoveUp,
      ActionIds.MoveDown,
      ActionIds.MoveToStart,
      ActionIds.MoveToEnd,

      ActionIds.Delete,
      ActionIds.CopyTaskInfo,
      ActionIds.CopyTaskLink,
      ActionIds.Archive,
      ActionIds.Unarchive,

      ActionIds.Link,
      ActionIds.Block,

      ActionIds.DashView,
      ActionIds.BreakDown,
      ActionIds.GenerateDesignDoc,
    ]
      .map((id) => command.get(id))
      .filter((action) => action !== undefined)
      .filter((action) => action.enabled()),
  );
</script>

<Menu>
  <MenuTriggerButton
    title="Task actions"
    variant="plain"
    shape="circle"
    icon={MoreVertical}
    {...restProps}
  />
  <MenuContent>
    <MenuActions {actions} />
  </MenuContent>
</Menu>
