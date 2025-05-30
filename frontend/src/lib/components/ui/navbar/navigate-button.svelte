<script lang="ts">
  import { Goto } from "$lib/kosui/goto";
  import type { NavigationAction } from "$lib/navigation-action";
  import { twMerge } from "tailwind-merge";
  import { getRegistryContext } from "../command-palette";

  type Props = {
    name: string;
    desktop?: boolean;
  };
  const { name, desktop = false }: Props = $props();

  const command = getRegistryContext();

  let action = $derived(command.get(name)) as NavigationAction;
</script>

{#if action && action.href && action.enabled()}
  <Goto
    variant="plain"
    shape="circle"
    href={action.href}
    title={action.title}
    class={twMerge("flex aspect-square", desktop && "max-sm:hidden")}
  >
    <action.icon size={20} />
  </Goto>
{/if}
