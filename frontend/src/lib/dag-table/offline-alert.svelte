<script lang="ts">
  import { Alert } from "kosui";

  type Props = {
    offline: boolean;
  };
  let { offline }: Props = $props();
</script>

{#if offline}
  <div class="m-2">
    <Alert variant="tonal" color="plain" elevated outlined>
      Connection to server lost. Working offline.
    </Alert>
  </div>
{/if}
