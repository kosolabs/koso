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
          "group toast group-[.toaster]:bg-background group-[.toaster]:text-foreground group-[.toaster]:border-border group-[.toaster]:shadow-lg",
        description: "group-[.toast]:text-muted-foreground",
        actionButton:
          "group-[.toast]:bg-primary group-[.toast]:text-primary-foreground",
        cancelButton:
          "group-[.toast]:bg-muted group-[.toast]:text-muted-foreground",
      },
    }}
    {...restProps}
  />
</div>
