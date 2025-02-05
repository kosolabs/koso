<script lang="ts">
  import { goto } from "$app/navigation";
  import { Button } from "$lib/kosui/button";
  import { toTitleCase } from "$lib/kosui/utils";

  const paths: string[] = [
    "/storybook/buttons-and-links",
    "/storybook/dialog",
    "/storybook/progress-indicators",
  ];
</script>

<div class="flex flex-col items-start rounded-lg border p-4">
  {#each paths as path}
    <Button variant="link" onclick={() => goto(path)}>
      {toTitleCase(path.split("/").slice(-1)[0])}
    </Button>
  {/each}
</div>
