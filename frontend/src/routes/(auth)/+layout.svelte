<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { user } from "$lib/auth";
  import { onLoginRedirect, DO_NOT_REDIRECT } from "$lib/nav";

  $: if (!$user) {
    if ($onLoginRedirect === DO_NOT_REDIRECT) {
      $onLoginRedirect = null;
      console.log(
        "User isn't logged in and DO_NOT_REDIRECT is set. Going to / without a redirect destination.",
      );
    } else {
      $onLoginRedirect = $page.url.pathname;
      console.log(
        `User isn't logged in. Going to / with redirect destination ${$onLoginRedirect}`,
      );
    }

    goto("/");
  }
</script>

<slot></slot>
