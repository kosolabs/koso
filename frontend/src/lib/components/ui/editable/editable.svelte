<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { KeyBinding } from "$lib/key-binding";
  import { KeyHandlerRegistry } from "$lib/key-handler-registry";
  import { cn } from "$lib/utils";

  type Props = {
    value: string;
    placeholder?: string;
    editing?: boolean;
    class?: string;
    "aria-label"?: string;
    onsave: (value: string) => void;
    ondone?: () => void;
  };
  let {
    value = $bindable(),
    editing = $bindable(false),
    placeholder = "Click to edit",
    class: classes,
    "aria-label": ariaLabel,
    onsave,
    ondone,
  }: Props = $props();

  let edited: string = $state(value);

  const registry = new KeyHandlerRegistry([
    [KeyBinding.SAVE_EDITABLE, save],
    [KeyBinding.REVERT_EDITABLE, revert],
  ]);

  function handleInputKeydown(event: KeyboardEvent) {
    if (
      KeyBinding.INSERT_NODE.matches(event) ||
      KeyBinding.INSERT_CHILD_NODE.matches(event)
    ) {
      save();
      event.preventDefault();
      return;
    }

    // Events after here will not bubble
    event.stopPropagation();
    registry.handle(event);
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
    if (edited === null) return;
    onsave(edited);
    value = edited;
    ondone?.();
    editing = false;
  }

  function revert() {
    if (edited === null) return;
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
