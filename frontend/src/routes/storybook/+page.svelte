<script lang="ts">
  import { goto } from "$app/navigation";
  import { Button } from "$lib/kosui/button";

  type Component = {
    title: string;
    path: string;
  };

  const components: Component[] = [
    { title: "Dialog", path: "/storybook/dialog" },
    { title: "Progress Indicators", path: "/storybook/progress" },
  ];
</script>

<div>
  {#each components as component}
    <div>
      <Button variant="link" onclick={() => goto(component.path)}>
        {component.title}
      </Button>
    </div>
  {/each}
</div>
