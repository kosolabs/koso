<script lang="ts">
  import { CircularProgress } from "$lib/kosui/progress";
  import { CircleCheck, CircleFadingArrowUp } from "lucide-svelte";
</script>

<div class="flex flex-wrap gap-2 rounded-lg border p-4">
  <div class="border">
    <CircularProgress progress={0.9} class="text-m3-primary">
      90%
    </CircularProgress>
  </div>
  <div class="border">
    <CircleFadingArrowUp class="text-m3-primary" />
  </div>
  <div class="border">
    <CircularProgress progress={0} class="text-m3-primary" />
  </div>
  <div class="border">
    <CircleCheck class="text-m3-primary" />
  </div>
  <div class="border">
    <CircularProgress class="text-m3-primary" />
  </div>
</div>

<div class="flex flex-wrap gap-2 rounded-lg border p-4">
  <div class="border">
    <CircularProgress class="text-m3-primary h-16 w-16" progress={0.9}>
      90%
    </CircularProgress>
  </div>
  <div class="border">
    <CircleFadingArrowUp class="text-m3-primary h-16 w-16" />
  </div>
  <div class="border">
    <CircularProgress class="text-m3-primary h-16 w-16" progress={0} />
  </div>
  <div class="border">
    <CircleCheck class="text-m3-primary h-16 w-16" />
  </div>
  <div class="border">
    <CircularProgress class="text-m3-primary h-16 w-16" />
  </div>
</div>

<div class="flex flex-wrap gap-2 rounded-lg border p-4">
  <div class="border">
    <CircularProgress size="200px" progress={0.9} class="text-m3-primary">
      90%
    </CircularProgress>
  </div>
  <div class="border">
    <CircleFadingArrowUp size="200px" class="text-m3-primary" />
  </div>
  <div class="border">
    <CircularProgress size="200px" progress={0} class="text-m3-primary" />
  </div>
  <div class="border">
    <CircleCheck size="200px" class="text-m3-primary" />
  </div>
  <div class="border">
    <CircularProgress size="200px" class="text-m3-primary" />
  </div>
</div>
