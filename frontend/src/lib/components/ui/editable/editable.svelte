<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Shortcut, ShortcutRegistry, type Action } from "$lib/shortcuts";
  import { cn } from "$lib/utils";
  import { Save, Undo2 } from "lucide-svelte";

  type Props = {
    value: string;
    placeholder?: string;
    editing?: boolean;
    class?: string;
    "aria-label"?: string;
    onsave: (value: string) => void;
    ondone?: () => void;
    onkeydown?: (event: KeyboardEvent) => void;
  };
  let {
    value = $bindable(),
    editing = $bindable(false),
    placeholder = "Click to edit",
    class: classes,
    "aria-label": ariaLabel,
    onsave,
    ondone,
    onkeydown,
  }: Props = $props();

  let edited: string = $state(value);

  const actions: Action[] = [
    {
      title: "Save and New Task",
      icon: Save,
      callback: save,
      toolbar: false,
      enabled: () => true,
      shortcut: Shortcut.INSERT_NODE,
    },
    {
      title: "Save and New Child",
      icon: Save,
      callback: save,
      toolbar: false,
      enabled: () => true,
      shortcut: Shortcut.INSERT_CHILD_NODE,
    },
    {
      title: "Save",
      icon: Save,
      callback: save,
      toolbar: false,
      enabled: () => true,
      shortcut: Shortcut.SAVE_EDITABLE,
    },
    {
      title: "Revert",
      icon: Undo2,
      callback: revert,
      toolbar: false,
      enabled: () => true,
      shortcut: Shortcut.REVERT_EDITABLE,
    },
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
    edit();
  }

  export function edit() {
    edited = value;
    editing = true;
  }

  function save() {
    value = edited;
    onsave(edited);
    ondone?.();
    editing = false;
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
