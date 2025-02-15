<script lang="ts">
  import { Button } from "$lib/kosui/button";
  import { PlainTooltip } from "$lib/kosui/tooltip";
  import { Shortcut } from "$lib/shortcuts";

  const shortcut = new Shortcut({ key: "Enter", shift: true });

  let ref: HTMLElement | undefined = $state();
  let tooltip: PlainTooltip | undefined = $state();
</script>

<div class="flex flex-wrap gap-2">
  <Button
    bind:ref
    onmouseenter={() => tooltip?.show()}
    onmouseleave={() => tooltip?.hide()}
    onfocus={() => tooltip?.show()}
    onblur={() => tooltip?.hide()}
  >
    Fully Controlled
  </Button>
  <PlainTooltip bind:this={tooltip} trigger={ref} arrow>
    <div class="flex items-center gap-2">
      I'm a fully controlled tooltip
      <div class="font-bold">
        {shortcut.toString()}
      </div>
    </div>
  </PlainTooltip>

  <PlainTooltip arrow>
    {#snippet trigger(ref, props)}
      <Button bind:ref={ref.value} {...props}>Render Delegated</Button>
    {/snippet}
    <div class="flex items-center gap-2">
      I'm a render delegated tooltip
      <div class="font-bold">
        {shortcut.toString()}
      </div>
    </div>
  </PlainTooltip>

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
<div class="h-[1024px] w-[1024px]"></div>
