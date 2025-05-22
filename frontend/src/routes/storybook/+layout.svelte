<script module lang="ts">
  export const paths: string[] = [
    "/storybook/alerts",
    "/storybook/autocomplete",
    "/storybook/avatar",
    "/storybook/badge",
    "/storybook/buttons",
    "/storybook/chips",
    "/storybook/code-mirror",
    "/storybook/command",
    "/storybook/dialogs",
    "/storybook/fab",
    "/storybook/goto",
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

<script lang="ts">
  import {
    getRegistryContext,
    type ActionID,
  } from "$lib/components/ui/command-palette";

  import { Navbar } from "$lib/components/ui/navbar";
  import { Breadcrumbs } from "$lib/kosui/breadcrumbs";
  import { toTitleCase } from "$lib/kosui/utils";
  import { NavigationAction } from "$lib/navigation-action";
  import { Book } from "lucide-svelte";
  import { onMount, type Snippet } from "svelte";

  type Props = {
    children: Snippet;
  };

  const { children }: Props = $props();

  const command = getRegistryContext();

  const actions: NavigationAction[] = paths.map((path) => {
    const name = toTitleCase(path.split("/").slice(-1)[0]);
    const id = `Storybook${name.replaceAll(" ", "")}` as ActionID;

    return new NavigationAction({
      id,
      href: path,
      title: `Storybook ${name}`,
      description: `Navigate to storybook ${name}`,
      icon: Book,
    });
  });

  console.log(actions);

  onMount(() => {
    return command.register(...actions);
  });
</script>

<Navbar />

<div class="flex flex-col gap-2 p-2">
  <h1 class="text-3xl font-thin">
    <Breadcrumbs />
  </h1>
  <hr />
  {@render children()}
</div>
