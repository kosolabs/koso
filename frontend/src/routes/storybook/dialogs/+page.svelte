<script lang="ts">
  import { toast } from "$lib/components/ui/sonner";
  import { Button } from "$lib/kosui/button";
  import { Dialog, dialog } from "$lib/kosui/dialog";
  import { TriangleAlert } from "lucide-svelte";

  let noticeResult: Promise<void> | undefined = $state();
  let confirmResult: Promise<boolean> | undefined = $state();

  let open: boolean = $state(false);
  let customResult: string | undefined = $state();
  function handleSelect(value: string) {
    customResult = value;
    console.log("handleSelect", value);
  }

  let customConfirmOpen: boolean = $state(false);
  function handleConfirm(value: string) {
    if (value === "ok") {
      customResult = "four";
    } else {
      customResult = undefined;
    }
    console.log("handleConfirm", value);
  }
</script>

<div class="flex flex-wrap items-center gap-2 rounded-lg border p-4">
  <Button
    tooltip="I'm the Show Notice Button"
    onclick={() => {
      noticeResult = dialog.notice({
        message: "Heads up. A thing just happened!",
        title: "Lookout",
        icon: TriangleAlert,
      });
    }}>Show Notice</Button
  >
  {#if noticeResult}
    {#await noticeResult}
      <div>Dialog Open!</div>
    {:then}
      <div>Dialog Closed!</div>
    {/await}
  {:else}
    <div>Dialog Closed!</div>
  {/if}
</div>

<div class="flex flex-wrap items-center gap-2 rounded-lg border p-4">
  <Button
    onclick={() => {
      confirmResult = dialog.confirm({
        message:
          "Doing this destructive thing will have adverse consequences. Are you sure you want to proceed?",
        title: "Do Something Destructive?",
        icon: TriangleAlert,
      });
    }}>Show Confirm</Button
  >
  {#if confirmResult}
    {#await confirmResult}
      <div>Dialog Open!</div>
    {:then confirmResult}
      {#if confirmResult}
        <div class="text-primary">Accepted!</div>
      {:else}
        <div class="text-destructive">Cancelled!</div>
      {/if}
    {/await}
  {:else}
    <div>Dialog Closed!</div>
  {/if}
</div>

<div class="flex flex-wrap items-center gap-2 rounded-lg border p-4">
  <Button
    onclick={() => {
      open = true;
    }}>Show Custom</Button
  >
  {#if open}
    <div>Dialog Open!</div>
  {:else if customResult}
    <div>Selected: <span class="text-primary">{customResult}</span></div>
  {:else}
    <div>Dialog Closed!</div>
  {/if}
</div>

<Dialog
  bind:open
  icon={TriangleAlert}
  title="Cool Title Batman!"
  onSelect={handleSelect}
>
  <div class="flex flex-col">
    <div>This is a custom dialog.</div>
    <div class="text-center text-4xl">where</div>
    <div class="text-primary text-right">...anything can happen!</div>
  </div>

  {#snippet actions()}
    <Button type="submit" variant="elevated" value="one">One</Button>
    <Button type="submit" variant="filled" value="two">Two</Button>
    <Button
      type="button"
      variant="tonal"
      value="three"
      onclick={() => toast.info("Three clicked!")}>Three</Button
    >
    <Button
      type="button"
      onclick={() => {
        customConfirmOpen = true;
      }}>Four</Button
    >
  {/snippet}
</Dialog>

<Dialog
  bind:open={customConfirmOpen}
  icon={TriangleAlert}
  title="Confirmation Title!"
  onSelect={handleConfirm}
>
  Are you sure you want to select four?

  {#snippet actions()}
    <Button type="submit" value="">No way!</Button>
    <Button type="submit" value="ok" variant="filled">Absolutely!</Button>
  {/snippet}
</Dialog>
