<script>
  import {
    CircularProgress,
    IndeterminateProgress,
  } from "$lib/components/ui/circular-progress";
  import { CircleCheck, CircleFadingArrowUp } from "lucide-svelte";
</script>

<div class="flex flex-wrap gap-2 rounded-lg border p-4">
  <div class="border">
    <CircularProgress progress={0.9} color="hsl(var(--primary))">
      90%
    </CircularProgress>
  </div>
  <div class="border">
    <CircleFadingArrowUp color="hsl(var(--primary))" />
  </div>
  <div class="border">
    <CircularProgress progress={0} color="hsl(var(--primary))" />
  </div>
  <div class="border">
    <CircleCheck color="hsl(var(--primary))" />
  </div>
  <div class="border">
    <IndeterminateProgress color="hsl(var(--primary))" />
  </div>
</div>

<div class="flex flex-wrap gap-2 rounded-lg border p-4">
  <div class="border">
    <CircularProgress
      class="h-16 w-16"
      progress={0.9}
      color="hsl(var(--primary))"
    >
      90%
    </CircularProgress>
  </div>
  <div class="border">
    <CircleFadingArrowUp class="h-16 w-16" color="hsl(var(--primary))" />
  </div>
  <div class="border">
    <CircularProgress
      class="h-16 w-16"
      progress={0}
      color="hsl(var(--primary))"
    />
  </div>
  <div class="border">
    <CircleCheck class="h-16 w-16" color="hsl(var(--primary))" />
  </div>
  <div class="border">
    <IndeterminateProgress class="h-16 w-16" color="hsl(var(--primary))" />
  </div>
</div>

<div class="flex flex-wrap gap-2 rounded-lg border p-4">
  <div class="border">
    <CircularProgress size="200px" progress={0.9} color="hsl(var(--primary))">
      90%
    </CircularProgress>
  </div>
  <div class="border">
    <CircleFadingArrowUp size="200px" color="hsl(var(--primary))" />
  </div>
  <div class="border">
    <CircularProgress size="200px" progress={0} color="hsl(var(--primary))" />
  </div>
  <div class="border">
    <CircleCheck size="200px" color="hsl(var(--primary))" />
  </div>
  <div class="border">
    <IndeterminateProgress size="200px" color="hsl(var(--primary))" />
  </div>
</div>
