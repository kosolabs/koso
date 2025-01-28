<script lang="ts">
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import type { Koso } from "$lib/dag-table";
  import type { Status, YTaskProxy } from "$lib/yproxy";
  import { Bot, LoaderCircle } from "lucide-svelte";
  import { TaskStatus, TaskStatusIcon } from ".";

  type Kind = "Rollup" | "Juggled";

  const statuses: Status[] = ["Not Started", "In Progress", "Done"];
  const kinds: Kind[] = ["Rollup", "Juggled"];

  type Props = {
    task: YTaskProxy;
    koso: Koso;
  };
  const { task, koso }: Props = $props();

  let canSetStatus = $derived(
    koso.isEditable(task.id) && task.children.length === 0,
  );
  let canSetKind = $derived(
    koso.isEditable(task.id) && task.children.length > 0,
  );

  function setStatus(status: Status) {
    console.log(status);
  }

  function setKind(kind: Kind) {
    console.log(kind);
  }
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger
    class="flex items-center gap-2"
    title={(task.status || "Not Started") +
      (task.statusTime ? " - " + task.statusTime.toLocaleString() : "")}
    disabled={!canSetStatus && !canSetKind}
  >
    <TaskStatus {task} {koso} />
  </DropdownMenu.Trigger>
  <DropdownMenu.Content portalProps={{ disabled: true }} preventScroll={false}>
    {#if canSetStatus}
      {#each statuses as status}
        <DropdownMenu.Item
          class="flex items-center gap-2 rounded p-2"
          onSelect={() => setStatus(status)}
        >
          <TaskStatusIcon {status} />
          {status}
        </DropdownMenu.Item>
      {/each}
    {/if}
    {#if canSetKind}
      {#each kinds as kind}
        <DropdownMenu.Item
          class="flex items-center gap-2 rounded p-2"
          onSelect={() => setKind(kind)}
        >
          {#if kind === "Rollup"}
            <LoaderCircle />
          {:else if kind === "Juggled"}
            <Bot />
          {/if}
          {kind}
        </DropdownMenu.Item>
      {/each}
    {/if}
  </DropdownMenu.Content>
</DropdownMenu.Root>
