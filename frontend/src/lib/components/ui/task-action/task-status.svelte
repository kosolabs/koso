<script lang="ts">
  import { ResponsiveText } from "$lib/components/ui/responsive-text";
  import { cn } from "$lib/utils";
  import type { YTaskProxy } from "$lib/yproxy";
  import { TaskStatusIcon } from ".";

  type Props = {
    task: YTaskProxy;
  };
  const { task }: Props = $props();
</script>

<div class={cn("flex items-center gap-2")}>
  <TaskStatusIcon status={task.status} />
  <ResponsiveText>{task.status || "Not Started"}</ResponsiveText>
</div>
