import type { Icon } from "@lucide/svelte";
import { getContext, setContext, type Snippet } from "svelte";
import type { Variants } from "../base";
import type { InputProps } from "../input";

export type ButtonProps<T> = {
  text: string;
  value: T;
  default?: boolean;
} & Variants;

type ShowDialogProps<T> = {
  icon?: typeof Icon;
  title?: string;
  inputProps?: InputProps;
  message: Snippet | string;
  buttons: ButtonProps<T>[];
};

type NoticeDialogProps = {
  icon?: typeof Icon;
  title?: string;
  message: Snippet | string;
  acceptText?: string;
};

type ConfirmDialogProps = {
  icon?: typeof Icon;
  title?: string;
  message: Snippet | string;
  cancelText?: string;
  acceptText?: string;
};

type InputDialogProps = {
  icon?: typeof Icon;
  title?: string;
  props: InputProps;
  message: Snippet | string;
  cancelText?: string;
  acceptText?: string;
};

export class DialoguerContext {
  message: Snippet | string = $state("");
  icon: typeof Icon | undefined = $state();
  title: string | undefined = $state();
  inputProps: InputProps | undefined = $state();
  buttons: ButtonProps<unknown>[] = $state.raw([]);
  resolve: (value: unknown) => void = $state(() => {});
  open: boolean = $state(false);

  show<T>(dialog: ShowDialogProps<T>): Promise<T> {
    ({
      icon: this.icon,
      title: this.title,
      inputProps: this.inputProps,
      message: this.message,
      buttons: this.buttons,
    } = dialog);
    this.open = true;
    return new Promise<unknown>(
      (newResolve) => (this.resolve = newResolve),
    ) as Promise<T>;
  }

  async notice(dialog: NoticeDialogProps): Promise<void> {
    await this.show({
      icon: dialog.icon,
      title: dialog.title,
      inputProps: undefined,
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

  async confirm(dialog: ConfirmDialogProps): Promise<boolean> {
    return await this.show({
      icon: dialog.icon,
      title: dialog.title,
      inputProps: undefined,
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

  async input(dialog: InputDialogProps): Promise<string | undefined> {
    return await this.show({
      icon: dialog.icon,
      title: dialog.title,
      inputProps: dialog.props,
      message: dialog.message,
      buttons: [
        {
          text: dialog.cancelText ?? "Cancel",
          value: "cancel",
        },
        {
          text: dialog.acceptText ?? "Accept",
          value: "accept",
          variant: "filled",
        },
      ],
    });
  }
}

export function setDialoguerContext(state: DialoguerContext): DialoguerContext {
  return setContext<DialoguerContext>(DialoguerContext, state);
}

export function getDialoguerContext(): DialoguerContext {
  const ctx = getContext<DialoguerContext>(DialoguerContext);
  if (!ctx) throw new Error("DialoguerContext is undefined");
  return ctx;
}
