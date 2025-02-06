<script lang="ts">
  import type { Icon } from "lucide-svelte";
  import type { Snippet } from "svelte";
  import type { HTMLDialogAttributes } from "svelte/elements";
  import { cn } from "../utils";

  type Props = {
    ref?: HTMLDialogElement;
    onSelect?: (result: string) => void;
    class?: string;
    icon?: typeof Icon;
    title?: string;
    children: Snippet;
    actions?: Snippet;
  } & HTMLDialogAttributes;
  let {
    ref = $bindable(),
    onSelect = () => {},
    class: className,
    icon: IconComponent,
    title,
    children,
    actions,
    ...props
  }: Props = $props();

  function handleToggle(event: ToggleEvent) {
    if (!ref) {
      console.error("ref should be defined!");
      return;
    }
    if (event.newState === "closed") {
      onSelect(ref.returnValue);
    } else {
      ref.returnValue = "";
    }
  }
</script>

<dialog
  bind:this={ref}
  ontoggle={(event) => handleToggle(event)}
  class={cn(
    "bg-background dialog-animation m-auto max-w-[min(calc(100%-1em),36em)] min-w-[18em] overflow-hidden rounded-lg border p-6 shadow-lg",
    className,
  )}
  {...props}
>
  <form method="dialog">
    <div class={cn("flex flex-col gap-4")}>
      {#if IconComponent}
        <IconComponent class="mx-auto" />
      {/if}
      {#if title}
        <div class={cn("text-xl", IconComponent ? "text-center" : "")}>
          {title}
        </div>
      {/if}
      <div>
        {@render children()}
      </div>
      {#if actions}
        <div class="mt-2 flex place-content-end gap-2">
          {@render actions()}
        </div>
      {/if}
    </div>
  </form>
</dialog>

<style>
  .dialog-animation {
    transition:
      display 0.15s allow-discrete,
      overlay 0.15s allow-discrete;

    animation: close-dialog 0.15s forwards;
    &[open] {
      animation: open-dialog 0.15s forwards;
    }

    &::backdrop {
      animation: close-backdrop 0.15s forwards;
    }
    &[open]::backdrop {
      animation: open-backdrop 0.15s forwards;
    }
  }

  @keyframes open-backdrop {
    from {
      background: rgba(0, 0, 0, 0);
      backdrop-filter: blur(0px);
    }
    to {
      background: rgba(0, 0, 0, 0.1);
      backdrop-filter: blur(2px);
    }
  }

  @keyframes close-backdrop {
    from {
      background: rgba(0, 0, 0, 0.1);
      backdrop-filter: blur(2px);
    }
    to {
      background: rgba(0, 0, 0, 0);
      backdrop-filter: blur(0px);
    }
  }

  @keyframes open-dialog {
    from {
      opacity: 0;
      scale: 0.95;
    }
    to {
      opacity: 1;
      scale: 1;
    }
  }

  @keyframes close-dialog {
    from {
      opacity: 1;
      scale: 1;
    }
    to {
      opacity: 0;
      scale: 0.95;
    }
  }
</style>
