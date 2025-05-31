<script lang="ts">
  import { Input } from "$lib/kosui/input";
  import { Link } from "$lib/kosui/link";
  import { CANCEL, INSERT_CHILD_NODE, INSERT_NODE, OK } from "$lib/shortcuts";
  import { cn } from "$lib/utils";
  import { tick } from "svelte";
  import type { HTMLInputTypeAttribute } from "svelte/elements";

  type Props = {
    value: string;
    placeholder?: string;
    editing?: boolean;
    class?: string;
    "aria-label"?: string;
    type?: HTMLInputTypeAttribute;
    closeFocus?: HTMLElement;
    onclick?: (event: MouseEvent) => void;
    // Callback invoked to apply the edited value.
    // May throw or return a failed promise if save fails
    // but should show users a warning.
    onsave: (value: string) => Promise<void>;
    ondone?: () => void;
    onkeydown?: (event: KeyboardEvent) => void;
    renderValue?: (value: string) => string;
  };

  let {
    value = $bindable(),
    editing = $bindable(false),
    placeholder = "Click to edit",
    class: classes,
    "aria-label": ariaLabel,
    type = "text",
    closeFocus,
    onclick,
    onsave,
    ondone,
    renderValue = (value) => value,
  }: Props = $props();

  let edited: string = $state(value);

  function handleKeyDown(event: KeyboardEvent) {
    if (OK.matches(event)) {
      save();
      event.stopImmediatePropagation();
    } else if (INSERT_NODE.matches(event) || INSERT_CHILD_NODE.matches(event)) {
      save();
    } else if (CANCEL.matches(event)) {
      revert();
      event.stopImmediatePropagation();
    } else {
      event.stopImmediatePropagation();
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
</script>

{#if editing}
  <Input
    bind:value={edited}
    ref={(el) => el.focus()}
    class={cn("h-auto w-full p-1 text-sm", classes)}
    variant="plain"
    color="primary"
    aria-label={ariaLabel}
    name={ariaLabel}
    onclick={(event) => event.stopPropagation()}
    onblur={save}
    onkeydown={handleKeyDown}
    autocomplete="off"
    {type}
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
    {value ? renderValue(value) : placeholder}
  </Link>
{/if}
