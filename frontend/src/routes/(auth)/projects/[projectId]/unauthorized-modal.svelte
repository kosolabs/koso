<script lang="ts">
  import { goto } from "$app/navigation";
  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog";
  import { nav } from "$lib/nav.svelte";

  type Props = {
    open: boolean;
  };
  let { open = $bindable(false) }: Props = $props();

  async function goHome(open: boolean) {
    if (!open) {
      // Don't redirect the user back to a project they don't have access too.
      nav.lastVisitedProjectId = null;
      await goto("/projects");
    }
  }
</script>

<Dialog.Root bind:open onOpenChange={goHome}>
  <Dialog.Content>
    <Dialog.Header>
      <Dialog.Title>Unauthorized</Dialog.Title>
      <Dialog.Description>
        You do not have access to the project or the project does not exist.
      </Dialog.Description>
    </Dialog.Header>
    <Dialog.Footer>
      <Button onclick={() => goHome(false)}>Take me home</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
