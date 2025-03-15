<script module lang="ts">
  import { mode } from "mode-watcher";
  import { onMount } from "svelte";
  import {
    Toaster as Sonner,
    toast as sonner,
    type ToasterProps as SonnerProps,
  } from "svelte-sonner";

  let ref: HTMLDivElement | undefined = $state();

  function promote() {
    ref?.hidePopover();
    ref?.showPopover();
    return toast;
  }

  type ToastFunction = (...args: unknown[]) => void;
  type SonnerType = {
    [K in keyof typeof sonner]: (typeof sonner)[K];
  };

  export const toast = new Proxy<SonnerType>(sonner, {
    get(target: SonnerType, prop: keyof SonnerType) {
      if (prop in target && typeof target[prop] === "function") {
        return (...args: Parameters<ToastFunction>) => {
          promote();
          return (target[prop] as ToastFunction)(...args);
        };
      }
      return target[prop];
    },
  });
</script>

<script lang="ts">
  let restProps: SonnerProps = $props();

  onMount(() => {
    ref?.showPopover();
  });
</script>

<div bind:this={ref} popover="manual" class="transform-style-preserve-3d">
  <Sonner
    theme={$mode}
    class="toaster group"
    toastOptions={{
      classes: {
        toast:
          "group toast group-[.toaster]:bg-m3-surface-container group-[.toaster]:text-m3-on-surface group-[.toaster]:shadow-lg",
        description: "group-[.toast]:text-m3-on-secondary",
        actionButton:
          "group-[.toast]:bg-m3-primary group-[.toast]:text-m3-on-primary",
        cancelButton:
          "group-[.toast]:bg-m3-secondary group-[.toast]:text-m3-on-secondary",
      },
    }}
    {...restProps}
  />
</div>
