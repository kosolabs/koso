<script lang="ts">
  import { Button } from "$lib/kosui/button";
  import type { Action } from "$lib/shortcuts";

  const { icon, title, description, shortcut, callback }: Action = $props();
</script>

<Button variant="plain" {icon} onclick={callback}>
  <div class="max-sm:hidden">{title}</div>
  {#snippet tooltip()}
    <div class="flex items-center gap-2">
      {description}
      {#if shortcut}
        <div class="font-bold">
          {shortcut.toString()}
        </div>
      {/if}
    </div>
  {/snippet}
</Button>
