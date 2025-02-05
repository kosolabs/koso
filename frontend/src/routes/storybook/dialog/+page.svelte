<script lang="ts">
  import { Button } from "$lib/kosui/button";
  import { dialog, Dialoguer } from "$lib/kosui/dialog";
  import { TriangleAlert } from "lucide-svelte";
  import { onMount } from "svelte";

  let result: Promise<boolean>;

  async function showDialog() {
    return await dialog.confirm({
      message,
      title: "Delete Telegram Authorization?",
      icon: TriangleAlert,
    });
  }

  onMount(() => {
    result = showDialog();
  });
</script>

{#snippet message()}
  Deleting the Telegram authorization will disable Koso from sending
  notifications to Telegram.
{/snippet}

<div class="flex flex-col items-center gap-2 p-4">
  <Button onclick={() => (result = showDialog())}>Show Dialog</Button>
  {#await result}
    Click a button in the dialog to get a result.
  {:then result}
    {#if result}
      <div class="text-primary">Accepted!</div>
    {:else}
      <div class="text-destructive">Cancelled!</div>
    {/if}
  {/await}
</div>

<Dialoguer />
