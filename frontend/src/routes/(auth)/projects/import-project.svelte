<script lang="ts">
  import { goto } from "$app/navigation";
  import { KosoError } from "$lib/api";
  import { Alert } from "$lib/components/ui/alert";
  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog";
  import { createProject } from "$lib/projects";
  import { Shortcut } from "$lib/shortcuts";
  import { HardDriveUpload } from "lucide-svelte";

  type Props = {
    open: boolean;
  };
  let { open = $bindable(false) }: Props = $props();

  let errorMessage: string | null = $state(null);

  function triggerFileSelect() {
    document.getElementById("projectImportFileInput")?.click();
  }

  async function importProject(
    event: Event & {
      currentTarget: EventTarget & HTMLInputElement;
    },
  ) {
    errorMessage = null;

    const files = event.currentTarget.files;
    const file = files && files.item(0);
    if (!file) {
      errorMessage = "Select a file.";
      return;
    }
    if (files.length > 1) {
      errorMessage = "Select a single file.";
      return;
    }

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
        >Import a new project from an exported .json file.</Dialog.Description
      >
    </Dialog.Header>
    <div class="flex flex-col gap-2">
      <input
        id="projectImportFileInput"
        type="file"
        accept=".json,application/JSON"
        multiple={false}
        hidden
        onchange={importProject}
      />
      <Button variant="outline" on:click={triggerFileSelect}>
        <div class="flex items-center space-x-3">
          <HardDriveUpload class="w-5" />
          <span>Upload project .json file</span>
        </div>
      </Button>
      {#if errorMessage}
        <div class="my-2 flex-grow-0">
          <Alert variant="destructive">{errorMessage}</Alert>
        </div>
      {/if}
    </div>
  </Dialog.Content>
</Dialog.Root>
