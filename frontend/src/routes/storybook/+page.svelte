<script>
  import { CircularProgress } from "$lib/components/ui/circular-progress";
  import Navbar from "$lib/navbar.svelte";
  import { CircleCheck, CircleFadingArrowUp } from "lucide-svelte";
</script>

<Navbar />

<div class="p-2">
  <div class="flex flex-col gap-2 rounded border p-2">
    <h1 class="text-3xl font-thin">Progress Indicators</h1>

    <div class="flex gap-2">
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
    </div>

    <div class="flex gap-2">
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
    </div>

    <div class="flex gap-2">
      <div class="border">
        <CircularProgress
          size="200px"
          progress={0.9}
          color="hsl(var(--primary))"
        >
          90%
        </CircularProgress>
      </div>
      <div class="border">
        <CircleFadingArrowUp size="200px" color="hsl(var(--primary))" />
      </div>
      <div class="border">
        <CircularProgress
          size="200px"
          progress={0}
          color="hsl(var(--primary))"
        />
      </div>
      <div class="border">
        <CircleCheck size="200px" color="hsl(var(--primary))" />
      </div>
    </div>
  </div>
</div>
