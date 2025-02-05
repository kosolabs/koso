<script module lang="ts">
  import type { Snippet } from "svelte";

  type Resolver = (ok: boolean) => void;

  type Dialog = {
    message: Snippet | string;
    title?: string;
    cancelText?: string;
    acceptText?: string;
    icon?: typeof Icon;
  };
  let dialog: (Dialog & { resolve: Resolver }) | null = $state(null);
  let dialogEl: HTMLDialogElement | undefined = $state();

  export function confirm(newDialog: Dialog): Promise<boolean> {
    dialogEl?.showModal();

    return new Promise((resolve) => {
      dialog = {
        resolve,
        ...newDialog,
      };
    });
  }
</script>

<script lang="ts">
  import type { Icon } from "lucide-svelte";
  import { Button } from "../button";
  import { cn } from "../utils";

  function handleToggle(event: ToggleEvent, resolve?: (ok: boolean) => void) {
    if (!dialogEl) {
      return;
    }

    if (event.newState === "closed") {
      resolve?.(dialogEl.returnValue === "ok");
    } else {
      dialogEl.returnValue = "";
    }
  }

  function accept() {
    dialogEl?.close("ok");
  }

  function cancel() {
    dialogEl?.close();
  }
</script>

<dialog
  bind:this={dialogEl}
  ontoggle={(event) => handleToggle(event, dialog?.resolve)}
  class={cn(
    "bg-background dialog-animation m-auto max-w-[min(calc(100%-1em),36em)] min-w-[18em] overflow-hidden rounded-lg border p-6 shadow-lg",
  )}
>
  {#if dialog}
    {@const {
      message,
      icon: Icon,
      title,
      cancelText = "Cancel",
      acceptText = "Accept",
    } = dialog}
    <div class={cn("flex flex-col gap-4")}>
      {#if Icon}
        <Icon class="mx-auto" />
      {/if}
      {#if title}
        <div class={cn("text-xl", Icon ? "text-center" : "")}>{title}</div>
      {/if}
      <div>
        {#if typeof message === "function"}
          {@render message()}
        {:else}
          {message}
        {/if}
      </div>
      <div class="mt-2 flex flex-row-reverse place-content-end gap-2">
        <Button onclick={accept}>{acceptText}</Button>
        <Button variant="outline" onclick={cancel}>{cancelText}</Button>
      </div>
    </div>
  {/if}
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
