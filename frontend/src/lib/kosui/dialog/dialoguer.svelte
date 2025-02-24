<script module lang="ts">
  import type { Icon } from "lucide-svelte";
  import type { Snippet } from "svelte";
  import { DialogButton } from ".";
  import { type ButtonVariants } from "../button";
  import Dialog from "./dialog.svelte";

  type ButtonProps<T> = ButtonVariants & {
    text: string;
    value: T;
    default?: boolean;
  };

  let message: Snippet | string = $state("");
  let icon: typeof Icon | undefined = $state();
  let title: string | undefined = $state();
  let buttons: ButtonProps<unknown>[] = $state.raw([]);
  let resolve: (value: unknown) => void = $state(() => {});

  let open: boolean = $state(false);

  type ShowDialogProps<T> = {
    icon?: typeof Icon;
    title?: string;
    message: Snippet | string;
    buttons: ButtonProps<T>[];
  };

  export function show<T>(dialog: ShowDialogProps<T>): Promise<T> {
    ({ icon, title, message, buttons } = dialog);
    open = true;
    return new Promise<unknown>(
      (newResolve) => (resolve = newResolve),
    ) as Promise<T>;
  }

  type NoticeDialogProps = {
    icon?: typeof Icon;
    title?: string;
    message: Snippet | string;
    acceptText?: string;
  };

  export async function notice(dialog: NoticeDialogProps): Promise<void> {
    await show({
      icon: dialog.icon,
      title: dialog.title,
      message: dialog.message,
      buttons: [
        {
          text: dialog.acceptText ?? "OK",
          value: null,
          default: true,
        },
      ],
    });
  }

  type ConfirmDialogProps = {
    icon?: typeof Icon;
    title?: string;
    message: Snippet | string;
    cancelText?: string;
    acceptText?: string;
  };

  export async function confirm(dialog: ConfirmDialogProps): Promise<boolean> {
    return await show({
      icon: dialog.icon,
      title: dialog.title,
      message: dialog.message,
      buttons: [
        {
          text: dialog.cancelText ?? "Cancel",
          value: false,
          default: false,
        },
        {
          text: dialog.acceptText ?? "Accept",
          value: true,
          variant: "filled",
          default: true,
        },
      ],
    });
  }
</script>

<Dialog bind:open {icon} {title} onSelect={resolve}>
  {#if typeof message === "function"}
    {@render message()}
  {:else}
    {message}
  {/if}
  {#snippet actions(props)}
    {#each buttons as { value, variant, text, default: autofocus }}
      <DialogButton {value} {variant} {autofocus} {...props}>
        {text}
      </DialogButton>
    {/each}
  {/snippet}
</Dialog>
