<script lang="ts">
  import * as Tooltip from "$lib/components/ui/tooltip";
  import { cn } from "$lib/utils";
  import { Github, ToyBrick, type Icon } from "lucide-svelte";

  type Props = {
    kind: string;
  };
  let { kind }: Props = $props();

  function getManagedTaskIcon(kind: string): typeof Icon {
    if (kind.startsWith("github")) {
      return Github;
    }
    console.warn(`No icon registered for kind ${kind}. Add one!`);
    return ToyBrick;
  }

  function getManagedTaskName(kind: string): string {
    if (kind.startsWith("github")) {
      return "GitHub";
    }
    return "Untitled";
  }

  let ManagedTaskIcon = getManagedTaskIcon(kind);
</script>

<Tooltip.Provider>
  <Tooltip.Root>
    <Tooltip.Trigger>
      {#snippet child({ props })}
        <div {...props} class={cn("max-w-4 min-w-4")}>
          <ManagedTaskIcon size={16} class="text-foreground" />
        </div>
      {/snippet}
    </Tooltip.Trigger>
    <Tooltip.Portal>
      <Tooltip.Content>
        <Tooltip.Arrow class="z-50" />
        This task is managed by the {getManagedTaskName(kind)} plugin.
      </Tooltip.Content>
    </Tooltip.Portal>
  </Tooltip.Root>
</Tooltip.Provider>
