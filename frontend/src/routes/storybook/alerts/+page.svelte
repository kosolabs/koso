<script lang="ts">
  import { Alert, alertColors, alertVariants } from "$lib/kosui/alert";
</script>

<div class="flex flex-col gap-4 rounded-lg border p-4">
  {#each alertVariants as variant}
    {#each alertColors as color}
      <div>
        <Alert {variant} {color}>
          This is an alert with variant and color: ({variant}, {color})
        </Alert>
      </div>
    {/each}
  {/each}
</div>
