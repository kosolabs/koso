<script lang="ts">
  import { Button } from "$lib/kosui/button";
  import { Link } from "$lib/kosui/link";
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
  <Tooltip open click>
    {#snippet trigger(props)}
      <Button {...props}>Tooltip Button Trigger</Button>
    {/snippet}
    Tooltip without arrow
  </Tooltip>

  <Tooltip open click arrow>
    {#snippet trigger(props)}
      <Button {...props}>Tooltip Button Trigger</Button>
    {/snippet}
    Tooltip with arrow
  </Tooltip>

  <Tooltip open click>
    {#snippet trigger(props)}
      <Button {...props}>Tooltip Button Trigger</Button>
    {/snippet}
    <div class="w-40">
      <div>Title</div>
      <div>I'm a plain tooltip with a longish body that should wrap.</div>
    </div>
  </Tooltip>

  <Tooltip open click rich>
    {#snippet trigger(props)}
      <Button {...props}>Extra Wide Tooltip Button Trigger</Button>
    {/snippet}
    <div class="flex w-50 flex-col gap-2">
      <div>Rich Title</div>
      <div>
        I'm a rich tooltip with a <Link>link to a thing</Link> which should wrap.
      </div>
      <div class="flex gap-2">
        <Button>Action 1</Button>
        <Button>Action 2</Button>
      </div>
    </div>
  </Tooltip>

  <Tooltip open click rich arrow>
    {#snippet trigger(props)}
      <Button {...props}>Extra Wide Tooltip Button Trigger</Button>
    {/snippet}
    <div class="flex w-50 flex-col gap-2">
      <div>Rich Title</div>
      <div>
        I'm a rich tooltip with a <Link>link to a thing</Link> which should wrap.
        And I should have an arrow.
      </div>
      <div class="flex gap-2">
        <Button>Action 1</Button>
        <Button>Action 2</Button>
      </div>
    </div>
  </Tooltip>

  <Tooltip open click rich class="max-h-2/5 overflow-y-scroll">
    {#snippet trigger(props)}
      <Button {...props}>Extra Wide Tooltip Button Trigger</Button>
    {/snippet}
    <div class="flex w-50 flex-col gap-2 p-1">
      <div>Rich Title</div>
      <div>
        I'm a rich tooltip with a lot of content. It should exceed the height of
        the tooltip and provide a scrollbar.
      </div>
      <div>Here's a bunch of additional text.</div>
      <div>
        Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nullam auctor,
        nisl eget ultricies venenatis, nunc magna faucibus diam, nec bibendum
        libero lectus non turpis. Donec euismod, arcu vel pharetra ultrices,
        metus sapien fermentum nisi, at tincidunt velit magna vel tortor.
        Suspendisse potenti. Sed eleifend lacus sed justo tincidunt, vel congue
        odio facilisis. Vivamus vestibulum purus at erat faucibus, nec dictum
        nunc dignissim.
      </div>
      <div class="flex gap-2">
        <Button>Action 1</Button>
        <Button>Action 2</Button>
      </div>
    </div>
  </Tooltip>
</div>
