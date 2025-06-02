<script module lang="ts">
  import {
    Categories,
    getRegistryContext,
  } from "$lib/components/ui/command-palette";
  import { Navbar } from "$lib/components/ui/navbar";
  import { Breadcrumbs } from "$lib/kosui/breadcrumbs";
  import { toTitleCase } from "$lib/kosui/utils";
  import { NavigationAction } from "$lib/navigation-action";
  import { Book } from "lucide-svelte";
  import { onMount, type Snippet } from "svelte";

  export const StorybookNavigationActionIds = {
    Alerts: "/storybook/alerts",
    Autocomplete: "/storybook/autocomplete",
    Avatar: "/storybook/avatar",
    Badge: "/storybook/badge",
    Buttons: "/storybook/buttons",
    Chips: "/storybook/chips",
    CodeMirror: "/storybook/code-mirror",
    Command: "/storybook/command",
    Dialogs: "/storybook/dialogs",
    Fab: "/storybook/fab",
    Goto: "/storybook/goto",
    Inputs: "/storybook/inputs",
    Links: "/storybook/links",
    Markdown: "/storybook/markdown",
    Menus: "/storybook/menus",
    ProgressIndicators: "/storybook/progress-indicators",
    Shortcuts: "/storybook/shortcuts",
    Toggles: "/storybook/toggles",
    Tooltips: "/storybook/tooltips",
  };

  const actions: NavigationAction[] = Object.values(
    StorybookNavigationActionIds,
  ).map((path) => {
    const name = toTitleCase(path.split("/").slice(-1)[0]);

    return new NavigationAction({
      id: path,
      href: path,
      category: Categories.Storybook,
      name: name,
      description: `Navigate to storybook ${name}`,
      icon: Book,
    });
  });
</script>

<script lang="ts">
  type Props = {
    children: Snippet;
  };

  const { children }: Props = $props();

  const command = getRegistryContext();

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
