<script lang="ts">
  import { Button } from "$lib/kosui/button";
  import { Shortcut } from "$lib/kosui/shortcut";
  import { Tooltip, TooltipTrigger } from "$lib/kosui/tooltip";

  const shortcut = new Shortcut({ key: "Enter", shift: true });

  let el: HTMLElement | undefined = $state();
  let tooltip: Tooltip | undefined = $state();
</script>

<div class="flex flex-wrap gap-2">
  <Button
    bind:el
    onmouseenter={() => tooltip?.show()}
    onmouseleave={() => tooltip?.hide()}
    onfocus={() => tooltip?.show()}
    onblur={() => tooltip?.hide()}
  >
    Fully Controlled
  </Button>
  <Tooltip bind:this={tooltip} anchorEl={el} arrow>
    <div class="flex items-center gap-2">
      I'm a fully controlled tooltip
      <div class="font-bold">
        {shortcut.toString()}
      </div>
    </div>
  </Tooltip>

  <Tooltip arrow>
    {#snippet trigger(props)}
      <Button {...props}>Render Delegated</Button>
    {/snippet}
    <div class="flex items-center gap-2">
      I'm a render delegated tooltip
      <div class="font-bold">
        {shortcut.toString()}
      </div>
    </div>
  </Tooltip>

  <Tooltip arrow>
    {#snippet trigger(props)}
      <TooltipTrigger {...props} class="rounded-m3 border p-1">
        Tooltip Trigger
      </TooltipTrigger>
    {/snippet}
    Uses a TooltipTrigger component
  </Tooltip>

  <Tooltip arrow>
    {#snippet trigger({ ref, ...props })}
      <div use:ref {...props} class="rounded-m3 border p-1">Just a Div</div>
    {/snippet}
    Trigger is just a styled div
  </Tooltip>

  <Button tooltip="I'm a tooltip prop">Tooltip Prop</Button>

  <Button>
    Tooltip Snippet
    {#snippet tooltip()}
      <div class="flex items-center gap-2">
        I'm a tooltip snippet
        <div class="font-bold">
          {shortcut.toString()}
        </div>
      </div>
    {/snippet}
  </Button>
</div>
