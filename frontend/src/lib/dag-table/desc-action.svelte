<script lang="ts">
  import { Button } from "$lib/kosui/button";
  import type { YTaskProxy } from "$lib/yproxy";
  import { FilePlus2, FileText } from "lucide-svelte";
  import type { Koso } from "./koso.svelte";

  type Props = {
    koso: Koso;
    task: YTaskProxy;
  };
  let { koso, task }: Props = $props();

  let icon = $derived(task.desc ? FileText : FilePlus2);
</script>

<Button
  class="m-0 p-2"
  variant="plain"
  color="primary"
  shape="circle"
  {icon}
  onclick={() => (koso.editor = true)}
/>
