<script lang="ts">
  import { Button } from "$lib/kosui/button";
  import type { Action } from "$lib/kosui/command";
  import type { ActionID } from "../command-palette";

  const { icon, title, description, shortcut, callback }: Action<ActionID> =
    $props();
</script>

<Button
  class="flex flex-1 justify-center p-2"
  variant="plain"
  {icon}
  onclick={callback}
>
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
