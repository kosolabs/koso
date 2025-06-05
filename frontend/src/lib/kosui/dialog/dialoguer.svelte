<script lang="ts">
  import type { Snippet } from "svelte";
  import { Input } from "../input";
  import DialogButton from "./dialog-button.svelte";
  import Dialog from "./dialog.svelte";
  import {
    DialoguerContext,
    setDialoguerContext,
    type ButtonProps,
  } from "./dialoguer-context.svelte";

  const dialog = setDialoguerContext(new DialoguerContext());

  type Props = {
    children: Snippet;
  };
  let { children }: Props = $props();

  let value: string = $derived(dialog.inputProps?.value ?? "");

  function getValue(button: ButtonProps<unknown>) {
    if (dialog.inputProps === undefined) {
      return button.value;
    }
    if (button.value === "cancel") {
      return undefined;
    }
    if (button.value === "accept") {
      return value;
    }
    throw new Error("Invalid state");
  }
</script>

<Dialog
  bind:open={dialog.open}
  icon={dialog.icon}
  title={dialog.title}
  onSelect={dialog.resolve}
>
  {#if typeof dialog.message === "function"}
    {@render dialog.message()}
  {:else}
    {dialog.message}
  {/if}

  {#if dialog.inputProps !== undefined}
    <Input class="mt-2 w-full" bind:value autofocus {...dialog.inputProps} />
  {/if}

  {#snippet actions(props)}
    {#each dialog.buttons as button (button)}
      <DialogButton
        variant={button.variant}
        value={getValue(button)}
        autofocus={button.default}
        {...props}
      >
        {button.text}
      </DialogButton>
    {/each}
  {/snippet}
</Dialog>

{@render children()}
