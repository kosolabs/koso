<script lang="ts">
  import { Portal } from "$lib/ui/portal";
  import { cn, flyAndScale } from "$lib/utils";
  import * as popper from "@floating-ui/dom";
  import type { Snippet } from "svelte";

  type Props = {
    children: Snippet;
    trigger: Snippet<[{ show: () => void; hide: () => void }]>;
    arrow?: boolean;
    visible?: boolean;
    delay?: number;
    placement?: popper.Placement;
    portalEl?: HTMLElement;
    class?: string;
  };

  let {
    children,
    trigger,
    visible = $bindable(false),
    delay = 500,
    placement = "top",
    portalEl,
    class: classes,
  }: Props = $props();

  let referenceEl: popper.ReferenceElement | undefined = $state();
  let tooltipEl: popper.FloatingElement | undefined = $state();
  let tooltipTimeout: number | undefined;

  function showTooltip() {
    tooltipTimeout = window.setTimeout(() => (visible = true), delay);
  }

  function hideTooltip() {
    window.clearTimeout(tooltipTimeout);
    visible = false;
  }

  async function update() {
    if (!referenceEl || !tooltipEl) return;
    const { x, y } = await popper.computePosition(referenceEl, tooltipEl, {
      placement,
      middleware: [
        popper.offset(6),
        popper.flip(),
        popper.shift({ padding: 5 }),
      ],
    });
    if (!referenceEl || !tooltipEl) return;
    Object.assign(tooltipEl.style, {
      left: `${x}px`,
      top: `${y}px`,
    });
  }

  $effect(() => {
    update();
  });
</script>

<div bind:this={referenceEl}>
  {#if trigger}
    {@render trigger({ show: showTooltip, hide: hideTooltip })}
  {/if}
</div>

{#snippet tooltip()}
  {#if visible}
    <div
      class={cn("absolute left-0 top-0 z-50 w-max overflow-hidden", classes)}
      role="tooltip"
      transition:flyAndScale
      bind:this={tooltipEl}
    >
      {#if children}
        {@render children()}
      {/if}
    </div>
  {/if}
{/snippet}

{#if !portalEl}
  {@render tooltip()}
{:else}
  <Portal target={portalEl}>
    {@render tooltip()}
  </Portal>
{/if}
