<script lang="ts">
  import { Alert } from "$lib/kosui/alert";

  type Props = {
    offline: boolean;
  };
  let { offline }: Props = $props();
</script>

{#if offline}
  <div class="m-2">
    <Alert variant="outlined" color="secondary">
      Connection to server lost. Working offline.
    </Alert>
  </div>
{/if}
