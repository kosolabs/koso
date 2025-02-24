<script lang="ts">
  import { Input } from "$lib/kosui/input";
  import { Link } from "$lib/kosui/link";
  import {
    Action,
    CANCEL,
    INSERT_CHILD_NODE,
    INSERT_NODE,
    OK,
    ShortcutRegistry,
  } from "$lib/shortcuts";
  import { cn } from "$lib/utils";
  import { tick } from "svelte";

  type Props = {
    value: string;
    placeholder?: string;
    editing?: boolean;
    class?: string;
    "aria-label"?: string;
    closeFocus?: HTMLElement;
    onclick?: (event: MouseEvent) => void;
    // Callback invoked to apply the edited value.
    // May throw or return a failed promise if save fails
    // but should show users a warning.
    onsave: (value: string) => Promise<void>;
    ondone?: () => void;
    onkeydown?: (event: KeyboardEvent) => void;
  };

  let {
    value = $bindable(),
    editing = $bindable(false),
    placeholder = "Click to edit",
    class: classes,
    "aria-label": ariaLabel,
    closeFocus,
    onclick,
    onsave,
    ondone,
    onkeydown,
  }: Props = $props();

  let edited: string = $state(value);
  let ref: HTMLInputElement | undefined = $state();

  const actions: Action[] = [
    new Action({
      callback: save,
      shortcut: INSERT_NODE,
    }),
    new Action({
      callback: save,
      shortcut: INSERT_CHILD_NODE,
    }),
    new Action({
      callback: save,
      shortcut: OK,
    }),
    new Action({
      callback: revert,
      shortcut: CANCEL,
    }),
  ];

  const registry = new ShortcutRegistry(actions);

  function handleInputKeydown(event: KeyboardEvent) {
    onkeydown?.(event);
    if (!registry.handle(event)) {
      event.stopPropagation();
    }
  }

  function handleButtonClick(event: MouseEvent) {
    event.stopPropagation();
    event.preventDefault();
    onclick?.(event);
    tick().then(edit);
  }

  export function edit() {
    edited = value;
    editing = true;
  }

  function save() {
    // Only trigger save if the value has changed.
    // This occurs as part of the normal flow due to
    // both the onblur and "Save" action callbacks triggering.
    if (value === edited) {
      done();
      return;
    }

    onsave(edited).then(() => {
      value = edited;
      done();
    });
  }

  function done() {
    ondone?.();
    closeFocus?.focus();
    editing = false;
  }

  function revert() {
    edited = value;
    done();
  }

  $effect(() => {
    if (ref) {
      ref.focus();
    }
  });
</script>

{#if editing}
  <Input
    bind:ref
    bind:value={edited}
    class={cn("bg-background my-1 h-auto w-full p-1 text-sm", classes)}
    variant="plain"
    color="primary"
    aria-label={ariaLabel}
    name={ariaLabel}
    onclick={(event) => event.stopPropagation()}
    onblur={save}
    onkeydown={handleInputKeydown}
    autocomplete="off"
  />
{:else}
  <Link
    class={cn(
      "h-auto p-0 text-left text-sm text-wrap whitespace-normal",
      classes,
    )}
    aria-label={ariaLabel}
    onclick={handleButtonClick}
    underline="none"
  >
    {value || placeholder}
  </Link>
{/if}
