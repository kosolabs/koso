<script lang="ts">
  import {
    ActionIds,
    getRegistryContext,
  } from "$lib/components/ui/command-palette";
  import {
    Menu,
    MenuContent,
    MenuDivider,
    MenuHeader,
    MenuItem,
    MenuTriggerButton,
    type MenuTriggerButtonProps,
  } from "$lib/kosui/menu";
  import { MoreVertical } from "lucide-svelte";

  type Props = {} & MenuTriggerButtonProps;
  let { ...restProps }: Props = $props();

  const command = getRegistryContext();

  type Section = {
    heading: string;
    actions: string[];
  }[];

  const menu: Section = [
    {
      heading: "Actions",
      actions: [
        ActionIds.Indent,
        ActionIds.Undent,
        ActionIds.InsertSubtask,
        ActionIds.Delete,
        ActionIds.CopyTaskInfo,
        ActionIds.CopyTaskLink,
      ],
    },
    {
      heading: "Reorder",
      actions: ["MoveUp", "MoveDown", "MoveToStart", "MoveToEnd"],
    },
    {
      heading: "Linking",
      actions: ["Link", "Block"],
    },
  ];

  let sections = $derived(
    menu
      .map((section) => {
        return {
          heading: section.heading,
          actions: section.actions
            .map((id) => command.get(id))
            .filter((action) => action !== undefined)
            .filter((action) => action.enabled()),
        };
      })
      .filter((section) => section.actions.length > 0),
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
    {#each sections as section, index (section)}
      {#if section.actions.length > 0}
        <MenuHeader>{section.heading}</MenuHeader>
        {#each section.actions as action (action)}
          <MenuItem
            onSelect={action.callback}
            disabled={!action.enabled()}
            title={action.description}
          >
            {action.name}
            {#if action.shortcut}
              <div class="ml-auto pl-2 text-xs">
                {action.shortcut.toString()}
              </div>
            {/if}
          </MenuItem>
        {/each}
        {#if index < sections.length - 1}
          <MenuDivider />
        {/if}
      {/if}
    {/each}
  </MenuContent>
</Menu>
