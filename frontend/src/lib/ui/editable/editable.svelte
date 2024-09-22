<script lang="ts">
  import { Button } from "$lib/ui/button";
  import { Input } from "$lib/ui/input";
  import { cn } from "$lib/utils";
  import { createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher<{ save: string }>();

  export let value: string;
  export let placeholder: string = "Click to edit";
  export { classes as class };

  let classes: string = "";
  let edited: string | null = null;

  const ariaLabel: string = $$props["aria-label"];

  function handleInputKeydown(event: KeyboardEvent) {
    event.stopPropagation();

    if (event.key === "Escape") {
      revert();
      event.preventDefault();
      return;
    }

    if (event.key === "Enter") {
      save();
      event.preventDefault();
      return;
    }
  }

  function handleButtonClick(event: MouseEvent) {
    event.stopPropagation();
    event.preventDefault();
    edit();
  }

  export function edit() {
    edited = value;
  }

  function save() {
    if (edited === null) return;
    dispatch("save", edited);
    value = edited;
    edited = null;
  }

  function revert() {
    if (edited === null) return;
    edited = null;
  }
</script>

{#if edited !== null}
  <Input
    class={cn("h-auto bg-background p-1", classes)}
    aria-label={ariaLabel}
    on:click={(event) => event.stopPropagation()}
    on:blur={save}
    on:keydown={handleInputKeydown}
    bind:value={edited}
    autofocus
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
