<script lang="ts">
  import type { Snippet } from "svelte";
  import DialogButton from "./dialog-button.svelte";
  import Dialog from "./dialog.svelte";
  import {
    DialoguerContext,
    setDialoguerContext,
  } from "./dialoguer-context.svelte";

  type Props = {
    children: Snippet;
  };
  let { children }: Props = $props();

  const dialog = setDialoguerContext(new DialoguerContext());
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
  {#snippet actions(props)}
    {#each dialog.buttons as { text, value, default: autofocus }}
      <DialogButton {value} {autofocus} {...props}>
        {text}
      </DialogButton>
    {/each}
  {/snippet}
</Dialog>

{@render children()}
