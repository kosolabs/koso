<script lang="ts">
  import { page } from "$app/state";
  import { Goto } from "$lib/components/ui/goto";
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

{#if action && action.href && action.enabled() && page.url.pathname !== action.href}
  <Goto
    variant="plain"
    shape="circle"
    href={action.href}
    title={action.name}
    class={twMerge("flex aspect-square", desktop && "max-sm:hidden")}
  >
    <action.icon size={20} />
  </Goto>
{/if}
