<script lang="ts">
  import { goto } from "$app/navigation";
  import { user } from "$lib/auth";
  import { pushRedirectOnUserNotAuthenticated } from "$lib/nav";

  $: if (!$user) {
    pushRedirectOnUserNotAuthenticated();
    goto("/");
  }
</script>

<slot></slot>
