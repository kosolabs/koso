<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { user } from "$lib/auth";

  $: if (!$user) {
    const existing = sessionStorage.getItem("login-redirect");
    if (existing !== "DO_NOT") {
      const loginRedirect = $page.url.pathname;
      console.log(
        `User is logged out. Going to / with redirect ${loginRedirect}`,
      );
      sessionStorage.setItem("login-redirect", loginRedirect);
    } else {
      console.log(`User is logged out. Going to / with DO_NOT redirect set`);
    }
    goto("/");
  }
</script>

<slot></slot>
