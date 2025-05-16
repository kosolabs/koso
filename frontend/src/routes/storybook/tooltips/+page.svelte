<script lang="ts">
  import { Button } from "$lib/kosui/button";
  import { Shortcut } from "$lib/kosui/shortcut";
  import { Tooltip } from "$lib/kosui/tooltip";

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

  <Tooltip click arrow>
    {#snippet trigger(props)}
      <Button {...props}>Show on Click</Button>
    {/snippet}
    <div class="flex items-center gap-2">
      I'm a render delegated tooltip
      <div class="font-bold">
        {shortcut.toString()}
      </div>
    </div>
  </Tooltip>

  <Tooltip arrow>
    {#snippet trigger({ ref, ...props })}
      <div use:ref {...props} class="rounded border p-1">Just a Div</div>
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

<div class="mt-20 flex flex-wrap gap-2">
  <Tooltip open arrow>
    {#snippet trigger(props)}
      <Button {...props}>Tooltip Button Trigger</Button>
    {/snippet}
    Tooltip with arrow
  </Tooltip>

  <Tooltip open>
    {#snippet trigger(props)}
      <Button {...props}>Tooltip Button Trigger</Button>
    {/snippet}
    Tooltip without arrow
  </Tooltip>

  <Tooltip open>
    {#snippet trigger(props)}
      <Button {...props}>Tooltip Button Trigger</Button>
    {/snippet}
    <div class="w-40">
      <div>Rich Title</div>
      <div>I'm a plain tooltip with a longish body that should wrap.</div>
    </div>
  </Tooltip>
</div>
