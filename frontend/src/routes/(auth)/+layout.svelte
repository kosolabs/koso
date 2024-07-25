<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { user } from "$lib/auth";
  import { setRedirectOnUserNotAuthenticated } from "$lib/nav";

  $: if (!$user) {
    setRedirectOnUserNotAuthenticated();
    goto("/");
  }
</script>

<slot></slot>
