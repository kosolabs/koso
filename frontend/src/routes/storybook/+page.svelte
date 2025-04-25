<script lang="ts">
  import { Link } from "$lib/kosui/link";
  import { toTitleCase } from "$lib/kosui/utils";

  const paths: string[] = [
    "/storybook/alerts",
    "/storybook/autocomplete",
    "/storybook/avatar",
    "/storybook/buttons",
    "/storybook/chips",
    "/storybook/code-mirror",
    "/storybook/command",
    "/storybook/dialogs",
    "/storybook/fab",
    "/storybook/inputs",
    "/storybook/links",
    "/storybook/markdown",
    "/storybook/menus",
    "/storybook/progress-indicators",
    "/storybook/shortcuts",
    "/storybook/toggles",
    "/storybook/tooltips",
  ];
</script>

<div class="flex flex-col items-start rounded-lg border p-4">
  {#each paths as path (path)}
    <Link href={path}>
      {toTitleCase(path.split("/").slice(-1)[0])}
    </Link>
  {/each}
</div>
