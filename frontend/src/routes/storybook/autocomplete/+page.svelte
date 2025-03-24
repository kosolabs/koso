<script lang="ts">
  import {
    Autocomplete,
    AutocompleteContent,
    AutocompleteInput,
    AutocompleteItem,
  } from "$lib/kosui/autocomplete";

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

  let showCompletions: boolean = $state(false);

  function wantCompletions() {
    if (filter.length > 0) {
      console.log("setting showCompletions to true");
      showCompletions = true;
    }
  }

  $effect(() => {
    if (filter.length > 0 && filteredItems.length > 0) {
      showCompletions = true;
    } else {
      showCompletions = false;
    }
  });
</script>

{#if selected}
  <div>{selected}</div>
{/if}

<div class="flex flex-col gap-4 rounded-lg border p-4">
  <Autocomplete>
    <AutocompleteInput
      bind:value={filter}
      onclick={wantCompletions}
      placeholder="Type stuff..."
    />
    <AutocompleteContent bind:open={showCompletions}>
      {#each filteredItems as item (item.title)}
        <AutocompleteItem onSelect={() => (selected = item.title)}>
          {item.title}
        </AutocompleteItem>
      {/each}
    </AutocompleteContent>
  </Autocomplete>
</div>
