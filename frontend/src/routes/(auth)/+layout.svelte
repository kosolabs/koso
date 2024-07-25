<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { user } from "$lib/auth";
  import {
    onLoginRedirect,
    lastVisitedProjectId,
    DO_NOT_REDIRECT,
  } from "$lib/nav";

  $: if (!$user) {
    if ($onLoginRedirect == DO_NOT_REDIRECT) {
      console.log(`User is logged out. Going to / with DO_NOT redirect set`);
    } else {
      $onLoginRedirect = $page.url.pathname;
      console.log(
        `User is logged out. Going to / with redirect ${$onLoginRedirect}`,
      );
    }

    goto("/");
  }
</script>

<slot></slot>
