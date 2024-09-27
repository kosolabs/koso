<script lang="ts">
  import { Button, type ButtonProps } from "$lib/components/ui/button";
  import * as Tooltip from "$lib/components/ui/tooltip/index.js";

  type Props = ButtonProps & {
    // TODO: Use Component once lucide-svelte exports a Svelte 5 compatible type
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    icon: any;
    title: string;
  };
  const { icon: Icon, title, ...props }: Props = $props();
</script>

<Tooltip.Root>
  <Tooltip.Trigger asChild let:builder>
    <Button builders={[builder]} variant="ghost" size="sm" {...props}>
      <Icon class="w-4 sm:me-2" />
      <div class="text-xs max-sm:hidden">{title}</div>
    </Button>
  </Tooltip.Trigger>
  <Tooltip.Content>
    {title}
  </Tooltip.Content>
</Tooltip.Root>
