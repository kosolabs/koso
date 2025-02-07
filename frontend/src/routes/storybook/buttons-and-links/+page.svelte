<script lang="ts">
  import { Button } from "$lib/kosui/button";
  import { Link } from "$lib/kosui/link";
  import { SegmentedButtons } from "$lib/kosui/segmented-buttons";
  import { Link2 } from "lucide-svelte";
</script>

<div class="flex flex-wrap gap-2 rounded-lg border p-4">
  <Button variant="default">Default</Button>
  <Button variant="ghost">Ghost</Button>
  <Button variant="link">Link</Button>
  <Button variant="outline">Outline</Button>
  <Button variant="secondary">Outline</Button>
</div>

<div class="flex flex-col gap-2 rounded-lg border p-4">
  <div>
    An inline link: <Link href="">Link</Link>
  </div>
  <div class="inline-flex gap-1">
    With an icon:
    <Link href="">
      <Link2 size={16} />
      <div>link</div>
    </Link>
  </div>
</div>

<div class="flex flex-col gap-2 rounded-lg border p-4">
  <SegmentedButtons
    buttons={[
      { name: "Light", value: "light" },
      { name: "Dark", value: "dark" },
      { name: "System", value: "system" },
    ]}
  />
</div>
