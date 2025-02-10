<script module lang="ts">
  import type { Icon } from "lucide-svelte";
  import type { Snippet } from "svelte";
  import { Button, type ButtonVariants } from "../button";
  import Dialog from "./dialog.svelte";

  type ButtonProps = ButtonVariants & {
    text: string;
    value: string;
    default: boolean;
  };

  let message: Snippet | string = $state("");
  let icon: typeof Icon | undefined = $state();
  let title: string | undefined = $state();
  let buttons: ButtonProps[] = $state.raw([]);
  let resolve: (value: string) => void = $state(() => {});

  let open: boolean = $state(false);

  type ShowDialogProps = {
    icon?: typeof Icon;
    title?: string;
    message: Snippet | string;
    buttons: ButtonProps[];
  };

  export function show(dialog: ShowDialogProps): Promise<string> {
    ({ icon, title, message, buttons } = dialog);
    open = true;
    return new Promise((newResolve) => (resolve = newResolve));
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
          value: "ok",
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
    return (
      (await show({
        icon: dialog.icon,
        title: dialog.title,
        message: dialog.message,
        buttons: [
          {
            text: dialog.cancelText ?? "Cancel",
            value: "",
            default: false,
          },
          {
            text: dialog.acceptText ?? "Accept",
            value: "ok",
            variant: "filled",
            default: true,
          },
        ],
      })) === "ok"
    );
  }
</script>

<Dialog bind:open {icon} {title} onSelect={resolve}>
  {#if typeof message === "function"}
    {@render message()}
  {:else}
    {message}
  {/if}
  {#snippet actions()}
    {#each buttons as { value, variant, text }}
      <Button type="submit" {value} {variant}>{text}</Button>
    {/each}
  {/snippet}
</Dialog>
