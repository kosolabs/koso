<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Action, Shortcut, ShortcutRegistry } from "$lib/shortcuts";
  import { cn } from "$lib/utils";

  type Props = {
    value: string;
    placeholder?: string;
    editing?: boolean;
    class?: string;
    "aria-label"?: string;
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
    onclick,
    onsave,
    ondone,
    onkeydown,
  }: Props = $props();

  let edited: string = $state(value);

  const actions: Action[] = [
    new Action({
      callback: save,
      shortcut: Shortcut.INSERT_NODE,
    }),
    new Action({
      callback: save,
      shortcut: Shortcut.INSERT_CHILD_NODE,
    }),
    new Action({
      callback: save,
      shortcut: Shortcut.OK,
    }),
    new Action({
      callback: revert,
      shortcut: Shortcut.CANCEL,
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
    edit();
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
      ondone?.();
      editing = false;
      return;
    }

    onsave(edited).then(() => {
      value = edited;
      ondone?.();
      editing = false;
    });
  }

  function revert() {
    edited = value;
    ondone?.();
    editing = false;
  }
</script>

{#if editing}
  <Input
    class={cn("h-auto bg-background p-1", classes)}
    aria-label={ariaLabel}
    name={ariaLabel}
    onclick={(event) => event.stopPropagation()}
    onblur={save}
    onkeydown={handleInputKeydown}
    autofocus={true}
    autocomplete="off"
    bind:value={edited}
  />
{:else}
  <Button
    variant="link"
    class={cn("h-auto text-wrap p-0 text-left hover:no-underline", classes)}
    aria-label={ariaLabel}
    onclick={handleButtonClick}
  >
    {value || placeholder}
  </Button>
{/if}
