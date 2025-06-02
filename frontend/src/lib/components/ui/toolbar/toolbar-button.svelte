<script lang="ts">
  import { Button } from "$lib/kosui/button";
  import type { Action } from "$lib/kosui/command";

  const {
    icon,
    name: title,
    description,
    shortcut,
    callback,
  }: Action = $props();
</script>

<Button
  class="flex flex-1 justify-center p-2"
  variant="plain"
  {icon}
  onclick={callback}
  title={`${description} ${shortcut?.toString()}`}
>
  <div class="max-sm:hidden">{title}</div>
</Button>
