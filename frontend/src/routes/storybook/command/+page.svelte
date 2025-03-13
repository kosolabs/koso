<script lang="ts">
  import { Command, CommandInput, CommandItem } from "$lib/kosui/command";

  type Item = {
    title: string;
  };

  const items: Item[] = [
    { title: "Calculator" },
    { title: "Search Emoji" },
    { title: "Calendar" },
    { title: "Settings" },
    { title: "Billing" },
    { title: "Profile" },
  ];

  let filter: string = $state("");

  let filteredItems: Item[] = $derived(
    items.filter((item) => item.title.toLocaleLowerCase().startsWith(filter)),
  );

  let selected: string | undefined = $state();
</script>

{#if selected}
  <div>{selected}</div>
{/if}

<div class="flex flex-col gap-4 rounded-lg border p-4">
  <Command>
    {#snippet input(command)}
      <CommandInput
        bind:value={filter}
        {command}
        placeholder="Type a command or search..."
      />
    {/snippet}
    {#snippet content(command)}
      {#each filteredItems as item (item.title)}
        <CommandItem {command} onSelect={() => (selected = item.title)}>
          {item.title}
        </CommandItem>
      {/each}
    {/snippet}
  </Command>
</div>
