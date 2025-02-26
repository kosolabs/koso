<script lang="ts">
  import { Link } from "$lib/kosui/link";
  import { toTitleCase } from "$lib/kosui/utils";

  const paths: string[] = [
    "/storybook/alerts",
    "/storybook/avatar",
    "/storybook/buttons-and-links",
    "/storybook/dialogs",
    "/storybook/inputs",
    "/storybook/menus",
    "/storybook/progress-indicators",
    "/storybook/shortcuts",
    "/storybook/tooltips",
  ];
</script>

<div class="flex flex-col items-start rounded-lg border p-4">
  {#each paths as path}
    <Link href={path}>
      {toTitleCase(path.split("/").slice(-1)[0])}
    </Link>
  {/each}
</div>
