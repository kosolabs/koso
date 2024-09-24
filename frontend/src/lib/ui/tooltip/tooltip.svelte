<script lang="ts">
  import { Portal } from "$lib/ui/portal";
  import { cn, flyAndScale } from "$lib/utils";
  import * as popper from "@floating-ui/dom";
  import type { Snippet } from "svelte";

  type Props = {
    children: Snippet;
    trigger: Snippet;
    arrow?: boolean;
    visible?: boolean;
    placement?: popper.Placement;
    portalEl?: HTMLElement;
  };

  let {
    children,
    trigger,
    visible = $bindable(false),
    placement = "top",
    portalEl,
  }: Props = $props();

  let referenceEl: popper.ReferenceElement | undefined = $state();
  let tooltipEl: popper.FloatingElement | undefined = $state();

  function showTooltip() {
    visible = true;
  }

  function hideTooltip() {
    visible = false;
  }

  function update() {
    if (!referenceEl || !tooltipEl) return;
    popper
      .computePosition(referenceEl, tooltipEl, {
        placement,
        middleware: [
          popper.offset(6),
          popper.flip(),
          popper.shift({ padding: 5 }),
        ],
      })
      .then(({ x, y }) => {
        if (!referenceEl || !tooltipEl) return;
        Object.assign(tooltipEl.style, {
          left: `${x}px`,
          top: `${y}px`,
        });
      });
  }

  $effect(() => {
    update();
  });
</script>

<span
  role="tooltip"
  bind:this={referenceEl}
  onmouseenter={showTooltip}
  onmouseleave={hideTooltip}
  onfocus={showTooltip}
  onblur={hideTooltip}
>
  {#if trigger}
    {@render trigger()}
  {/if}
</span>

{#snippet tooltip()}
  {#if visible}
    <div
      class={cn(
        "absolute left-0 top-0 z-50 w-max overflow-hidden rounded bg-primary p-1 text-sm text-primary-foreground",
      )}
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
