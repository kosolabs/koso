<script lang="ts">
  import { toast } from "$lib/components/ui/sonner";
  import { Button } from "$lib/kosui/button";
  import { Dialog, dialog, DialogButton } from "$lib/kosui/dialog";
  import { TriangleAlert } from "lucide-svelte";

  let noticeResult: Promise<void> | undefined = $state();
  let confirmResult: Promise<boolean> | undefined = $state();

  type Cardinal = "North" | "South" | "East" | "West";
  let directionResult: Promise<Cardinal> | undefined = $state();

  let open: boolean = $state(false);
  let customResult: "one" | "two" | "four" | undefined = $state();
  function handleSelect(value: "one" | "two" | "four" | undefined) {
    customResult = value;
  }

  let customConfirmOpen: boolean = $state(false);
  function handleConfirm(value: "ok" | "" | undefined) {
    if (value === "ok") {
      open = false;
      handleSelect("four");
    }
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
      <div>Notice Dialog Open!</div>
    {:then}
      <div>Notice Dialog Closed!</div>
    {/await}
  {:else}
    <div>Notice Dialog Closed!</div>
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
      <div>Confirm Dialog Open!</div>
    {:then confirmResult}
      {#if confirmResult}
        <div class="text-m3-primary">Confirm Dialog Accepted!</div>
      {:else}
        <div class="text-m3-error">Confirm Dialog Cancelled!</div>
      {/if}
    {/await}
  {:else}
    <div>Confirm Dialog Closed!</div>
  {/if}
</div>

<div class="flex flex-wrap items-center gap-2 rounded-lg border p-4">
  <Button
    onclick={() => {
      directionResult = dialog.show<Cardinal>({
        message: "Which way would you like to go?",
        title: "Pick a direction.",
        buttons: [
          { text: "North", value: "North" },
          { text: "South", value: "South" },
          { text: "East", value: "East" },
          { text: "West", value: "West" },
        ],
      });
    }}>Show Select</Button
  >
  {#if directionResult}
    {#await directionResult}
      <div>Select Dialog Open!</div>
    {:then directionResult}
      {#if directionResult}
        <div class="text-m3-primary">Select Dialog {directionResult}</div>
      {:else}
        <div class="text-m3-error">Select Dialog Cancelled!</div>
      {/if}
    {/await}
  {:else}
    <div>Select Dialog Closed!</div>
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
    <div>Selected: <span class="text-m3-primary">{customResult}</span></div>
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
  <div class="text-m3-on-primary-container flex flex-col">
    <div>This is a custom dialog.</div>
    <div class="text-center text-4xl">where</div>
    <div class="text-right">...anything can happen!</div>
  </div>

  {#snippet actions(props)}
    <DialogButton variant="elevated" value="one" {...props}>One</DialogButton>
    <DialogButton variant="filled" value="two" {...props}>Two</DialogButton>
    <Button
      variant="tonal"
      value="three"
      onclick={() => toast.info("Three clicked!")}>Three</Button
    >
    <Button
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

  {#snippet actions(props)}
    <DialogButton value="" {...props}>No way!</DialogButton>
    <DialogButton variant="filled" value="ok" {...props}>
      Absolutely!
    </DialogButton>
  {/snippet}
</Dialog>
