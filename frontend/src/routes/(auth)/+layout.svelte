<script lang="ts">
  import { goto } from "$app/navigation";
  import { user } from "$lib/auth";

  $: if (!$user) {
    goto("/");
  }
</script>

<slot></slot>
