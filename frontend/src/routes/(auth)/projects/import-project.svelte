<script lang="ts">
  import { goto } from "$app/navigation";
  import { KosoError } from "$lib/api";
  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog";
  import { createProject } from "$lib/projects";
  import { Shortcut } from "$lib/shortcuts";

  type Props = {
    open: boolean;
  };
  let { open = $bindable(false) }: Props = $props();

  let files: FileList | null = $state(null);
  let errorMessage: string | null = $state(null);

  async function importProject() {
    const file = files?.item(0) ?? null;
    if (!file) {
      errorMessage = "Select a file.";
      return;
    }

    errorMessage = null;
    let project;
    try {
      project = await createProject(await file.text());
    } catch (err) {
      if (err instanceof KosoError && err.hasReason("TOO_MANY_PROJECTS")) {
        errorMessage =
          "Cannot create new project, you already have too many. Contact us for more!";
      } else if (
        err instanceof KosoError &&
        err.hasReason("MALFORMED_IMPORT")
      ) {
        errorMessage =
          "The selected import file is malformed. Verify the correct file was selected and try again.";
      } else {
        errorMessage = "Something went wrong. Please try again.";
      }
      return;
    }
    await goto(`/projects/${project.project_id}`);
  }
</script>

<Dialog.Root bind:open portal={null}>
  <Dialog.Content
    onkeydown={(event) => {
      event.stopPropagation();
      if (Shortcut.CANCEL.matches(event)) {
        open = false;
      }
    }}
  >
    <Dialog.Header>
      <Dialog.Title>Import Project</Dialog.Title>
      <Dialog.Description
        >Import a new project from a project export file.</Dialog.Description
      >
    </Dialog.Header>
    <div class="flex flex-col gap-2">
      <input id="projectImportFileInput" type="file" bind:files />
      {#if errorMessage}
        <div>{errorMessage}</div>
      {/if}
      <Button onclick={() => importProject()}>Import</Button>
    </div>
  </Dialog.Content>
</Dialog.Root>
